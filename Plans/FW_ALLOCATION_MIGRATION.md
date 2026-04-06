# FW Allocation — Migration Reference

## Task

Migrate this miniapp from Streamlit + Python to the Tauri app. The algorithm and business logic below are correct and tested. Port the logic, not the framework code.

## Algorithm

Takes a count of field welds per ISO drawing (from an Excel pivot table) and flips the ShopField value on BW/SW labor rows from 1 (Shop) to 2 (Field). The FW count doesn't specify pipe size or material, so the algorithm uses heuristics:

1. **Header weld first:** Find the largest pipe size on the drawing. Flip one BW (prefer BW over SW) at that size to Field. This guarantees at least one field weld at the header size, since the header connects to the next drawing.
2. **Longest pipe run next:** Sort all pipe sizes by total PIPE linear feet descending. Starting with the most footage, flip BW/SW rows at that size to Field one at a time, decrementing the remaining FW count.
3. **Cascade:** When all BW/SW at a size are exhausted, move to the next-longest pipe size. Repeat until FW count is zero or all BW/SW rows are used.
4. **BW over SW:** When choosing between BW and SW at the same size, always pick BW first. SW is rare at the same size anyway.

The header weld is a safety net — usually the largest pipe also has the most footage, so the algorithm continues flipping at that size after the header weld.

## Inputs

- **FW Count file:** Excel (.xlsx) with first two columns = drawing number and weld count, starting at row 2. Column headers vary between projects.
- **Labor file:** Excel (.xlsx) with one sheet containing columns: Drawing Number, Component, Size, Quantity, ShopField. Contains both PIPE rows (used for footage calculation) and BW/SW rows (targets for flipping).

## Outputs

- **Modified labor file:** Same as input with ShopField values changed from 1 → 2 on allocated rows.
- **Issues log:** Excel file listing drawings that had no BW/SW rows or insufficient rows to cover the FW count. Columns: Drawing Number, FW Count, Flipped, Unallocated, Issue.

## Current Working Code (Streamlit + Python + openpyxl)

```python
import streamlit as st
import openpyxl
from collections import defaultdict
from io import BytesIO

st.set_page_config(page_title="FW Allocation", page_icon="🔧", layout="wide")
st.title("Field Weld Allocation")
st.markdown("Allocates field weld counts from a pivot table to BW/SW labor rows. "
            "Prioritizes largest pipe size first (header weld), then longest pipe run by linear feet.")

fw_file = st.file_uploader("Upload FW Count file (pivot with Drawing Number + Count)", type="xlsx")
labor_file = st.file_uploader("Upload Labor file (must have PIPE and BW/SW rows)", type="xlsx")

if fw_file and labor_file and st.button("Run Allocation", type="primary"):
    with st.spinner("Processing..."):
        # Load FW counts
        fw_wb = openpyxl.load_workbook(fw_file, read_only=True)
        fw_ws = fw_wb.active
        fw_counts = {}
        for row in fw_ws.iter_rows(min_row=2, values_only=True):
            if row[0] and row[1]:
                fw_counts[str(row[0]).strip()] = int(row[1])

        # Load workbook for modification
        labor_file.seek(0)
        wb = openpyxl.load_workbook(labor_file)
        ws = wb[wb.sheetnames[0]]

        headers = [cell.value for cell in ws[1]]
        dwg_col = headers.index("Drawing Number")
        comp_col = headers.index("Component")
        size_col = headers.index("Size")
        qty_col = headers.index("Quantity")
        sf_col = headers.index("ShopField")

        # Build PIPE linear feet by drawing+size
        pipe_footage = defaultdict(lambda: defaultdict(float))
        for row in ws.iter_rows(min_row=2, values_only=True):
            if row[comp_col] == "PIPE" and row[dwg_col]:
                dwg = str(row[dwg_col]).strip()
                size = row[size_col]
                qty = row[qty_col]
                if size is not None and qty is not None:
                    try:
                        pipe_footage[dwg][float(size)] += float(qty)
                    except (ValueError, TypeError):
                        pass

        # Build index of BW/SW rows (ShopField == 1 only)
        labor_bw_sw = defaultdict(list)
        for row_num in range(2, ws.max_row + 1):
            comp = ws.cell(row=row_num, column=comp_col + 1).value
            sf = ws.cell(row=row_num, column=sf_col + 1).value
            if comp in ("BW", "SW") and sf == 1:
                dwg = str(ws.cell(row=row_num, column=dwg_col + 1).value or "").strip()
                size_raw = ws.cell(row=row_num, column=size_col + 1).value
                try:
                    size_f = float(size_raw)
                except (ValueError, TypeError):
                    continue
                labor_bw_sw[dwg].append((row_num, comp, size_f))

        # Run allocation
        total_flipped = 0
        drawings_processed = 0
        issues = []

        for dwg, fw_remaining in sorted(fw_counts.items()):
            if fw_remaining <= 0:
                continue

            candidates = labor_bw_sw.get(dwg, [])
            if not candidates:
                issues.append((dwg, fw_remaining, 0, fw_remaining, "NO BW/SW labor rows found"))
                continue

            drawings_processed += 1
            remaining = fw_remaining
            flipped_rows = set()

            footage = pipe_footage.get(dwg, {})
            all_sizes = set(c[2] for c in candidates)
            largest_size = max(all_sizes)
            sizes_by_footage = sorted(all_sizes, key=lambda s: (footage.get(s, 0), s), reverse=True)

            # Header weld — one BW (prefer BW over SW) at largest size
            header_candidates = [c for c in candidates if c[2] == largest_size]
            header_candidates.sort(key=lambda c: (0 if c[1] == "BW" else 1))

            if header_candidates:
                row_num, comp, size = header_candidates[0]
                ws.cell(row=row_num, column=sf_col + 1, value=2)
                flipped_rows.add(row_num)
                remaining -= 1
                total_flipped += 1

            if remaining <= 0:
                continue

            # Walk sizes by footage descending, flip BW then SW
            for size in sizes_by_footage:
                if remaining <= 0:
                    break
                size_candidates = [c for c in candidates if c[2] == size and c[0] not in flipped_rows]
                size_candidates.sort(key=lambda c: (0 if c[1] == "BW" else 1))
                for row_num, comp, sz in size_candidates:
                    if remaining <= 0:
                        break
                    ws.cell(row=row_num, column=sf_col + 1, value=2)
                    flipped_rows.add(row_num)
                    remaining -= 1
                    total_flipped += 1

            if remaining > 0:
                issues.append((dwg, fw_remaining, fw_remaining - remaining, remaining, "EXHAUSTED — not enough BW/SW rows"))

        # Save to buffer
        output_buffer = BytesIO()
        wb.save(output_buffer)
        output_buffer.seek(0)

        # Save issues workbook
        issues_buffer = BytesIO()
        issues_wb = openpyxl.Workbook()
        issues_ws = issues_wb.active
        issues_ws.title = "FW Issues"
        issues_ws.append(["Drawing Number", "FW Count", "Flipped", "Unallocated", "Issue"])
        for row in issues:
            issues_ws.append(row)
        from openpyxl.styles import Font
        for cell in issues_ws[1]:
            cell.font = Font(bold=True)
        for col in issues_ws.columns:
            max_len = max(len(str(c.value or "")) for c in col)
            issues_ws.column_dimensions[col[0].column_letter].width = min(max_len + 2, 50)
        issues_wb.save(issues_buffer)
        issues_buffer.seek(0)

    # Results
    st.success(f"Done — {total_flipped} welds flipped to Field across {drawings_processed} drawings")

    if issues:
        st.warning(f"{len(issues)} drawings had issues ({sum(r[3] for r in issues)} unallocated welds)")

    col1, col2 = st.columns(2)
    with col1:
        st.download_button("Download Updated Labor", data=output_buffer,
                           file_name="All_Labor_FW_Applied.xlsx",
                           mime="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                           type="primary")
    with col2:
        st.download_button("Download Issues Log", data=issues_buffer,
                           file_name="FW_Unallocated_Issues.xlsx",
                           mime="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")
```

## Domain Context

- **ISO drawings** — engineering drawings showing pipe routing, identified by drawing numbers
- **Components** — PIPE (linear pipe), BW (butt weld), SW (socket weld), fittings, flanges, etc.
- **Size** — nominal pipe size in inches (e.g., 2, 4, 6, 8, 12, 24)
- **ShopField** — 1 = fabrication shop, 2 = field
- **Field welds (FW)** — welds performed at the construction site rather than in the shop
- **Linear feet / footage** — quantity of pipe measured in feet

## UI Requirements

- Two file upload inputs (FW Count file, Labor file) accepting .xlsx
- A Run button that triggers processing
- Status/result display showing total flipped, drawings processed, and any issues
- Two download outputs: the modified labor file and the issues log
- Clear error messages if expected columns are missing from the uploaded files
