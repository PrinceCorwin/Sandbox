<p align="center">
  <img src="Assets/LogoImages/newSandboxLogo128.png" alt="Sandbox" width="128" height="128">
</p>

# SANDBOX

A personal miniapp platform for project controls, data tools, and utilities. Runs as a native Windows desktop app — no cloud, no auth, no browser.

## Running the App

After installation, launch **Sandbox** from the Start menu or desktop shortcut. The installer lives at:

```
src-tauri/target/release/bundle/nsis/Sandbox_<version>_x64-setup.exe
```

Run that once to install. Or, for a portable copy, use the standalone executable:

```
src-tauri/target/release/sandbox.exe
```

## Building from Source

### Prerequisites (one-time per machine)

- **Node.js** (LTS) — https://nodejs.org
- **Rust** via rustup — https://win.rustup.rs
- **Visual Studio** (Community edition is fine) with the **"Desktop development with C++"** workload. This provides MSVC, Windows SDK, and `vcvarsall.bat` — all required by the Rust MSVC toolchain.

### Build

```bash
npx tauri build
```

Produces both the standalone `.exe` and the installer described above.

For development with hot reload:

```bash
npx tauri dev
```

### Windows / Git Bash gotcha

When building from Git Bash, MSVC's `link.exe` must resolve before Git's built-in `link.exe`. A helper batch file (`build_once.bat`) handles this by sourcing `vcvars64.bat` and ordering `PATH` correctly. Double-click `build_once.bat` (or run it from cmd) if `npx tauri build` from bash hits `kernel32.lib` or `link.exe` errors.

## Adding a Miniapp

Create a new folder under `src/apps/` with this structure:

```
src/apps/my_tool/
├── app.json            # { "id", "name", "description", "icon" }
├── index.html          # Self-contained UI (links ../../css/base.css, ../../js/common.js)
└── my_tool.js          # (optional) app-specific JS
```

If the miniapp needs backend logic (file I/O, Excel processing, etc.), add a Rust Tauri command in `src-tauri/src/commands/` and call it from the frontend via `invoke('command_name', { args })`.

Miniapps are auto-discovered at app launch. Each is self-contained — adding or removing one must not affect others.

## Tech Stack

- **Shell:** Tauri 2 (Rust) — native desktop window via WebView2
- **Backend:** Rust commands in `src-tauri/src/commands/`
- **Frontend:** HTML/CSS/JS with Alpine.js in `src/` (no build step)
- **Config:** `app_config.json` stored in `%APPDATA%/com.princecorwin.sandbox/` at runtime
- **Excel:** `calamine` (read) + `rust_xlsxwriter` (write)
