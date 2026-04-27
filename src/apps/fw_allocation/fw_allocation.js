/* ===== FW Allocation — App Logic ===== */

function fwApp() {
    return {
        fwPath: null,
        fwFileName: null,
        laborPath: null,
        laborFileName: null,
        processing: false,
        result: null,

        init() {},

        async pickFwFile() {
            const path = await pickFile({
                filters: [{ name: 'Excel Files', extensions: ['xlsx'] }],
            });
            if (path) {
                this.fwPath = path;
                this.fwFileName = path.split(/[\\/]/).pop();
            }
        },

        async pickLaborFile() {
            const path = await pickFile({
                filters: [{ name: 'Excel Files', extensions: ['xlsx'] }],
            });
            if (path) {
                this.laborPath = path;
                this.laborFileName = path.split(/[\\/]/).pop();
            }
        },

        async runAllocation() {
            if (!this.fwPath || !this.laborPath) return;

            this.processing = true;
            this.result = null;

            try {
                this.result = await invoke('run_fw_allocation', {
                    fwPath: this.fwPath,
                    laborPath: this.laborPath,
                });
                showToast(
                    `Done — ${this.result.total_flipped} welds flipped across ${this.result.drawings_processed} drawings`
                );
            } catch (e) {
                showToast('Allocation failed: ' + e, 'error');
            } finally {
                this.processing = false;
            }
        },

        async saveOutputFile() {
            if (!this.result) return;
            const dest = await pickSavePath({
                defaultPath: 'All_Labor_FW_Applied.xlsx',
                filters: [{ name: 'Excel Files', extensions: ['xlsx'] }],
            });
            if (dest) {
                try {
                    await invoke('app_copy_file', { src: this.result.output_path, dst: dest });
                    showToast('File saved');
                } catch (e) {
                    showToast('Save failed: ' + e, 'error');
                }
            }
        },

        async saveIssuesFile() {
            if (!this.result) return;
            const dest = await pickSavePath({
                defaultPath: 'FW_Unallocated_Issues.xlsx',
                filters: [{ name: 'Excel Files', extensions: ['xlsx'] }],
            });
            if (dest) {
                try {
                    await invoke('app_copy_file', { src: this.result.issues_path, dst: dest });
                    showToast('File saved');
                } catch (e) {
                    showToast('Save failed: ' + e, 'error');
                }
            }
        },
    };
}
