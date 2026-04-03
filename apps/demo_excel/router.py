from fastapi import APIRouter, Request
from fastapi.responses import HTMLResponse
from fastapi.templating import Jinja2Templates
import os

router = APIRouter()
templates = Jinja2Templates(directory=os.path.join(os.path.dirname(__file__), "templates"))

MINIAPP_META = {
    "name": "Demo Excel App",
    "description": "A demo miniapp for the Excel category",
    "icon": "\U0001f4ca",
}


@router.get("/", response_class=HTMLResponse)
async def index(request: Request):
    return templates.TemplateResponse(request, "index.html", {
        "app_name": MINIAPP_META["name"],
        "message": "This is the first Excel app",
    })
