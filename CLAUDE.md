# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Naming Conventions
- **App name:** Sandbox
- **Company name:** Summit Industrial — this is the ONLY correct name for the company
  - NEVER use "Summit Constructors", "Summit Industrial Constructors", or any other variation

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

## Development Approach
- ONE change at a time, test before proceeding
- No quick fixes — proper architectural solutions
- Delete/refactor legacy code when no longer relevant
- After completing a feature: identify improvements, check for dead code, suggest refactoring
- ALWAYS run `python run.py` after changes and verify before reporting completion

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

## Communication Preferences
- Be direct, skip pleasantries
- State what to do and why
- Challenge assumptions if there's a better approach
- Present code one block at a time, wait for confirmation
- When in plan mode, do NOT call ExitPlanMode until user explicitly agrees to the plan
