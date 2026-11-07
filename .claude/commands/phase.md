---
description: Plan and build one phase in this session
argument-hint: "[phase number] [folders this phase may touch]"
---

Phase: $ARGUMENTS

Read `brief/brief.md`, `plans/00-general.md` and `AGENTS.md`. Only the rules for the files you touch.

1. Make sure you're on branch `phase-##-name`. Create it if not.
2. Write `plans/phase-##-name.md` from `plans/phase-template.md`. Short checklist, this phase only.
3. Build it. Use subagents only for parts that are truly independent.
4. Other phases may be running in parallel. Don't touch files outside this phase's folders. A schema change is "ask first".
5. If something is unclear, pick the reversible option and write it under "Assumptions made".
6. Walk this phase's step of the demo path. Report what you clicked and what you saw.

Don't commit. Stop and wait for me.
