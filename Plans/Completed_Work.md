# Completed Work

## 2026-04-06

- [x] Migrated entire app from Python/FastAPI to Tauri 2 (Rust + HTML/CSS/JS)
- [x] App now runs as a native desktop window — no terminal, no browser
- [x] Installed Rust toolchain on dev machine
- [x] Created Tauri project scaffold: `src-tauri/` with Cargo.toml, tauri.conf.json, capabilities
- [x] Implemented Rust backend commands: config CRUD, tag management, app discovery, thumbnail handling
- [x] Ported FW Allocation algorithm from Python/openpyxl to Rust (calamine + rust_xlsxwriter)
- [x] Migrated frontend: `src/index.html` (Alpine.js), `src/js/app.js`, `src/js/common.js`, `src/css/base.css`
- [x] Replaced Jinja2 server-side templates with client-side Alpine.js templates using Tauri `invoke()` calls
- [x] Bundled Alpine.js locally (no CDN dependency)
- [x] Created `app.json` manifest files for miniapp discovery (replaces Python `MINIAPP_META`)
- [x] Built FW Allocation miniapp frontend: file picker dialogs, run button, save-file dialogs
- [x] Created four miniapp pages: demo_work, demo_excel, demo_personal, fw_allocation
- [x] Fixed CSP issues — disabled CSP entirely (no security benefit for local desktop app)
- [x] Fixed embedded asset caching — `cargo clean` required when frontend files change
- [x] Tags now display in alphabetical order in sidebar and edit modal
- [x] Added tag delete functionality (X button on hover in sidebar, with confirmation)
- [x] Replaced flat checkbox tag selector with dropdown multi-select in edit modal
- [x] Removed all Python files: main.py, launcher.py, build.py, run.py, requirements.txt, venv/
- [x] Removed old app structure: templates/, static/, apps/*/router.py, apps/*/logic.py
- [x] Updated CLAUDE.md for Tauri architecture
- [x] Updated .gitignore for Rust/Node stack

## 2026-04-02

- [x] Created GitHub repo (PrinceCorwin/Sandbox, public)
- [x] Built app skeleton: FastAPI backend with miniapp auto-discovery
- [x] Created home page with Alpine.js: sidebar, card/list view toggle, search, sort, favorites, tag filtering, edit modal, tag drag-reorder
- [x] Dark theme with cyan/orange accents
- [x] Toast notification system (bottom-right)
- [x] Spinner and confirmation dialog utilities
- [x] Three demo miniapps (Work, Excel, Personal) with message box pages
- [x] `run.py` bootstrapper for one-command startup
- [x] App config persistence via git-tracked `app_config.json`
- [x] Thumbnail upload and persistence via `static/thumbnails/`
- [x] API endpoints: config CRUD, tag management, app metadata, reorder
- [x] CLAUDE.md with architecture and permanent instructions
- [x] README.md with quick start guide
