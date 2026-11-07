---
description: Write the general plan from the brief
---

Read `brief/brief.md` and `AGENTS.md`. Fill `plans/00-general.md`.

- High-level phases only. No code, no implementation detail.
- Phases run in parallel wherever possible. The blocking phase (contract + seed data) goes first.
- The wow moment gets its own hardening phase before the last mile.
- Copy every requirement from brief §6 verbatim.
- List what we cut on purpose and why.
- Anything that doesn't trace to the rubric or the demo path gets cut.

Then list which `.claude/rules/` pack files to delete for this stack.
