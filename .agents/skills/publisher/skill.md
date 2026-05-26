---
name: publisher
description: >
  End-to-end release workflow that bumps the version, builds the installer,
  and publishes to GitHub Releases. Invoked explicitly by the user via /publisher.
---

# Publisher

Automates cutting a new release of Sandbox. Runs all steps in sequence without pausing for user review, except where explicit confirmation is required.

## Workflow Context

The typical release flow is: work → `/finisher` (commit session work) → test the committed build → `/publisher` (cut release). Publisher assumes finisher has already run — it does not commit outstanding session work.

## Prerequisites

Before running publisher, confirm:
1. Working tree is clean (`git status` shows nothing uncommitted) — if not, stop and ask the user to run `/finisher` first or handle manually
2. Current branch is `main` — if not, confirm with user
3. The committed build has been tested

If any of these are not confirmed, ask the user before proceeding.

## Step 1: Determine Next Version

Read the current version from `src-tauri/tauri.conf.json` (the `"version"` field).

Sandbox uses semver (`MAJOR.MINOR.PATCH`). Propose the next version based on the nature of changes in `Plans/Completed_Work.md` since the last release:
- **PATCH** bump for bug fixes and minor improvements
- **MINOR** bump for new features (backward compatible)
- **MAJOR** bump for breaking changes

Show the user the proposed version and **wait for explicit confirmation** before proceeding. Example:

```
Current version: 1.0.0
Proposed next version: 1.1.0 (new miniapp added)
Proceed? (yes/no)
```

## Step 2: Bump Version

Update the version in BOTH files (they must match):
- `src-tauri/tauri.conf.json` — the `"version"` field
- `src-tauri/Cargo.toml` — the `version = "..."` line at the top

## Step 3: Build the Installer

Run:

```
npx tauri build
```

This produces installers in `src-tauri/target/release/bundle/`:
- NSIS installer: `src-tauri/target/release/bundle/nsis/Sandbox_X.Y.Z_x64-setup.exe`
- MSI installer: `src-tauri/target/release/bundle/msi/Sandbox_X.Y.Z_x64_en-US.msi`

If the build fails, STOP. Do not proceed. Report the error to the user.

After success, verify both installer files exist.

## Step 4: Draft Release Notes

Draft user-facing release notes sourced from the most recent entries in `Plans/Completed_Work.md` since the last release, translated to user-facing language:
- Example: "Fixed thumbnail race condition in discovery.rs" → "Improved reliability of app thumbnails"
- Example: "Refactored config loader to use serde" → (skip — internal refactor, no user impact)

Show the user the proposed notes and **wait for confirmation** before proceeding.

## Step 5: Commit and Push Version Bump

- Run `git add -A`
- Commit message format: `Release vX.Y.Z`
  - Do NOT include "Generated with Codex" or AI attribution
- Run `git push`

## Step 6: Create GitHub Release

Run:

```
gh release create vX.Y.Z "src-tauri/target/release/bundle/nsis/Sandbox_X.Y.Z_x64-setup.exe" "src-tauri/target/release/bundle/msi/Sandbox_X.Y.Z_x64_en-US.msi" --title "Sandbox vX.Y.Z" --notes "<release notes from Step 4>"
```

Release conventions:
- **Tag:** `vX.Y.Z` (with "v" prefix)
- **Title:** `Sandbox vX.Y.Z`
- **Assets:** Both NSIS (`.exe`) and MSI (`.msi`) installers
- **Notes:** User-facing change description from Step 4

## Step 7: Post-Publish Verification

After the release is created:
1. Confirm the release exists: `gh release view vX.Y.Z`
2. Confirm both installer assets are attached and downloadable
3. Report back to the user with:
   - Commit hash (short)
   - Version published
   - GitHub release URL

## Important Rules

- All file paths are relative to the repository root — NEVER use absolute paths
- NEVER proceed past Step 1 without explicit user confirmation of the version number
- NEVER proceed past Step 4 without explicit user confirmation of the release notes
- If any step fails, STOP and report — do not attempt to continue or partially recover without user direction
- The version in `tauri.conf.json` and `Cargo.toml` MUST match — verify before building
