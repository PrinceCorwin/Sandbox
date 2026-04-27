use chrono::Local;
use rusqlite::{types::ValueRef, Connection};
use rust_xlsxwriter::{Format, Workbook};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const EXCEL_MAX_ROWS: u32 = 1_048_576; // total rows including header
const EXCEL_SHEET_NAME_MAX: usize = 31;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Excel,
    Csv,
    Both,
}

#[derive(Debug, Serialize)]
pub struct TableSummary {
    pub name: String,
    pub rows: usize,
    pub truncated: bool,
}

#[derive(Debug, Serialize)]
pub struct DataLoreResult {
    pub run_dir: String,
    pub xlsx_path: Option<String>,
    pub csv_dir: Option<String>,
    pub tables: Vec<TableSummary>,
    pub total_rows: usize,
}

#[tauri::command]
pub fn export_sqlite(
    app: AppHandle,
    db_path: String,
    format: OutputFormat,
) -> Result<DataLoreResult, String> {
    let conn = Connection::open(&db_path).map_err(|e| format!("Cannot open DB: {}", e))?;

    let table_names = list_user_tables(&conn)?;
    if table_names.is_empty() {
        return Err("No user tables found in database.".to_string());
    }

    let run_dir = make_run_dir(&app)?;
    let want_excel = matches!(format, OutputFormat::Excel | OutputFormat::Both);
    let want_csv = matches!(format, OutputFormat::Csv | OutputFormat::Both);

    let csv_dir = if want_csv {
        let d = run_dir.join("csv");
        fs::create_dir_all(&d).map_err(|e| format!("Cannot create csv dir: {}", e))?;
        Some(d)
    } else {
        None
    };

    let mut workbook = if want_excel {
        Some(Workbook::new())
    } else {
        None
    };
    let header_format = Format::new().set_bold();

    let mut used_sheet_names: HashSet<String> = HashSet::new();
    let mut summaries: Vec<TableSummary> = Vec::new();
    let mut total_rows: usize = 0;

    for name in &table_names {
        let headers = list_columns(&conn, name)?;
        let sheet_name = if want_excel {
            unique_sheet_name(name, &mut used_sheet_names)
        } else {
            String::new()
        };

        let summary = export_one_table(
            &conn,
            name,
            &headers,
            workbook.as_mut(),
            &sheet_name,
            &header_format,
            csv_dir.as_deref(),
        )?;

        total_rows += summary.rows;
        summaries.push(summary);
    }

    let xlsx_path = if let Some(mut wb) = workbook {
        let stem = Path::new(&db_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "export".to_string());
        let path = run_dir.join(format!("{}.xlsx", stem));
        wb.save(&path).map_err(|e| format!("Cannot save xlsx: {}", e))?;
        Some(path.to_string_lossy().to_string())
    } else {
        None
    };

    Ok(DataLoreResult {
        run_dir: run_dir.to_string_lossy().to_string(),
        xlsx_path,
        csv_dir: csv_dir.map(|p| p.to_string_lossy().to_string()),
        tables: summaries,
        total_rows,
    })
}

fn make_run_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let stamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let dir = data_dir.join("datalore").join(stamp);
    fs::create_dir_all(&dir).map_err(|e| format!("Cannot create run dir: {}", e))?;
    Ok(dir)
}

fn list_user_tables(conn: &Connection) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master \
             WHERE type='table' AND name NOT LIKE 'sqlite_%' \
             ORDER BY name",
        )
        .map_err(|e| format!("Cannot list tables: {}", e))?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(names)
}

fn list_columns(conn: &Connection, table: &str) -> Result<Vec<String>, String> {
    let sql = format!("PRAGMA table_info(\"{}\")", escape_ident(table));
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Cannot read schema for '{}': {}", table, e))?;
    let cols = stmt
        .query_map([], |row| row.get::<_, String>(1)) // column 1 = name
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    Ok(cols)
}

fn export_one_table(
    conn: &Connection,
    table: &str,
    headers: &[String],
    workbook: Option<&mut Workbook>,
    sheet_name: &str,
    header_format: &Format,
    csv_dir: Option<&Path>,
) -> Result<TableSummary, String> {
    // Optional Excel worksheet
    let mut excel_state: Option<(&mut rust_xlsxwriter::Worksheet, u32, bool)> = None;
    if let Some(wb) = workbook {
        let ws = wb.add_worksheet();
        ws.set_name(sheet_name)
            .map_err(|e| format!("Cannot name sheet '{}': {}", sheet_name, e))?;
        for (col_idx, h) in headers.iter().enumerate() {
            ws.write_string_with_format(0, col_idx as u16, h, header_format)
                .map_err(|e| e.to_string())?;
        }
        excel_state = Some((ws, 1u32, false)); // (ws, next_data_row, truncated)
    }

    // Optional CSV writer
    let mut csv_writer = if let Some(dir) = csv_dir {
        let path = dir.join(format!("{}.csv", sanitize_filename(table)));
        let mut w = csv::Writer::from_path(&path)
            .map_err(|e| format!("Cannot create CSV '{}': {}", path.display(), e))?;
        w.write_record(headers).map_err(|e| e.to_string())?;
        Some(w)
    } else {
        None
    };

    let sql = format!("SELECT * FROM \"{}\"", escape_ident(table));
    let mut stmt = conn
        .prepare(&sql)
        .map_err(|e| format!("Cannot query '{}': {}", table, e))?;
    let col_count = stmt.column_count();
    let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

    let mut row_count: usize = 0;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        row_count += 1;

        // Excel side
        if let Some((ws, next_row, truncated)) = excel_state.as_mut() {
            if *next_row >= EXCEL_MAX_ROWS {
                *truncated = true;
            } else {
                for col in 0..col_count {
                    let value = row.get_ref(col).map_err(|e| e.to_string())?;
                    write_excel_cell(*ws, *next_row, col as u16, value)?;
                }
                *next_row += 1;
            }
        }

        // CSV side
        if let Some(w) = csv_writer.as_mut() {
            let mut record: Vec<String> = Vec::with_capacity(col_count);
            for col in 0..col_count {
                let value = row.get_ref(col).map_err(|e| e.to_string())?;
                record.push(value_to_csv(value));
            }
            w.write_record(&record).map_err(|e| e.to_string())?;
        }
    }

    if let Some(mut w) = csv_writer {
        w.flush().map_err(|e| e.to_string())?;
    }

    let truncated = excel_state.map(|(_, _, t)| t).unwrap_or(false);

    Ok(TableSummary {
        name: table.to_string(),
        rows: row_count,
        truncated,
    })
}

fn write_excel_cell(
    ws: &mut rust_xlsxwriter::Worksheet,
    row: u32,
    col: u16,
    value: ValueRef,
) -> Result<(), String> {
    match value {
        ValueRef::Null => {
            ws.write_blank(row, col, &Format::new())
                .map_err(|e| e.to_string())?;
        }
        ValueRef::Integer(i) => {
            ws.write_number(row, col, i as f64)
                .map_err(|e| e.to_string())?;
        }
        ValueRef::Real(f) => {
            ws.write_number(row, col, f).map_err(|e| e.to_string())?;
        }
        ValueRef::Text(bytes) => {
            let s = std::str::from_utf8(bytes)
                .map(|s| s.to_string())
                .unwrap_or_else(|_| String::from_utf8_lossy(bytes).to_string());
            ws.write_string(row, col, &s).map_err(|e| e.to_string())?;
        }
        ValueRef::Blob(bytes) => {
            ws.write_string(row, col, &format!("<BLOB {} bytes>", bytes.len()))
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn value_to_csv(value: ValueRef) -> String {
    match value {
        ValueRef::Null => String::new(),
        ValueRef::Integer(i) => i.to_string(),
        ValueRef::Real(f) => f.to_string(),
        ValueRef::Text(bytes) => std::str::from_utf8(bytes)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| String::from_utf8_lossy(bytes).to_string()),
        ValueRef::Blob(bytes) => format!("<BLOB {} bytes>", bytes.len()),
    }
}

fn escape_ident(name: &str) -> String {
    name.replace('"', "\"\"")
}

fn sanitize_sheet_name(raw: &str) -> String {
    let stripped: String = raw
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | '?' | '*' | '[' | ']' | ':'))
        .collect();
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        return "Sheet".to_string();
    }
    let limited: String = trimmed.chars().take(EXCEL_SHEET_NAME_MAX).collect();
    limited
}

fn unique_sheet_name(raw: &str, used: &mut HashSet<String>) -> String {
    let base = sanitize_sheet_name(raw);
    if used.insert(base.clone()) {
        return base;
    }
    // Collision: truncate to 28 and append "_<n>".
    let head: String = base.chars().take(EXCEL_SHEET_NAME_MAX - 3).collect();
    for n in 2..=999 {
        let candidate = format!("{}_{}", head, n);
        if used.insert(candidate.clone()) {
            return candidate;
        }
    }
    // Extreme fallback — should never happen with sane input.
    let fallback = format!("Sheet_{}", used.len() + 1);
    used.insert(fallback.clone());
    fallback
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | '?' | '*' | '[' | ']' | ':' | '<' | '>' | '"' | '|' => '_',
            _ => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() {
        "table".to_string()
    } else {
        trimmed.to_string()
    }
}
