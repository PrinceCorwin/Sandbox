/* ===== SANDBOX — Alpine.js App Logic ===== */

function sandbox() {
    return {
        // State
        sidebarOpen: true,
        viewMode: 'card',
        searchQuery: '',
        activeTag: 'all',
        sortBy: 'order',
        apps: [],
        tagOrder: [],
        editingApp: null,
        editForm: { title: '', description: '', tags: [], thumbnail: null, thumbnailDataUrl: null, icon: '' },
        showAddTag: false,
        newTagName: '',
        showSettings: false,
        showHelp: false,
        tagDragIdx: null,

        async init() {
            try {
                const [apps, config] = await Promise.all([
                    invoke('discover_apps'),
                    invoke('get_config'),
                ]);
                this.apps = apps;
                this.tagOrder = config.tag_order || config.tags || [];

                for (const app of this.apps) {
                    if (app.thumbnail) {
                        try {
                            app.thumbnailDataUrl = await invoke('get_thumbnail_base64', { filename: app.thumbnail });
                        } catch { /* ignore */ }
                    }
                }
            } catch (e) {
                console.error('Failed to load app data:', e);
                showToast('Failed to load apps', 'error');
            }

            const prefs = JSON.parse(localStorage.getItem('sandbox_prefs') || '{}');
            if (prefs.sidebarOpen !== undefined) this.sidebarOpen = prefs.sidebarOpen;
            if (prefs.viewMode) this.viewMode = prefs.viewMode;
            if (prefs.sortBy) this.sortBy = prefs.sortBy;

            this.$watch('showAddTag', (val) => {
                if (val) this.$nextTick(() => this.$refs.newTagInput?.focus());
            });
        },

        persistPrefs() {
            localStorage.setItem('sandbox_prefs', JSON.stringify({
                sidebarOpen: this.sidebarOpen,
                viewMode: this.viewMode,
                sortBy: this.sortBy,
            }));
        },

        toggleSidebar() {
            this.sidebarOpen = !this.sidebarOpen;
            this.persistPrefs();
        },

        setViewMode(mode) {
            this.viewMode = mode;
            this.persistPrefs();
        },

        setTag(tag) {
            this.activeTag = tag;
        },

        get filteredApps() {
            let list = [...this.apps];

            if (this.activeTag === 'favorites') {
                list = list.filter(a => a.favorite);
            } else if (this.activeTag === 'uncategorized') {
                list = list.filter(a => !a.tags || a.tags.length === 0);
            } else if (this.activeTag !== 'all') {
                list = list.filter(a => a.tags && a.tags.includes(this.activeTag));
            }

            if (this.searchQuery.trim()) {
                const q = this.searchQuery.toLowerCase();
                list = list.filter(a =>
                    a.title.toLowerCase().includes(q) ||
                    a.description.toLowerCase().includes(q)
                );
            }

            return this.sortList(list);
        },

        get favoritedApps() {
            return this.sortList(this.apps.filter(a => a.favorite));
        },

        get sortedTags() {
            return [...this.tagOrder].sort((a, b) => a.localeCompare(b));
        },

        get sectionTitle() {
            if (this.activeTag === 'all') return 'All Apps';
            if (this.activeTag === 'favorites') return 'Favorites';
            if (this.activeTag === 'uncategorized') return 'Uncategorized';
            return this.activeTag;
        },

        sortList(list) {
            const copy = [...list];
            switch (this.sortBy) {
                case 'alpha':
                    return copy.sort((a, b) => a.title.localeCompare(b.title));
                case 'recent':
                    return copy.sort((a, b) => (b.last_used || '').localeCompare(a.last_used || ''));
                case 'added':
                    return copy.sort((a, b) => (b.date_added || '').localeCompare(a.date_added || ''));
                default:
                    return copy.sort((a, b) => a.order - b.order);
            }
        },

        // ── Actions ──

        openEdit(appId) {
            const app = this.apps.find(a => a.id === appId);
            if (!app) return;
            this.editForm = {
                title: app.title,
                description: app.description,
                tags: [...(app.tags || [])],
                thumbnail: app.thumbnail,
                thumbnailDataUrl: app.thumbnailDataUrl || null,
                icon: app.icon,
            };
            this.editingApp = appId;
        },

        toggleEditTag(tag) {
            const idx = this.editForm.tags.indexOf(tag);
            if (idx >= 0) {
                this.editForm.tags.splice(idx, 1);
            } else {
                this.editForm.tags.push(tag);
            }
        },

        async saveEdit() {
            const appId = this.editingApp;
            const spinner = showSpinner(document.querySelector('.modal'));
            try {
                await invoke('update_app', {
                    appId,
                    data: {
                        title: this.editForm.title,
                        description: this.editForm.description,
                        tags: this.editForm.tags,
                        thumbnail: this.editForm.thumbnail,
                    },
                });
                const app = this.apps.find(a => a.id === appId);
                if (app) {
                    app.title = this.editForm.title;
                    app.description = this.editForm.description;
                    app.tags = [...this.editForm.tags];
                    app.thumbnail = this.editForm.thumbnail;
                    app.thumbnailDataUrl = this.editForm.thumbnailDataUrl;
                }
                this.editingApp = null;
                showToast('App updated');
            } catch (e) {
                showToast('Failed to save: ' + e, 'error');
            } finally {
                hideSpinner(spinner);
            }
        },

        async handleThumbnailUpload() {
            const appId = this.editingApp;
            try {
                const selected = await pickFile({
                    filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp'] }],
                });
                if (!selected) return;

                const spinner = showSpinner(document.querySelector('.modal'));
                try {
                    const filename = await invoke('save_thumbnail', { appId, sourcePath: selected });
                    const dataUrl = await invoke('get_thumbnail_base64', { filename });
                    this.editForm.thumbnail = filename;
                    this.editForm.thumbnailDataUrl = dataUrl;
                    showToast('Thumbnail uploaded');
                } finally {
                    hideSpinner(spinner);
                }
            } catch (e) {
                showToast('Upload failed: ' + e, 'error');
            }
        },

        async toggleFavorite(appId) {
            const app = this.apps.find(a => a.id === appId);
            if (!app) return;
            const newVal = !app.favorite;
            try {
                await invoke('update_app', {
                    appId,
                    data: { favorite: newVal },
                });
                app.favorite = newVal;
                showToast(newVal ? 'Added to favorites' : 'Removed from favorites');
            } catch {
                showToast('Failed to update', 'error');
            }
        },

        async navigateToApp(appId, url) {
            try {
                await invoke('mark_used', { appId });
            } catch { /* best-effort */ }
            window.location.href = url;
        },

        async addTag() {
            const name = this.newTagName.trim();
            if (!name) return;
            try {
                await invoke('add_tag', { name });
                this.tagOrder.push(name);
                this.showAddTag = false;
                this.newTagName = '';
                showToast(`Tag "${name}" added`);
            } catch (e) {
                showToast(e || 'Failed to add tag', 'error');
            }
        },

        async deleteTag(tagName) {
            if (!await confirmAction(`Delete tag "${tagName}"? It will be removed from all apps.`)) return;
            try {
                await invoke('delete_tag', { tagName });
                const idx = this.tagOrder.indexOf(tagName);
                if (idx >= 0) this.tagOrder.splice(idx, 1);
                for (const app of this.apps) {
                    if (app.tags) {
                        const ti = app.tags.indexOf(tagName);
                        if (ti >= 0) app.tags.splice(ti, 1);
                    }
                }
                if (this.activeTag === tagName) this.activeTag = 'all';
                showToast(`Tag "${tagName}" deleted`);
            } catch (e) {
                showToast(e || 'Failed to delete tag', 'error');
            }
        },

        // ── Tag drag & drop ──

        onTagDragStart(event, idx) {
            this.tagDragIdx = idx;
            event.dataTransfer.effectAllowed = 'move';
        },

        onTagDragOver(event, idx) {
            if (this.tagDragIdx === null || this.tagDragIdx === idx) return;
            event.dataTransfer.dropEffect = 'move';
        },

        async onTagDrop(event, toIdx) {
            const fromIdx = this.tagDragIdx;
            if (fromIdx === null || fromIdx === toIdx) return;
            const item = this.tagOrder.splice(fromIdx, 1)[0];
            this.tagOrder.splice(toIdx, 0, item);
            this.tagDragIdx = null;
            try {
                await invoke('reorder_tags', { order: this.tagOrder });
            } catch { /* best-effort */ }
        },
    };
}
