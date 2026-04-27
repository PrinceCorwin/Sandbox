/* ===== DataLore — App Logic ===== */

function dataloreApp() {
    return {
        dbPath: null,
        dbFileName: null,
        format: 'excel',
        processing: false,
        result: null,

        init() {},

        async pickDb() {
            const path = await pickFile({
                filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite', 'sqlite3'] }],
            });
            if (path) {
                this.dbPath = path;
                this.dbFileName = path.split(/[\\/]/).pop();
            }
        },

        async runExport() {
            if (!this.dbPath) return;

            this.processing = true;
            this.result = null;

            try {
                this.result = await invoke('export_sqlite', {
                    dbPath: this.dbPath,
                    format: this.format,
                });
                showToast(
                    `Exported ${this.result.tables.length} tables / ${this.result.total_rows.toLocaleString()} rows`
                );
            } catch (e) {
                showToast('Export failed: ' + e, 'error');
            } finally {
                this.processing = false;
            }
        },

        anyTruncated() {
            return (this.result?.tables || []).some((t) => t.truncated);
        },

        async saveAs() {
            if (!this.result) return;

            // Excel-only: single-file save dialog.
            if (this.format === 'excel') {
                const stem = (this.dbFileName || 'export').replace(/\.[^.]+$/, '');
                const dest = await pickSavePath({
                    defaultPath: `${stem}.xlsx`,
                    filters: [{ name: 'Excel Workbook', extensions: ['xlsx'] }],
                });
                if (!dest) return;
                try {
                    await invoke('app_copy_file', { src: this.result.xlsx_path, dst: dest });
                    showToast('Saved');
                } catch (e) {
                    showToast('Save failed: ' + e, 'error');
                }
                return;
            }

            // CSV or Both: copy the whole run dir (xlsx + csv/) into a chosen folder.
            const { open } = window.__TAURI__.dialog;
            const dir = await open({ directory: true, multiple: false });
            if (!dir) return;
            try {
                await invoke('app_copy_tree', { src: this.result.run_dir, dst: dir });
                showToast('Saved');
            } catch (e) {
                showToast('Save failed: ' + e, 'error');
            }
        },

        async revealRun() {
            if (!this.result) return;
            try {
                await invoke('app_open_path', { path: this.result.run_dir });
            } catch (e) {
                showToast('Cannot open folder: ' + e, 'error');
            }
        },
    };
}
