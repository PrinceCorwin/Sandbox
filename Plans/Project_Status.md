# Project Status

**Last Updated:** 2026-05-31

## In Progress

- [ ] Test FW Allocation miniapp with real Excel files
- [ ] Set up auto-updater (signing keys, GitHub Releases endpoint)
- [ ] Set up GitHub Actions workflow for release builds

## Upcoming

- [ ] Settings menu content (theme selection, about, check for updates)
- [ ] Clean up unused mobile icon files Tauri generated in src-tauri/icons/ (iOS/Android PNGs not needed for Windows-only app)
- [ ] Consider prune-orphans logic in discovery.rs to auto-remove config entries whose miniapp folders were deleted
- [ ] Stand up Supabase backend for SQL-based miniapps (see `Plans/sandbox-supabase-plan.md`) — create project, save keys, wire SDK
- [ ] Add more miniapps as needed
- [ ] Fix broken `bundle.resources` glob in `src-tauri/tauri.conf.json`. The current pattern `"../src/apps/*/app.json": "apps/"` collapses every miniapp's `app.json` to the same destination path (`apps/app.json`), so only the last one wins on a production install. This is masked on dev machines by `discovery.rs`'s fallback to the project source path, but a fresh-machine install via NSIS or portable zip would show only one miniapp. Fix: enumerate each app explicitly (e.g. `"../src/apps/fw_allocation/app.json": "apps/fw_allocation/"`) or switch to a copy step that preserves subdirectory structure. Decide alongside the Tauri-vs-web decision — irrelevant if Sandbox pivots to Cloudflare Pages.
- [ ] **Decide: keep Sandbox on Tauri or pivot to Cloudflare Pages.** All current miniapps (`fw_allocation`, `datalore`, `photog_tips`) are client-side compatible — Excel via SheetJS, SQLite via sql.js, photog_tips already pure static. Pages free tier handles unlimited bandwidth, custom domain (`princecorwinstudios.com`) already on Cloudflare. Wins: no install/distribution problem, no Tauri toolchain pain, multi-device including mobile. Catches: file-save UX via browser downloads is clunkier, no offline guarantee.
