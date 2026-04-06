use calamine::{open_workbook, Reader, Xlsx};
use rust_xlsxwriter::{Format, Workbook};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize)]
pub struct FwResult {
    pub output_path: String,
    pub issues_path: String,
    pub total_flipped: usize,
    pub drawings_processed: usize,
    pub issues_count: usize,
    pub unallocated_total: usize,
}

#[derive(Debug)]
struct LaborRow {
    row_idx: usize,
    component: String,
    size: f64,
}

#[derive(Debug, Serialize)]
struct Issue {
    drawing: String,
    fw_count: usize,
    flipped: usize,
    unallocated: usize,
    issue: String,
}

#[tauri::command]
pub fn run_fw_allocation(
    app: AppHandle,
    fw_path: String,
    labor_path: String,
) -> Result<FwResult, String> {
    // --- Load FW counts ---
    let mut fw_wb: Xlsx<_> =
        open_workbook(&fw_path).map_err(|e| format!("Cannot open FW file: {}", e))?;

    let fw_sheet_name = fw_wb
        .sheet_names()
        .first()
        .cloned()
        .ok_or("FW file has no sheets")?;

    let fw_range = fw_wb
        .worksheet_range(&fw_sheet_name)
        .map_err(|e| format!("Cannot read FW sheet: {}", e))?;

    let mut fw_counts: HashMap<String, usize> = HashMap::new();
    for row in fw_range.rows().skip(1) {
        if row.len() >= 2 {
            let drawing = row[0].to_string().trim().to_string();
            if let Ok(count) = row[1].to_string().trim().parse::<f64>() {
                if !drawing.is_empty() && count > 0.0 {
                    fw_counts.insert(drawing, count as usize);
                }
            }
        }
    }

    // --- Load labor file with calamine to read data ---
    let mut labor_wb: Xlsx<_> =
        open_workbook(&labor_path).map_err(|e| format!("Cannot open labor file: {}", e))?;

    let labor_sheet_name = labor_wb
        .sheet_names()
        .first()
        .cloned()
        .ok_or("Labor file has no sheets")?;

    let labor_range = labor_wb
        .worksheet_range(&labor_sheet_name)
        .map_err(|e| format!("Cannot read labor sheet: {}", e))?;

    let headers: Vec<String> = labor_range
        .rows()
        .next()
        .ok_or("Labor file is empty")?
        .iter()
        .map(|c| c.to_string().trim().to_string())
        .collect();

    let find_col = |name: &str| -> Result<usize, String> {
        headers
            .iter()
            .position(|h| h == name)
            .ok_or(format!("Column '{}' not found in labor file", name))
    };

    let dwg_col = find_col("Drawing Number")?;
    let comp_col = find_col("Component")?;
    let size_col = find_col("Size")?;
    let qty_col = find_col("Quantity")?;
    let sf_col = find_col("ShopField")?;

    // Read all rows into memory
    let all_rows: Vec<Vec<String>> = labor_range
        .rows()
        .map(|r| r.iter().map(|c| c.to_string().trim().to_string()).collect())
        .collect();

    // Build PIPE footage by drawing+size
    let mut pipe_footage: HashMap<String, HashMap<u64, f64>> = HashMap::new();
    for row in all_rows.iter().skip(1) {
        if row.len() <= std::cmp::max(comp_col, std::cmp::max(size_col, qty_col)) {
            continue;
        }
        if row[comp_col] == "PIPE" && !row[dwg_col].is_empty() {
            let dwg = row[dwg_col].clone();
            if let (Ok(size), Ok(qty)) = (row[size_col].parse::<f64>(), row[qty_col].parse::<f64>())
            {
                *pipe_footage
                    .entry(dwg)
                    .or_default()
                    .entry(size.to_bits())
                    .or_insert(0.0) += qty;
            }
        }
    }

    // Build index of BW/SW rows (ShopField == 1)
    let mut labor_bw_sw: HashMap<String, Vec<LaborRow>> = HashMap::new();
    for (row_idx, row) in all_rows.iter().enumerate().skip(1) {
        if row.len() <= std::cmp::max(comp_col, std::cmp::max(size_col, sf_col)) {
            continue;
        }
        let comp = &row[comp_col];
        let sf = &row[sf_col];
        if (comp == "BW" || comp == "SW") && (sf == "1" || sf == "1.0") {
            let dwg = row[dwg_col].clone();
            if let Ok(size) = row[size_col].parse::<f64>() {
                labor_bw_sw.entry(dwg).or_default().push(LaborRow {
                    row_idx,
                    component: comp.clone(),
                    size,
                });
            }
        }
    }

    // --- Run allocation ---
    let mut total_flipped: usize = 0;
    let mut drawings_processed: usize = 0;
    let mut issues: Vec<Issue> = Vec::new();

    // Track which rows get flipped (row_idx -> true)
    let mut flipped_rows_global: HashSet<usize> = HashSet::new();

    let mut sorted_drawings: Vec<_> = fw_counts.iter().collect();
    sorted_drawings.sort_by_key(|(dwg, _)| (*dwg).clone());

    for (dwg, &fw_count) in &sorted_drawings {
        if fw_count == 0 {
            continue;
        }

        let candidates = match labor_bw_sw.get(*dwg) {
            Some(c) => c,
            None => {
                issues.push(Issue {
                    drawing: dwg.to_string(),
                    fw_count,
                    flipped: 0,
                    unallocated: fw_count,
                    issue: "NO BW/SW labor rows found".to_string(),
                });
                continue;
            }
        };

        drawings_processed += 1;
        let mut remaining = fw_count;
        let mut flipped_rows: HashSet<usize> = HashSet::new();

        let footage = pipe_footage.get(*dwg);
        let all_sizes: HashSet<u64> = candidates.iter().map(|c| c.size.to_bits()).collect();
        let largest_size = f64::from_bits(*all_sizes.iter().max().unwrap());

        let mut sizes_by_footage: Vec<f64> = all_sizes.iter().map(|b| f64::from_bits(*b)).collect();
        sizes_by_footage.sort_by(|a, b| {
            let fa = footage
                .and_then(|f| f.get(&a.to_bits()))
                .copied()
                .unwrap_or(0.0);
            let fb = footage
                .and_then(|f| f.get(&b.to_bits()))
                .copied()
                .unwrap_or(0.0);
            fb.partial_cmp(&fa)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal))
        });

        // Header weld — one BW (prefer BW) at largest size
        let mut header_candidates: Vec<&LaborRow> = candidates
            .iter()
            .filter(|c| (c.size - largest_size).abs() < 0.001)
            .collect();
        header_candidates.sort_by_key(|c| if c.component == "BW" { 0 } else { 1 });

        if let Some(hc) = header_candidates.first() {
            flipped_rows.insert(hc.row_idx);
            flipped_rows_global.insert(hc.row_idx);
            remaining -= 1;
            total_flipped += 1;
        }

        if remaining == 0 {
            continue;
        }

        // Walk sizes by footage descending
        for size in &sizes_by_footage {
            if remaining == 0 {
                break;
            }
            let mut size_candidates: Vec<&LaborRow> = candidates
                .iter()
                .filter(|c| {
                    (c.size - size).abs() < 0.001 && !flipped_rows.contains(&c.row_idx)
                })
                .collect();
            size_candidates.sort_by_key(|c| if c.component == "BW" { 0 } else { 1 });

            for sc in size_candidates {
                if remaining == 0 {
                    break;
                }
                flipped_rows.insert(sc.row_idx);
                flipped_rows_global.insert(sc.row_idx);
                remaining -= 1;
                total_flipped += 1;
            }
        }

        if remaining > 0 {
            issues.push(Issue {
                drawing: dwg.to_string(),
                fw_count,
                flipped: fw_count - remaining,
                unallocated: remaining,
                issue: "EXHAUSTED — not enough BW/SW rows".to_string(),
            });
        }
    }

    // --- Write output files ---
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    let output_dir = data_dir.join("fw_output");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    // Write modified labor file
    let output_path = output_dir.join("All_Labor_FW_Applied.xlsx");
    write_labor_output(&all_rows, sf_col, &flipped_rows_global, &output_path)?;

    // Write issues log
    let issues_path = output_dir.join("FW_Unallocated_Issues.xlsx");
    write_issues_output(&issues, &issues_path)?;

    let unallocated_total: usize = issues.iter().map(|i| i.unallocated).sum();

    Ok(FwResult {
        output_path: output_path.to_string_lossy().to_string(),
        issues_path: issues_path.to_string_lossy().to_string(),
        total_flipped,
        drawings_processed,
        issues_count: issues.len(),
        unallocated_total,
    })
}

fn write_labor_output(
    all_rows: &[Vec<String>],
    sf_col: usize,
    flipped_rows: &HashSet<usize>,
    output_path: &PathBuf,
) -> Result<(), String> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();

    let bold = Format::new().set_bold();

    for (row_idx, row) in all_rows.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let value = if row_idx > 0 && col_idx == sf_col && flipped_rows.contains(&row_idx) {
                "2".to_string()
            } else {
                cell.clone()
            };

            // Try to write as number if possible
            if row_idx == 0 {
                ws.write_string_with_format(row_idx as u32, col_idx as u16, &value, &bold)
                    .map_err(|e| e.to_string())?;
            } else if let Ok(num) = value.parse::<f64>() {
                ws.write_number(row_idx as u32, col_idx as u16, num)
                    .map_err(|e| e.to_string())?;
            } else {
                ws.write_string(row_idx as u32, col_idx as u16, &value)
                    .map_err(|e| e.to_string())?;
            }
        }
    }

    wb.save(output_path).map_err(|e| e.to_string())
}

fn write_issues_output(issues: &[Issue], output_path: &PathBuf) -> Result<(), String> {
    let mut wb = Workbook::new();
    let ws = wb.add_worksheet();
    ws.set_name("FW Issues").map_err(|e| e.to_string())?;

    let bold = Format::new().set_bold();
    let headers = [
        "Drawing Number",
        "FW Count",
        "Flipped",
        "Unallocated",
        "Issue",
    ];

    for (col, header) in headers.iter().enumerate() {
        ws.write_string_with_format(0, col as u16, *header, &bold)
            .map_err(|e| e.to_string())?;
    }

    for (row_idx, issue) in issues.iter().enumerate() {
        let row = (row_idx + 1) as u32;
        ws.write_string(row, 0, &issue.drawing)
            .map_err(|e| e.to_string())?;
        ws.write_number(row, 1, issue.fw_count as f64)
            .map_err(|e| e.to_string())?;
        ws.write_number(row, 2, issue.flipped as f64)
            .map_err(|e| e.to_string())?;
        ws.write_number(row, 3, issue.unallocated as f64)
            .map_err(|e| e.to_string())?;
        ws.write_string(row, 4, &issue.issue)
            .map_err(|e| e.to_string())?;
    }

    // Auto-fit column widths
    for col in 0..5u16 {
        ws.set_column_width(col, 20).map_err(|e| e.to_string())?;
    }

    wb.save(output_path).map_err(|e| e.to_string())
}
