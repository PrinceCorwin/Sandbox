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

### Supabase free tier as backend for SQL-based miniapps
**Decision:** Any Sandbox miniapp that needs SQL/tabular data with
cross-device sync will use a single Supabase free-tier project
(Postgres + auto REST/GraphQL + Auth + Realtime). Multiple miniapps
share the project via Postgres schemas or table prefixes.
Out of scope: iList (stays on Firebase), VANTAGE production data (stays
on company Azure), self-hosted Supabase on AmalfiNAS.
**Why:** Long evaluation rejected several alternatives:
- **SQL Server on AmalfiNAS** — Sandbox runs on the work laptop;
  reaching the NAS would require Tailscale (visible to Comfort Systems
  USA umbrella tooling) or Cloudflare Tunnel (adds external attack
  surface and a daemon on the work machine).
- **Personal Azure** — cost-creep risk and another paid account to
  manage.
- **Synology Drive folder-sync of SQLite** — sync clients don't honour
  DB file locks, leading to corruption.
- **Self-hosting Supabase on the NAS** — possible but defeats the point
  of picking a BaaS.
Supabase wins on: hard-capped free tier (project suspends, no surprise
bills), real Postgres, generated APIs (no custom backend code for CRUD),
HTTPS-only traffic that works from any network without VPN, and RLS
giving per-user row scoping at the DB layer. Full context in
`Plans/sandbox-supabase-plan.md` — don't re-litigate without new info.
**Date:** May 2026.

---

### Sandbox stays on Tauri (re-evaluated vs WPF + WebView2)
**Decision:** Sandbox remains on the Tauri 2 (Rust + WebView2) stack.
Not migrating to WPF or any other shell.
**Why:** User hit real Windows toolchain friction on a fresh machine —
no Rust, no MSVC C++ Build Tools, and Git's `link.exe` (coreutils)
shadowing MSVC's linker on PATH. Triggered a stack re-evaluation. The
honest comparison: WPF's "one-VS-installer-and-it-works" ergonomics are
genuinely better than Tauri's "Node + Rust + MSVC C++ Build Tools, all
separate installs" setup. But every Sandbox miniapp to date is
HTML/JS/Alpine (FW Allocation, DataLore, Photography Tip Cards) and is
staying that way. Tauri's value prop is exactly "drop HTML into
`src/apps/` and it shows up." WPF + WebView2 would re-implement the
same shell with more C# glue and a full port of `discovery.rs` /
Excel / SQLite commands. Worth the one-time MSVC install; not worth a
rewrite. Revisit only if miniapps shift to native UI (forms, charts,
native dialogs) rather than HTML.
**Date:** May 2026.

---

### Miniapp asset filenames must be space-free
**Decision:** All static assets shipped with a miniapp (images, fonts,
data files) must use lowercase, space-free filenames (underscores or
hyphens). Both the file on disk and any reference in HTML/JS/`app.json`
follow the same rule.
**Why:** Tauri 2's WebView2 protocol handler refuses to load assets
whose filenames contain spaces, even when the `<img src>` is URL-encoded
(`%20`). Discovered while importing the Photography Tip Cards miniapp:
44 JPGs originally named `The Exposure Triangle.jpg`-style all failed
to load; only the 3 files without spaces rendered. URL-encoding the
references didn't help; renaming the files (and updating references)
did. The asset protocol's behaviour here is undocumented but consistent,
so the safest rule is to never ship a space-containing filename in the
first place.
**Date:** May 2026.

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
