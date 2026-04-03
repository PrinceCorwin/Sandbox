# SANDBOX

A personal miniapp platform for project controls, data tools, and utilities. Runs locally on `localhost` — no cloud, no auth, no fuss.

## Quick Start

```bash
python run.py
```

This creates a virtual environment (if needed), installs dependencies, and starts the server at **http://localhost:8000**.

**Windows shortcut:** Copy `start.bat.template` to `start.bat` and double-click it.

## Adding a Miniapp

Create a new folder under `apps/` with this structure:

```
apps/my_tool/
├── __init__.py        # empty
├── router.py          # FastAPI router + MINIAPP_META dict
├── logic.py           # Business logic (no FastAPI imports)
└── templates/
    └── index.html     # UI
```

The app auto-discovers it on restart. Assign tags and edit metadata from the home page.

## Tech Stack

- **Backend:** Python, FastAPI, uvicorn
- **Frontend:** HTML/CSS/JS, Alpine.js, htmx
- **Config:** `app_config.json` (git-tracked), `localStorage` for UI prefs
