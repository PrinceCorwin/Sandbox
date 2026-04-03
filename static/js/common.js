/* ===== SANDBOX — Shared Utilities ===== */

/**
 * Show a toast notification (bottom-right).
 * @param {string} message
 * @param {"success"|"error"|"info"} type
 * @param {number} duration - ms before auto-dismiss
 */
function showToast(message, type = 'success', duration = 3000) {
    const container = document.getElementById('toast-container');
    if (!container) return;

    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.textContent = message;
    container.appendChild(toast);

    setTimeout(() => {
        toast.classList.add('fade-out');
        toast.addEventListener('animationend', () => toast.remove());
    }, duration);
}

/**
 * Show a spinner overlay on an element.
 * @param {HTMLElement} el - The element to overlay
 * @returns {HTMLElement} The overlay element (for later removal)
 */
function showSpinner(el) {
    el.style.position = el.style.position || 'relative';
    const overlay = document.createElement('div');
    overlay.className = 'spinner-overlay';
    overlay.innerHTML = '<div class="spinner"></div>';
    el.appendChild(overlay);
    return overlay;
}

/**
 * Remove a spinner overlay.
 * @param {HTMLElement} overlay - The overlay returned by showSpinner
 */
function hideSpinner(overlay) {
    if (overlay && overlay.parentNode) {
        overlay.remove();
    }
}

/**
 * Show a confirmation dialog. Returns a Promise<boolean>.
 * @param {string} message
 * @returns {Promise<boolean>}
 */
function confirmAction(message) {
    return new Promise((resolve) => {
        const overlay = document.createElement('div');
        overlay.className = 'modal-overlay';
        overlay.innerHTML = `
            <div class="modal confirm-dialog">
                <div class="message">${escapeHtml(message)}</div>
                <div class="modal-actions">
                    <button class="btn" id="confirm-cancel">Cancel</button>
                    <button class="btn btn-danger" id="confirm-ok">Confirm</button>
                </div>
            </div>
        `;
        document.body.appendChild(overlay);

        overlay.querySelector('#confirm-ok').addEventListener('click', () => {
            overlay.remove();
            resolve(true);
        });
        overlay.querySelector('#confirm-cancel').addEventListener('click', () => {
            overlay.remove();
            resolve(false);
        });
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) {
                overlay.remove();
                resolve(false);
            }
        });
    });
}

/**
 * Escape HTML to prevent XSS in dynamic content.
 * @param {string} str
 * @returns {string}
 */
function escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

/**
 * Debounce a function.
 * @param {Function} fn
 * @param {number} ms
 * @returns {Function}
 */
function debounce(fn, ms = 300) {
    let timer;
    return (...args) => {
        clearTimeout(timer);
        timer = setTimeout(() => fn.apply(this, args), ms);
    };
}

/**
 * Upload a file via fetch.
 * @param {string} url
 * @param {File} file
 * @param {string} fieldName
 * @returns {Promise<Response>}
 */
async function uploadFile(url, file, fieldName = 'file') {
    const formData = new FormData();
    formData.append(fieldName, file);
    return fetch(url, { method: 'POST', body: formData });
}

/**
 * Trigger a file download from a fetch Response.
 * @param {Response} response
 * @param {string} filename
 */
async function downloadBlob(response, filename) {
    const blob = await response.blob();
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);
}
