# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Naming Conventions
- **App name:** Sandbox
- **Company name:** Summit Industrial — this is the ONLY correct name for the company
  - NEVER use "Summit Constructors", "Summit Industrial Constructors", or any other variation

## Build & Run

**Prerequisites:** Rust toolchain (`rustup`), Node.js, Visual Studio Build Tools (MSVC)

```bash
# Development (opens app window with hot reload)
npx tauri dev

# Production build (creates installer)
npx tauri build
```

**Important:** When running `cargo build` from Git Bash, MSVC's `link.exe` must be on PATH before Git's:
```bash
MSVC_LINK=$(find "/c/Program Files/Microsoft Visual Studio/2022/Community/VC/Tools/MSVC" -name "link.exe" -path "*/x64/*" | head -1)
export PATH="$(dirname "$MSVC_LINK"):$HOME/.cargo/bin:$PATH"
```

**Frontend changes require `cargo build`** because Tauri embeds HTML/CSS/JS at compile time. After changing frontend files, you MUST rebuild — the running binary serves baked-in assets, not live files.

## Architecture

- **Shell:** Tauri 2 (Rust) — native desktop window via WebView2, no terminal, no browser
- **Backend:** Rust commands in `src-tauri/src/commands/` — config CRUD, file I/O, Excel processing
- **Frontend:** HTML/CSS/JS with Alpine.js in `src/` — same dark theme, no build step
- **Config:** `app_config.json` stored in `%APPDATA%/com.princecorwin.sandbox/` at runtime. Default bundled as resource.
- **Thumbnails:** Stored in `%APPDATA%/com.princecorwin.sandbox/thumbnails/`, served as base64 data URLs
- **CSP:** Disabled (`"csp": null` in tauri.conf.json) — this is a local desktop app, CSP adds no security value here

## Project Structure

```
Sandbox/
├── src/                          # Frontend
│   ├── index.html                # Home page (Alpine.js app)
│   ├── css/base.css              # Dark theme styles
│   ├── js/
│   │   ├── alpine.min.js         # Alpine.js (bundled locally, no CDN)
│   │   ├── common.js             # Utilities (invoke, toast, spinner, dialog)
│   │   └── app.js                # Alpine sandbox() component
│   └── apps/                     # Miniapps
│       ├── {app_id}/
│       │   ├── app.json          # Metadata (name, description, icon)
│       │   └── index.html        # App page
│       └── fw_allocation/        # First real miniapp
│           ├── app.json
│           ├── index.html
│           └── fw_allocation.js
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── lib.rs                # Plugin/command registration
│       └── commands/
│           ├── config.rs         # Config read/write
│           ├── discovery.rs      # Scan apps/ for app.json
│           ├── thumbnails.rs     # Thumbnail save/serve
│           └── fw_allocation.rs  # Excel processing
├── app_config.json               # Default config (bundled as resource)
├── package.json
└── CLAUDE.md
```

## Miniapp Structure

Each miniapp is a folder under `src/apps/` with:
- `app.json` — Metadata: `{ "id", "name", "description", "icon" }`
- `index.html` — Self-contained HTML page. Links to `../../css/base.css` and `../../js/common.js`.
- Optional JS files for app-specific logic

Miniapps must not depend on each other. Adding/removing a folder under `src/apps/` should not affect other miniapps.

**Backend logic** for miniapps that need it (e.g., Excel processing) goes in `src-tauri/src/commands/` as Rust Tauri commands. Frontend calls them via `invoke('command_name', { args })`.

## Development Approach
- ONE change at a time, test before proceeding
- No quick fixes — proper architectural solutions
- Delete/refactor legacy code when no longer relevant
- After completing a feature: identify improvements, check for dead code, suggest refactoring
- ALWAYS rebuild and test after changes before reporting completion

**See also:** `Plans/Project_Status.md` (todos, backlog), `Plans/Completed_Work.md` (changelog)

### Help Manual Maintenance
- When features are added, deleted, or modified, update `Help/manual.html` to keep documentation current
- Add new features to the appropriate section and update the Table of Contents if adding new sections
- Remove documentation for deleted features
- Update existing documentation when feature behavior changes

## Permanent Instructions

**Always show loading feedback.** Use `showSpinner()` / `hideSpinner()` from `common.js` for any async operation. Use `showToast()` for success/error feedback. The user must never wonder if something is happening.

**Always confirm destructive actions.** Use `confirmAction()` from `common.js` before any delete, remove, or irreversible operation. No exceptions unless explicitly told otherwise.

**Toast notifications** go bottom-right via `#toast-container`. Three types: `success`, `error`, `info`.

**Tauri invoke pattern:** All backend calls use `invoke('command_name', { arg1, arg2 })` from `window.__TAURI__.core` (exposed globally via `withGlobalTauri: true`).

## Git Commits
- **NEVER commit without explicit user permission** — user needs to test changes first
  - "Update X" or "Add Y" means make the change, NOT commit it
  - Wait for explicit "commit" instruction before running git commit
- ALWAYS push to remote after committing, unless user instructs otherwise
- **ALWAYS commit ALL uncommitted changes** when user says "commit":
  - Use `git add -A` to stage EVERYTHING — no exceptions
  - NEVER selectively choose files based on what you worked on
  - NEVER skip files because they were "already modified" or "unrelated to current work"
  - If a file shows up in `git status`, it gets committed. Period.
  - The ONLY exception is if user explicitly says "commit only X" or "don't commit Y"
- Do NOT add "Generated with Claude" or "Co-Authored-By: Claude" to commit messages
- Do NOT add AI attribution comments in code
- Write clear, concise commit messages describing the change
- Watch for `nul` file in git status — this is a Windows artifact that gets created accidentally. Delete it immediately with `rm -f nul` when spotted.

### MANDATORY Pre-Commit Checklist
**When user says "commit", invoke the `/finisher` skill.** If the skill file is unavailable, follow these steps manually:

1. Update `Plans/Project_Status.md` — Remove completed items from the backlog
2. Update `Plans/Completed_Work.md` — Add entry describing what was completed (with date header)
3. **HELP MANUAL CHECK (frequently missed!):**
   - Ask: "Did this work change anything a user would see or interact with?"
   - If YES: Update `Help/manual.html` (add/update/remove sections, update TOC if needed)
   - If NO: Confirm why not (e.g., "internal refactor only, no UI change")
4. Update any other relevant plan docs if the work relates to a specific feature plan
5. ONLY THEN proceed with `git add -A` and `git commit`

**All paths are relative to the repository root. NEVER use absolute paths.**
**This is NOT optional. Failure to update status docs before committing is a workflow violation.**

### Status Doc Timing
- **Do NOT update Project_Status.md or Completed_Work.md until user confirms the work is tested and complete**
- Making code changes does not mean the work is done — user must test first
- Only move items to Completed_Work.md after explicit user confirmation that testing passed

### Plan File Management
- Plan files in the `Plans/` folder should be deleted once fully implemented
- Before deleting: ensure Project_Status.md and any related docs are updated
- **ALWAYS ask the user before deleting plan files, even if accept edits is enabled**

## Code Standards

- Rust: Tauri commands use `#[tauri::command]`, serde for JSON, `AppHandle` for path resolution
- Frontend: vanilla JS + Alpine.js. No React/Vue/build tools.
- Excel: `calamine` crate (read) + `rust_xlsxwriter` crate (write). No Python/openpyxl.
- Logic functions return structured results with issues/errors alongside primary output — don't panic for expected failures.
- Comments only where logic is non-obvious.

## What Not to Do

- No cloud deployment, auth, or multi-user features unless explicitly asked.
- No ORMs, migration frameworks, or task queues. SQLite via `rusqlite` if needed.
- No shared modules coupling miniapps together.
- No packages requiring system-level or admin installs.
- Never commit `node_modules/`, `src-tauri/target/`, `venv/`, `data/`, or `start.bat`.

## Communication Preferences
- Be direct, skip pleasantries
- State what to do and why
- Challenge assumptions if there's a better approach
- Present code one block at a time, wait for confirmation
- When in plan mode, do NOT call ExitPlanMode until user explicitly agrees to the plan
