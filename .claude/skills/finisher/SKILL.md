---
name: finisher
description: >
  End-of-session workflow that updates project documentation and commits
  changes. Trigger when the user says "finisher" or "commit". Do NOT trigger
  on "finish", "wrap up", or other variations — only the exact words
  "finisher" or "commit".
---

# Finisher

End-of-session automation that documents completed work and commits/pushes
all changes. Run all steps in sequence without pausing for user review.

## Prerequisites

Before running finisher, confirm:
1. All work has been tested/verified by the user
2. The project is in a stable state (build clean, tests passing — whichever
   applies to this project)

If these are not confirmed, ask the user before proceeding.

## Step 1: Update Plans/Project_Status.md

- Read `Plans/Project_Status.md`
- Identify backlog/todo items completed during this session and remove them
- Add any newly identified work that is NOT yet complete to the appropriate
  section
- Update **Last Updated** at the top to today's date
- Preserve the existing structure and formatting

## Step 2: Update Plans/Completed_Work.md

### Month rollover check
Before adding entries, scan ALL `###` date headers in `Completed_Work.md` —
not just the first one:

1. Read `Completed_Work.md` and collect every `###` date header
2. Check if ANY entries are from a month older than the current month
3. **If old-month entries exist:**
   - Move those entries (and only those) into an archive file at
     `Plans/Archives/Completed_Work_YYYY-MM.md`, using the OLD month's
     year-month
   - If the archive file already exists, append to it
   - Archive file header format:
     `# Sandbox — Completed Work (Month YYYY)`
     (e.g. "March 2026")
   - Archive file contains only the entries (the `###` date sections), no
     `## Unreleased` header or intro paragraph
   - Keep current-month entries in `Completed_Work.md` with the standard
     header/intro
   - End the live file with `---` followed by
     `**Archives:** See Plans/Archives/ for previous months.`
4. **If all entries are current month:** add the new entry at the top as
   normal (below `## Unreleased`)

### Adding entries
- Add a new `### {Month D, YYYY} ({Brief title})` section at the TOP of the
  changelog, below `## Unreleased`
- Concise but descriptive bullet points
- Group related changes
- Include detail that helps a reader understand what changed (file names,
  feature names, behavioral changes)

## Step 3: Update src/help/manual.html (user-facing changes only)

`src/help/manual.html` is the **in-app help manual** shown to end users
inside Sandbox. It must stay in sync with anything a user can see or
interact with.

- Determine if the session's work changed user-facing behavior — UI,
  miniapp features, workflows, buttons, dialogs, settings, error messages
  the user sees, etc.
- If YES:
  - Read `src/help/manual.html`
  - Add new sections/subsections for new features
  - Update existing sections if behavior changed
  - Remove documentation for removed features
  - Update the Table of Contents (`.toc ul`) with anchor links for any new
    `<h2>` sections added — the existing TOC pattern uses
    `<li><a href="#anchor">Title</a></li>`
  - Match the existing HTML structure, styling, and CSS variable conventions
    in the file — do not introduce new colors, fonts, or layout rules
- If NO user-facing changes were made (internal refactor, build tooling,
  dev-only tweaks), skip this step

## Step 4: Update Plans/Decisions.md

`Plans/Decisions.md` captures **architectural choices and dev-side
rationale** — the kind of context that wouldn't go in the user manual but
is important to remember when revisiting the code.

- Review the session's work for design decisions, architectural choices,
  implementation rationale, or non-user-facing changes worth recording
- A "decision" is a choice between alternatives with reasoning, OR a
  non-trivial dev-only change (refactor, dependency swap, internal
  restructuring) — NOT routine bug fixes or simple feature additions
- Examples: "chose X over Y because...", "removed X because...", "changed
  approach from X to Y because...", data format choices, things
  intentionally NOT done and why, internal refactors with rationale
- If decisions / dev-only changes were made:
  - Read `Plans/Decisions.md`
  - Add new entries under the appropriate section, creating a new section
    if none fits
  - Follow the existing format: `### Title`, `**Decision:**`, `**Why:**`,
    `**Date:**`
- If nothing of this nature happened this session, skip this step

## Step 5: Commit and push

- `git add -A`
- Auto-generate a clear, concise commit message summarizing the session's
  functional work
  - Do NOT include "Generated with Claude" or "Co-Authored-By: Claude"
  - Do NOT include AI attribution of any kind
- `git commit` with the generated message
- `git push` to the current checked-out branch (no branch restrictions)

## Important Rules

- All file paths are relative to the project root — NEVER absolute
- Do NOT selectively commit files — always `git add -A` unless the user
  explicitly excluded specific files during the session
- The commit message describes the functional change, not the documentation
  update (e.g. "Add CSV export to reports view", not "Update docs and commit")
- If there are no uncommitted code changes (only doc updates), still commit
  the doc updates with an appropriate message
