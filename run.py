"""SANDBOX bootstrapper — creates venv if needed, installs deps, starts the server."""

import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.abspath(__file__))
VENV = os.path.join(ROOT, "venv")
IS_WIN = sys.platform == "win32"
PYTHON = os.path.join(VENV, "Scripts" if IS_WIN else "bin", "python")
PIP = os.path.join(VENV, "Scripts" if IS_WIN else "bin", "pip")


def ensure_venv():
    if not os.path.isdir(VENV):
        print("Creating virtual environment...")
        subprocess.run([sys.executable, "-m", "venv", VENV], check=True)

    print("Installing/updating dependencies...")
    subprocess.run(
        [PIP, "install", "-r", os.path.join(ROOT, "requirements.txt"), "--quiet"],
        check=True,
    )


def start_server():
    print("\n  SANDBOX is running at http://localhost:8000\n")
    subprocess.run(
        [PYTHON, "-m", "uvicorn", "main:app", "--host", "localhost", "--port", "8000", "--reload"],
        cwd=ROOT,
    )


if __name__ == "__main__":
    ensure_venv()
    start_server()
