# PRD: Port Sandbox to Web (`princecorwinstudios.com/sandbox`)

**Author:** Steve Legion (via Claude) — handoff document
**Audience:** Claude instance working in the `princecorwinstudios.com` project folder
**Status:** Draft, awaiting implementation
**Date:** 2026-05-31

---

## TL;DR

Port the existing Sandbox desktop app (Tauri + Rust + HTML/JS/Alpine) to a
static web app deployed at `https://princecorwinstudios.com/sandbox/`. The
parent site at `princecorwinstudios.com/` is for something else and stays
public. The `/sandbox/*` subtree must require login (Cloudflare Access).

All three current miniapps (`fw_allocation`, `datalore`, `photog_tips`) are
convertible to pure client-side browser apps with no backend. The Rust
backend logic is replaced by JS libraries: SheetJS for Excel, sql.js for
SQLite. Per-user state moves from `%APPDATA%/app_config.json` to browser
`localStorage`.

Deployment target: **Cloudflare Pages** (free tier; user already owns the
domain on Cloudflare DNS).

---

## Source Repository

The existing implementation is the reference truth for behavior, UI, and
miniapp logic. Do not rebuild from scratch — port.

- **GitHub:** https://github.com/PrinceCorwin/Sandbox
- **Branch:** `main`
- **License:** No public license file. Personal project; user is the
  copyright holder.
- **Clone command:** `git clone https://github.com/PrinceCorwin/Sandbox.git
  /tmp/sandbox-source` (or wherever convenient — read-only, do not push
  changes back).

### Key files to read before starting

Read these in order:

1. `CLAUDE.md` — overall project conventions and architecture notes
2. `Plans/Project_Status.md` — current state and pending decisions
3. `Plans/Decisions.md` — past architectural choices and why
4. `src/index.html` and `src/js/app.js` — home page Alpine.js component
5. `src/css/base.css` — full dark theme + component styles to reuse verbatim
6. `src/js/common.js` — `invoke`, `showToast`, `showSpinner`, `confirmAction`
   helpers (web version needs equivalents)
7. `src-tauri/src/commands/discovery.rs` — how miniapps are enumerated
8. `src-tauri/src/commands/config.rs` — config CRUD shape
9. `src-tauri/src/commands/fw_allocation.rs` — Excel allocation algorithm
   (port this logic to JS)
10. `src-tauri/src/commands/datalore.rs` — SQLite-to-Excel/CSV export logic
11. `src/apps/fw_allocation/`, `src/apps/datalore/`, `src/apps/photog_tips/`
    — each miniapp's frontend
12. `src/help/manual.html` — user-facing help docs to mirror

---

## Goals

1. Single URL on every device — `https://princecorwinstudios.com/sandbox/`
2. Auth required (Cloudflare Access scoped to `/sandbox/*`)
3. All three current miniapps working end-to-end in browser
4. Mobile-responsive (phone/tablet usable)
5. State persists per-browser via `localStorage`
6. Zero ongoing cost (Cloudflare Pages free tier; Access free tier ≤50 users)
7. Deploy on git push (Cloudflare Pages → GitHub integration)

## Non-Goals

- Cross-device state sync (would need a backend; out of scope for v1)
- Replacing the Tauri desktop app immediately (decision deferred; web can
  coexist or replace later)
- Public access (the page is gated)
- Native file-system save dialogs (browser downloads only)
- Offline-first / PWA installability (defer to v2 if requested)

---

## Hosting & Auth Architecture

### Cloudflare Pages
- Project name: `princecorwinstudios` (or whatever matches the parent
  site's existing Pages project — if the parent isn't yet on Pages, create
  one project for the whole site; serve `/sandbox` as a subdirectory)
- Custom domain: `princecorwinstudios.com`
- Source: GitHub repo (use the parent site's existing repo if it exists;
  otherwise create one)
- Build output directory: whatever the parent site uses; place sandbox
  assets under `/sandbox/`
- Free tier limits: unlimited bandwidth, 500 builds/month, 100 custom
  domains. Comfortable headroom.

### Cloudflare Access (Zero Trust)
- Free tier covers up to 50 seats — more than enough for personal use
- In Cloudflare Zero Trust dashboard → Access → Applications → Add an
  Application → Self-hosted
  - Application name: `Sandbox`
  - Subdomain: `princecorwinstudios.com`
  - Path: `/sandbox*` (the trailing wildcard matters — `/sandbox` and
    `/sandbox/...` both protected)
  - Session duration: 24h or longer (personal pref)
- Add a policy:
  - Action: Allow
  - Include: Emails → `<user's email>` (the GitHub-registered email is
    `amalfi615@gmail.com` per memory)
- Identity provider: Google OAuth is simplest (no extra setup). GitHub
  also works since the user is already on GitHub.

The parent site stays unprotected. Access policies are path-scoped, so
the `/` route serves freely while `/sandbox*` requires the cookie.

---

## Tech Stack

Match the existing Sandbox's no-build-step ethos. Vanilla web tech, no
React/Vue/bundlers.

- HTML5 + CSS3 + vanilla JS
- **Alpine.js** for component logic (already used in Sandbox — bundle the
  same `src/js/alpine.min.js` file locally, no CDN)
- **SheetJS Community Edition** (Apache 2.0) for `.xlsx` read/write —
  load via `<script>` tag from local copy or `cdn.sheetjs.com` if CDN
  use is acceptable. Library docs: https://docs.sheetjs.com
- **sql.js** (MIT) for SQLite reads in the browser. Library docs:
  https://sql.js.org. Ships a WASM file ~1 MB plus a JS wrapper. Bundle
  locally.
- **FileSaver.js** (MIT) for triggering browser downloads with a chosen
  filename. Optional — modern browsers support `<a download>` directly.
- No backend, no API, no database. Everything runs in the user's
  browser tab.

---

## Routing & File Layout

Suggested layout inside the parent project (adjust to match parent's
conventions):

```
<parent-project-root>/
├── ... (parent site files at /)
└── sandbox/
    ├── index.html              ← Sandbox home page
    ├── css/
    │   └── base.css            ← copied verbatim from Sandbox repo
    ├── js/
    │   ├── alpine.min.js       ← from Sandbox repo
    │   ├── common.js           ← rewritten for web (see below)
    │   └── app.js              ← home page Alpine component
    ├── vendor/
    │   ├── sheetjs/xlsx.full.min.js
    │   └── sqljs/sql-wasm.js
    │   └── sqljs/sql-wasm.wasm
    ├── help/
    │   └── manual.html         ← from Sandbox repo, light edits
    └── apps/
        ├── photog_tips/
        │   ├── index.html      ← from Sandbox repo, drop in
        │   └── images/         ← all 44 JPGs
        ├── fw_allocation/
        │   ├── index.html      ← rewritten — drop Tauri invokes
        │   └── fw_allocation.js
        └── datalore/
            ├── index.html      ← rewritten — drop Tauri invokes
            └── datalore.js
```

URL routing under the parent site:
- `/sandbox` → home page
- `/sandbox/apps/photog_tips/` → photog_tips miniapp
- `/sandbox/help/manual.html` → in-app help

---

## Per-Miniapp Migration

### `photog_tips` — trivial port (~1 hour)

Pure static. 44 JPGs + one HTML page with embedded CSS/JS. No backend
calls.

- Copy `src/apps/photog_tips/index.html` and `src/apps/photog_tips/images/`
  verbatim
- No JS changes needed
- Verify image paths still resolve (relative paths should be fine; the
  Tauri WebView2 quirk that required lowercase filenames doesn't apply
  to browsers, but the filenames are already lowercase so leave them)

### `fw_allocation` — JS port (~1–2 days)

The Rust backend in `src-tauri/src/commands/fw_allocation.rs` does:
1. Read two `.xlsx` files (FW count pivot + labor file)
2. Allocate field weld counts to BW/SW labor rows by drawing, sorted by
   pipe size then footage
3. Emit a modified labor `.xlsx` + an issues log `.xlsx`

Replacement (browser):
1. Two `<input type="file" accept=".xlsx">` pickers
2. Parse both files with SheetJS (`XLSX.read`)
3. Port the allocation algorithm from Rust to JS — algorithm is described
   in `src/help/manual.html` under the "FW Allocation > Allocation logic"
   section. Read both the Rust source AND the manual section.
4. Write outputs with SheetJS (`XLSX.write`)
5. Trigger downloads with `<a download="FW_Updated.xlsx" href="data:...">`
   or FileSaver

The current frontend at `src/apps/fw_allocation/` shows the UI pattern:
file pickers, "Run Allocation" button, results panel with summary stats,
two download buttons. Replicate that UI in the web version.

### `datalore` — JS port (~1–2 days)

The Rust backend in `src-tauri/src/commands/datalore.rs` does:
1. Read a SQLite file via `rusqlite`
2. Iterate every user table (skip `sqlite_*` system tables)
3. Emit Excel (one sheet per table), CSV (one file per table in a `csv/`
   subfolder), or both
4. Save outputs to disk

Replacement (browser):
1. `<input type="file" accept=".db,.sqlite,.sqlite3">` picker
2. Load the file as ArrayBuffer, pass to sql.js: `new SQL.Database(bytes)`
3. List tables: `SELECT name FROM sqlite_master WHERE type='table' AND
   name NOT LIKE 'sqlite_%'`
4. For each table, `SELECT *` and convert to a SheetJS sheet (Excel) or
   CSV string
5. For Excel: build one workbook with multiple sheets via SheetJS, trigger
   download
6. For CSV: build a zip via `JSZip` (MIT) containing one .csv per table,
   trigger download
7. For "Both": trigger two separate downloads, or zip everything together

Type-handling quirks from the Rust version (preserve these):
- BLOB columns are not extracted — show `<BLOB N bytes>` placeholder
- Excel sheet names sanitized: strip `/ \ ? * [ ] :`, truncate to 31 chars,
  numeric suffix on collision
- Excel row cap at 1,048,576 — flag truncation in the UI
- NULL → empty cell

Skip the "Reveal in Run Folder" feature — irrelevant in a browser.
Skip the per-run staging-folder concept — generate downloads directly.

---

## Home Page

The Sandbox home is an Alpine.js component (`src/index.html` +
`src/js/app.js`). It provides:

- Sidebar with tag filter list (All / Favorites / Uncategorized / per-tag)
- Top bar: search input, card/list view toggle, sort dropdown
- Main: cards or list rows for each miniapp
- Per-app: title, description, icon emoji, thumbnail, favorite star,
  edit pencil → modal
- Edit modal: title, description, tag checkboxes, thumbnail picker
- Add Tag button in sidebar; delete-tag X on hover
- Drag-to-reorder cards (custom order persisted)
- "Sort by: Custom Order / Alphabetical / Recently Used / Date Added"

Port all of this. The data structures are in `src-tauri/src/commands/`
(`config.rs`, `discovery.rs`); the storage shape is described in the
**State Model** section below.

The current home page reads miniapp metadata by:
1. Calling Rust `discover_apps()` which scans `src/apps/*/app.json`
2. Merging with per-app user state from `app_config.json`

For the web port, replace step 1 with a build-time or load-time static
manifest:

```javascript
// apps/manifest.js — generated by hand or by a small build script
window.SANDBOX_APPS = [
  {id: "photog_tips", name: "Photography Tip Cards", description: "...",
   icon: "📷", thumbnail: "apps/photog_tips/images/the_exposure_triangle.jpg"},
  {id: "fw_allocation", name: "FW Allocation", description: "...",
   icon: "🔧", thumbnail: "apps/fw_allocation/logo.png"},
  {id: "datalore", name: "DataLore", description: "...",
   icon: "📊", thumbnail: null},
]
```

Replace `discover_apps()` calls in `app.js` with `window.SANDBOX_APPS`.

---

## State Model — `localStorage`

Replaces `%APPDATA%/com.princecorwin.sandbox/app_config.json`.

Storage key: `sandbox:config`
Storage value: JSON serialization of:

```json
{
  "tags": ["Work", "Personal"],
  "tag_order": ["Work", "Personal"],
  "apps": {
    "fw_allocation": {
      "tags": ["Work"],
      "favorite": false,
      "order": 1,
      "last_used": "2026-05-31",
      "date_added": "2026-04-15",
      "title": "FW Allocation",
      "description": "..."
    }
  }
}
```

This is the exact shape the existing `config.rs` reads and writes.
Keeping it identical means the home page Alpine code ports nearly
verbatim.

Skip the `thumbnail` field — browsers can't write to a thumbnail cache
on disk. If per-user thumbnail customization is desired later, store as
base64 in `localStorage` (small images only) or skip the feature entirely.

`last_used` updates on every miniapp open. `localStorage` has a 5–10 MB
quota; the config is well under 1 KB so this is fine.

---

## Common JS — `common.js` rewrite

Sandbox's `src/js/common.js` has these helpers:

- `invoke(cmd, args)` — wrapper for `window.__TAURI__.core.invoke`.
  **DELETE.** No backend in the web version.
- `showToast(message, type)` — bottom-right toast. **KEEP.** Pure DOM.
- `showSpinner()` / `hideSpinner()` — full-screen loading overlay. **KEEP.**
- `confirmAction(message)` — modal confirm. **KEEP.**

Rewrite as a single file with no Tauri imports. Then any references to
`invoke(...)` from miniapp code become direct JS function calls (e.g.
allocation logic that previously hit Rust now runs inline).

---

## UI / Theme

Use Sandbox's existing dark theme **verbatim**. Copy `src/css/base.css`
into the web project. The CSS variables and component classes are
already battle-tested.

Photog Tips has its own self-contained dark theme in its `<style>` block;
leave it alone.

Logos / icons: copy from `src/img/` and per-miniapp folders.

---

## What NOT to Port

- Anything under `src-tauri/` (Rust backend)
- `window.__TAURI__.*` references — there is no Tauri in the browser
- `app_copy_file`, `app_copy_tree`, `app_open_path` (`fs_helpers` module)
  — replaced by `<a download>` browser behavior
- Thumbnail save/load via `commands/thumbnails.rs` — replaced by direct
  URL references in the manifest
- The auto-updater plan from `Project_Status.md` — irrelevant for web
- The NSIS/MSI installer config — irrelevant for web
- The `Sandbox_portable.zip` build — irrelevant for web
- Cargo.toml, tauri.conf.json, package.json (the Tauri one) — none apply

---

## Out-of-Repo Files Needed

Vendored libraries to add to the web project (download once, commit):

- **Alpine.js 3.x** — from Sandbox repo at `src/js/alpine.min.js`
- **SheetJS Community Edition** — `xlsx.full.min.js` from
  https://cdn.sheetjs.com/xlsx-latest/package/dist/xlsx.full.min.js
- **sql.js** — `sql-wasm.js` and `sql-wasm.wasm` from
  https://github.com/sql-js/sql.js/releases (latest)
- **JSZip** (if implementing CSV-zip download for DataLore) — from
  https://stuk.github.io/jszip/
- **FileSaver.js** (optional, for nicer download UX) — from
  https://github.com/eligrey/FileSaver.js/

---

## Acceptance Criteria

V1 is shippable when:

1. `https://princecorwinstudios.com/sandbox/` loads the home page
2. Anonymous visit redirects to Cloudflare Access login
3. After login, the home shows three miniapp cards: FW Allocation,
   DataLore, Photography Tip Cards
4. Search, sort, tag filter, favorite-toggle, edit modal, add/delete tag
   all work and persist across browser reloads
5. Photo Tips opens, search works, lightbox works, swipe/arrow navigation
   works
6. FW Allocation: pick two .xlsx files, click Run, see results stats,
   download updated labor file, results match what the Tauri version
   produces on the same input
7. DataLore: pick a .db file, choose format (Excel/CSV/Both), see
   per-table row counts, download outputs
8. Works on Chrome, Edge, Firefox on Windows; Safari on iOS/Mac
9. Mobile layout is usable on a phone (cards stack, search and back
   nav reachable)
10. Custom domain certificate is valid (Cloudflare handles automatically)

---

## Open Decisions (User Input Needed)

Resolve these before/during implementation. Do not assume.

1. **Parent site project status.** Is `princecorwinstudios.com` already
   live on Cloudflare Pages with its own project, or does Sandbox start
   the Pages project? If parent already has Pages, sandbox lives as a
   subfolder in the same repo. If not, you'll be standing up the Pages
   project too — confirm the parent's needs first.
2. **Identity provider.** Google OAuth, GitHub OAuth, or email PIN for
   Cloudflare Access? Google is one-click for a Gmail user.
3. **Mobile priority.** Phone-usable in v1 or defer? If v1, plan extra
   CSS work for the cards-stack-on-narrow layout (current Sandbox is
   desktop-only).
4. **Optional auth bypass for owner.** Want a way to skip Access when
   testing locally (e.g. localhost dev URL exempt from policy)? Easy to
   add as a second Access policy: "Include: AuthN method = warp" or
   similar.
5. **Domain spelling.** User has variously typed `princecorwinstudios.com`
   (plural, recommended) and `princecorwinstudio.com` (singular).
   Confirm the actual registered domain before any DNS or Pages config.

---

## Suggested Implementation Order

1. Stand up Cloudflare Pages project + custom domain + Access policy.
   Confirm `/sandbox/index.html` (a placeholder) requires login.
2. Port `photog_tips` first (trivial). Validate the whole hosting +
   auth + asset-loading pipeline end-to-end with a simple miniapp.
3. Port the home page next. Get tags, favorites, sort, edit modal all
   working against `localStorage`. At this point, only photog_tips is
   on the home but everything else is mechanical.
4. Port `fw_allocation` (Excel logic). Use the Tauri version side-by-side
   to validate output parity on real input files.
5. Port `datalore` (SQLite logic). Same parity-test approach.
6. Port the help manual. Update sections that refer to native UI
   ("Reveal in Run Folder" etc.).
7. Final QA pass on mobile, on each browser, on auth edge cases (logged
   out → tries to open a deep link to a miniapp).

---

## Handoff Notes for the Implementing Claude

- **The Sandbox repo is the spec.** When in doubt about behavior,
  cross-reference the Tauri code. The Rust commands describe exact
  algorithms; the Alpine components describe exact UI flows.
- **Do not modify the Sandbox repo.** Read-only reference. Any
  improvements that should land back in Sandbox should be raised with
  the user separately.
- **`localStorage` is brittle.** Wrap reads/writes in try/catch; treat
  corrupt JSON as "first-run state" and rewrite the default.
- **Don't add a build step unless required.** The user values the
  no-bundler simplicity of Sandbox. Vanilla JS + Alpine + a few
  vendored libraries is the bar. If sql.js or SheetJS forces a build
  step (they don't — both have prebuilt scripts), reconsider.
- **Auth is enforced at the Cloudflare edge, not in your code.** Your
  code can assume any request that reached it is from an authenticated
  user. Don't reinvent auth in JS.
- **Update this PRD as decisions land.** If you change scope or
  approach, record it here so future-you has the context.
