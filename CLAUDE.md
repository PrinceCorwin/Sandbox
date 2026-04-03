# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run

```bash
python run.py
```

Creates venv if needed, installs deps, starts uvicorn at `http://localhost:8000` with `--reload`.

## Architecture

- **Backend:** Python + FastAPI (`main.py`). Miniapps auto-discovered from `apps/` subdirectories.
- **Frontend:** HTML/CSS/JS with Alpine.js (client-side reactivity) and htmx (server-driven UI). No build step.
- **Config:** `app_config.json` (git-tracked) stores app metadata, tags, and order. `localStorage` stores per-machine UI preferences.
- **Thumbnails:** Stored in `static/thumbnails/` (git-tracked). Uploaded via edit modal.

## Miniapp Structure

Each miniapp is a folder under `apps/` with:
- `router.py` — FastAPI `APIRouter` + `MINIAPP_META` dict (name, description, icon). Glue only, no business logic.
- `logic.py` — Pure Python business logic. No FastAPI imports. Accepts/returns plain Python types.
- `templates/index.html` — Self-contained HTML page. Links to `/static/css/base.css` and `/static/js/common.js`.

Miniapps must not depend on each other. Adding/removing a folder under `apps/` should not affect other miniapps.

## Permanent Instructions

**Always show loading feedback.** Use `showSpinner()` / `hideSpinner()` from `common.js` for any async operation. Use `showToast()` for success/error feedback. The user must never wonder if something is happening.

**Always confirm destructive actions.** Use `confirmAction()` from `common.js` before any delete, remove, or irreversible operation. No exceptions unless explicitly told otherwise.

**Toast notifications** go bottom-right via `#toast-container`. Three types: `success`, `error`, `info`.

**Update project plans before every commit.** Before committing, update `plans/ProjectStatus.md` (current/future work) and `plans/CompletedWork.md` (finished items). Move completed items from ProjectStatus to CompletedWork. On the 1st of each month, archive CompletedWork to `plans/archive/CompletedWork-YYYY-MM-DD.md` and start a fresh CompletedWork file.

## Code Standards

- Python: type hints on function signatures, `logging` module (not print), functions over classes unless complexity warrants it.
- Frontend: vanilla JS + Alpine.js + htmx. No React/Vue/npm/build tools.
- Logic functions return structured results with issues/errors alongside primary output — don't raise exceptions for expected failures.
- Comments only where logic is non-obvious.
- Do not use pandas for Excel work — use openpyxl.

## What Not to Do

- No cloud deployment, auth, or multi-user features unless explicitly asked.
- No ORMs, migration frameworks, or task queues. SQLite via stdlib `sqlite3` directly.
- No shared Python modules coupling miniapps together.
- No packages requiring system-level or admin installs.
- Never commit `venv/`, `data/`, or `start.bat`.
