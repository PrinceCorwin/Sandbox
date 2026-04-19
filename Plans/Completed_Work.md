# Completed Work

## 2026-04-19

- [x] Wired up custom app icon: rounded black-background version embedded in sandbox.exe (drives taskbar/window/installer); transparent version used for in-app sidebar logo (40px) and favicon
- [x] Generated full Tauri icon set via `npx tauri icon` from 1024x1024 source
- [x] Added Help nav item to sidebar (above Settings) that opens the manual in an iframe modal
- [x] Moved `Help/manual.html` to `src/help/manual.html` so it bundles with the frontend and ships inside the app
- [x] Expanded FW Allocation section of the manual with exact column specs (case-sensitive header names, required values), step-by-step walkthrough, allocation logic, and issues log interpretation
- [x] Added logo header to manual top
- [x] Added shipped-with-miniapp default thumbnail support: new optional `thumbnail` field in `app.json`, discovery.rs exposes `default_thumbnail` URL, frontend falls back to it when no user thumbnail is set
- [x] Wired up green pipes FW Allocation logo as the FW miniapp's shipped thumbnail
- [x] Removed demo miniapps (`demo_excel`, `demo_personal`, `demo_work`): deleted folders and cleared entries from `app_config.json`
- [x] Rewrote README.md for the Tauri stack (removed old Python quick-start), added centered logo banner
- [x] Created `build_once.bat` helper that sources vcvars64.bat and prepends MSVC link.exe on PATH so `npx tauri build` works from Git Bash without hitting Git's `link.exe`
- [x] Updated CLAUDE.md references from `Help/manual.html` to `src/help/manual.html`
- [x] Built production installer and standalone exe — first full Tauri build on this machine after installing Node/Rust/VS C++ workload

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
