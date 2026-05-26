# Completed Work

## 2026-05-26

- [x] Committed previously-pending local-only changes that had drifted out of sync with `origin/main`:
  - Added `AGENTS.md` (project conventions doc for agent-style assistants) and `.agents/skills/` mirror.
  - Added two new Claude Code skills: `.claude/skills/publisher/skill.md` (release workflow — version bump → installer build → GitHub Release publish) and `.claude/skills/speedup/skill.md` (summarises Project_Status.md todos).
  - Trimmed CLAUDE.md `## Git Commits` section: removed verbose pre-commit checklist and `git add -A` enforcement rules; now defers that workflow to the `/finisher` skill (single source of truth) and just notes "finisher is user-invoked, don't self-invoke".
  - Changed thumbnail rendering from `object-fit: cover` to `object-fit: contain` in `src/css/base.css` so shipped miniapp icons (e.g. FW Allocation green-pipes logo) aren't cropped in card view.
  - Appended new permission entries to `.claude/settings.local.json` allowlist (git commit, tasklist, PowerShell Get-Item/Get-ChildItem, etc.).
- [x] Confirmed `src-tauri/Cargo.toml` "modified" status was a Git CRLF/LF line-ending false-positive (no content diff) — did NOT stage it; working tree clean afterward.
- [x] Archived April 2026 entries from `Plans/Completed_Work.md` to `Plans/Archives/Completed_Work_2026-04.md` per the finisher month-rollover rule.

## 2026-05-22

- [x] Added `Plans/sandbox-supabase-plan.md` — architecture plan documenting Supabase free-tier as the chosen backend for SQL-based Sandbox miniapps. Captures rationale (rejected NAS/SQL Server, Azure, Cloudflare Tunnel, Synology Drive sync), free-tier limits, standard client/server topology, .NET SDK notes, and bootstrap next-steps. No code changes.

## 2026-05-21

- [x] Renamed `.claude/skills/finisher/skill.md` to `.claude/skills/finisher/SKILL.md` to match Claude Code's canonical skill discovery convention. Lowercase happens to work on Windows (NTFS case-insensitive lookup) but Linux/macOS skill discovery looks for `SKILL.md` specifically. Two-step rename via temp filename for reliability on NTFS. No content changes to the skill file.
- [x] Updated stale reference in the April 26 entry below from `.claude/skills/finisher/skill.md` to `SKILL.md` for consistency with the new canonical casing.

---

**Archives:** See Plans/Archives/ for previous months.
