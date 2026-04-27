# Sandbox — Design Decisions

Permanent record of architectural choices, design rationale, and
implementation decisions. Consult this when asking "why did we do X?" before
changing existing behavior.

Format for each entry:

### Title
**Decision:** What was chosen.
**Why:** The reasoning, including alternatives considered and rejected.
**Date:** Month YYYY.

---

### Backend `fs_helpers` module for save-as and reveal-in-Explorer
**Decision:** All file-system operations that touch user-picked paths
(Save As dialogs, Reveal in Explorer) go through a shared Rust module
`src-tauri/src/commands/fs_helpers.rs` exposing `app_copy_file`,
`app_copy_tree`, and `app_open_path`. Frontend miniapps invoke these by
name rather than calling `window.__TAURI__.fs.copyFile` /
`tauri-plugin-shell` directly.
**Why:** Two Tauri 2 issues hit during DataLore build:
(1) `tauri-plugin-fs` registers a `copy_file` command of its own — naming
a custom Rust command `copy_file` causes the invoke router to match the
plugin first, demand `fs:allow-copy-file`, and never reach the custom
handler. The `app_` prefix avoids the name collision.
(2) The default `fs:default` scope doesn't grant write access to
user-picked locations like Downloads or Desktop, so the JS `copyFile`
fails on Save As. A backend Rust command running `std::fs::copy` uses
the process's full user permissions and sidesteps the plugin scope
entirely. Alternative considered: widen `fs:default` scope in
`capabilities/default.json`. Rejected because backend helpers are
narrower and don't expose unneeded paths to the JS layer.
**Date:** April 2026.

---

### DataLore is SQLite-only in v1
**Decision:** DataLore reads `.db` / `.sqlite` / `.sqlite3` files only.
No Azure SQL, Postgres, MySQL, or BACPAC support.
**Why:** Azure SQL is a managed cloud service with no importable
single-file format. The closest equivalent (BACPAC) is a Microsoft-only
zip of XML schema + BCP-format data; no maintained Rust crate exists for
parsing it. Live `tiberius` connections are possible but require
connection-string handling, credential storage, and an entirely
different UX (form vs file picker). Deferred to v2 if/when a real need
arises. SQLite alone covers the immediate use cases (local databases
from other tools, exported snapshots).
**Date:** April 2026.

---

### Error-toast duration extended to 8s + click-to-dismiss
**Decision:** `showToast` defaults error toasts to 8 seconds (up from
3s); success/info stay at 3s. Any toast can be dismissed by clicking it.
**Why:** Tauri permission-denied errors are long, multi-line strings.
At 3s they slid off-screen before they could be read, making a known
class of bugs (fs scope, plugin name collisions) impossible to diagnose
without DevTools. 8s is enough to read the message; click-to-dismiss
prevents the longer error from blocking the UI when it's already been
understood.
**Date:** April 2026.
