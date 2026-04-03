"""SANDBOX — FastAPI entry point with auto-discovery of miniapps."""

import importlib
import json
import logging
import os
import shutil
from datetime import date
from pathlib import Path

from fastapi import FastAPI, Request, UploadFile
from fastapi.responses import HTMLResponse, JSONResponse
from fastapi.staticfiles import StaticFiles
from fastapi.templating import Jinja2Templates

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("sandbox")

ROOT = Path(__file__).parent
CONFIG_PATH = ROOT / "app_config.json"
THUMBNAILS_DIR = ROOT / "static" / "thumbnails"

app = FastAPI(title="SANDBOX")
app.mount("/static", StaticFiles(directory=ROOT / "static"), name="static")
templates = Jinja2Templates(directory=ROOT / "templates")

THUMBNAILS_DIR.mkdir(parents=True, exist_ok=True)


# ── Config helpers ──────────────────────────────────────────────────────────

def load_config() -> dict:
    if CONFIG_PATH.exists():
        return json.loads(CONFIG_PATH.read_text(encoding="utf-8"))
    return {"tags": [], "tag_order": [], "apps": {}}


def save_config(config: dict):
    CONFIG_PATH.write_text(json.dumps(config, indent=2, ensure_ascii=False), encoding="utf-8")


# ── Auto-discovery ──────────────────────────────────────────────────────────

def discover_miniapps() -> dict:
    """Scan apps/ for subdirectories with router.py and register them."""
    apps_dir = ROOT / "apps"
    registry = {}

    if not apps_dir.is_dir():
        return registry

    for entry in sorted(apps_dir.iterdir()):
        router_file = entry / "router.py"
        if not entry.is_dir() or not router_file.exists():
            continue

        app_id = entry.name
        try:
            module = importlib.import_module(f"apps.{app_id}.router")
            router = getattr(module, "router", None)
            meta = getattr(module, "MINIAPP_META", {})

            if router is None:
                logger.warning("apps/%s/router.py has no 'router' object, skipping", app_id)
                continue

            app.include_router(router, prefix=f"/apps/{app_id}", tags=[app_id])
            registry[app_id] = {
                "name": meta.get("name", app_id),
                "description": meta.get("description", ""),
                "icon": meta.get("icon", ""),
            }
            logger.info("Registered miniapp: %s → /apps/%s", meta.get("name", app_id), app_id)

        except Exception:
            logger.exception("Failed to load miniapp: %s", app_id)

    return registry


discovered = discover_miniapps()


def build_app_list() -> list[dict]:
    """Merge discovered miniapps with saved config for the home page."""
    config = load_config()
    apps_config = config.get("apps", {})
    result = []

    for app_id, meta in discovered.items():
        saved = apps_config.get(app_id, {})
        result.append({
            "id": app_id,
            "title": saved.get("title", meta["name"]),
            "description": saved.get("description", meta["description"]),
            "icon": meta["icon"],
            "tags": saved.get("tags", []),
            "order": saved.get("order", 999),
            "favorite": saved.get("favorite", False),
            "thumbnail": saved.get("thumbnail"),
            "date_added": saved.get("date_added", str(date.today())),
            "last_used": saved.get("last_used"),
            "url": f"/apps/{app_id}",
        })

    result.sort(key=lambda a: a["order"])
    return result


# ── Routes ──────────────────────────────────────────────────────────────────

@app.get("/", response_class=HTMLResponse)
async def home(request: Request):
    config = load_config()
    apps_list = build_app_list()
    return templates.TemplateResponse(request, "home.html", {
        "apps": apps_list,
        "apps_json": json.dumps(apps_list),
        "tags": config.get("tags", []),
        "tags_json": json.dumps(config.get("tag_order", config.get("tags", []))),
    })


@app.get("/api/config")
async def get_config():
    return load_config()


@app.put("/api/config")
async def update_config(request: Request):
    data = await request.json()
    config = load_config()
    config.update(data)
    save_config(config)
    return {"status": "ok"}


@app.patch("/api/apps/{app_id}")
async def update_app(app_id: str, request: Request):
    data = await request.json()
    config = load_config()
    if "apps" not in config:
        config["apps"] = {}
    if app_id not in config["apps"]:
        config["apps"][app_id] = {}
    config["apps"][app_id].update(data)
    save_config(config)
    return {"status": "ok"}


@app.post("/api/apps/{app_id}/thumbnail")
async def upload_thumbnail(app_id: str, file: UploadFile):
    ext = Path(file.filename).suffix or ".png"
    filename = f"{app_id}{ext}"
    dest = THUMBNAILS_DIR / filename

    with open(dest, "wb") as f:
        shutil.copyfileobj(file.file, f)

    thumbnail_url = f"/static/thumbnails/{filename}"

    config = load_config()
    if "apps" not in config:
        config["apps"] = {}
    if app_id not in config["apps"]:
        config["apps"][app_id] = {}
    config["apps"][app_id]["thumbnail"] = thumbnail_url
    save_config(config)

    return {"status": "ok", "thumbnail": thumbnail_url}


@app.post("/api/apps/{app_id}/use")
async def mark_used(app_id: str):
    config = load_config()
    if "apps" not in config:
        config["apps"] = {}
    if app_id not in config["apps"]:
        config["apps"][app_id] = {}
    config["apps"][app_id]["last_used"] = str(date.today())
    save_config(config)
    return {"status": "ok"}


@app.post("/api/tags")
async def add_tag(request: Request):
    data = await request.json()
    tag_name = data.get("name", "").strip()
    if not tag_name:
        return JSONResponse({"error": "Tag name required"}, status_code=400)

    config = load_config()
    tags = config.get("tags", [])
    tag_order = config.get("tag_order", list(tags))

    if tag_name in tags:
        return JSONResponse({"error": "Tag already exists"}, status_code=400)

    tags.append(tag_name)
    tag_order.append(tag_name)
    config["tags"] = tags
    config["tag_order"] = tag_order
    save_config(config)
    return {"status": "ok", "tags": tags}


@app.delete("/api/tags/{tag_name}")
async def delete_tag(tag_name: str):
    config = load_config()
    tags = config.get("tags", [])
    tag_order = config.get("tag_order", list(tags))

    if tag_name not in tags:
        return JSONResponse({"error": "Tag not found"}, status_code=404)

    tags.remove(tag_name)
    if tag_name in tag_order:
        tag_order.remove(tag_name)

    for app_data in config.get("apps", {}).values():
        app_tags = app_data.get("tags", [])
        if tag_name in app_tags:
            app_tags.remove(tag_name)

    config["tags"] = tags
    config["tag_order"] = tag_order
    save_config(config)
    return {"status": "ok", "tags": tags}


@app.put("/api/tags/reorder")
async def reorder_tags(request: Request):
    data = await request.json()
    new_order = data.get("order", [])
    config = load_config()
    config["tag_order"] = new_order
    save_config(config)
    return {"status": "ok"}


@app.put("/api/apps/reorder")
async def reorder_apps(request: Request):
    data = await request.json()
    order_list = data.get("order", [])
    config = load_config()
    for idx, app_id in enumerate(order_list):
        if app_id in config.get("apps", {}):
            config["apps"][app_id]["order"] = idx
    save_config(config)
    return {"status": "ok"}
