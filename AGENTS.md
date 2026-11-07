# AGENTS.md

<!-- Fill the brackets at kickoff. Keep this file short. Detail lives in brief/ and .claude/rules/. -->

## Project

**[Product]**: [one-line identity from brief/brief.md].
Built for **[event]**, solo. Stack: [packs in use, e.g. Next.js on Vercel + Rust service].

Read `brief/brief.md` before touching any feature. Phases live in `plans/00-general.md`.

This is a hackathon build. `.claude/rules/hackathon.md` wins over any stricter rule.

## Commands

| Task | Command |
|------|---------|
| Install | `[...]` |
| Dev | `[...]` → `http://localhost:[port]` |
| Build | `[...]` |
| Seed demo data | `[...]` |
| Deploy | `[...]` (ask first) |
| Live URL | `[...]` |

## Walk the demo

1. Start with `[dev command]`, open `[url]`.
2. Log in as `[seeded user]` if there is auth.
3. Follow the demo path in `brief/brief.md` step by step in a browser tool.
4. Check phone width (375px).
5. Report what you clicked and what you saw.

## Rules

Claude loads these by file type. Other agents: read only the files for the area you touch.

| File | Load when |
|------|-----------|
| `.claude/rules/hackathon.md` | Always |
| `.claude/rules/ai.md` | Always (where the model goes) |
| `.claude/rules/git.md` | Always |
| `.claude/rules/react.md` | Any `.ts` or `.tsx` |
| `.claude/rules/next.md` | Next.js app (delete if Vite) |
| `.claude/rules/vite.md` | Vite SPA (delete if Next) |
| `.claude/rules/rust.md` | `.rs`, `Cargo.toml` |
| `.claude/rules/python.md` | `.py`, `pyproject.toml` |
| `.claude/rules/contract.md` | `schema/`, or any code that crosses languages |

## Workflow

- `[AGT-01]` One phase per session: read the brief, the general plan and that phase's plan. Not the whole repo.
- `[AGT-02]` Plan the phase as a short checklist in `plans/phase-##-name.md`, then build it.
- `[AGT-03]` Use subagents only to run independent work in parallel. Don't use `superpowers:subagent-driven-development` or `superpowers:test-driven-development`.
- `[AGT-04]` Use `superpowers:brainstorming` to scope, `superpowers:systematic-debugging` for bugs, `superpowers:verification-before-completion` before saying it works.
- `[AGT-05]` When something is unclear, pick the reversible option, write the assumption at the top of your report, and keep going. Stop only if the choice changes the demo path or can't be undone.

## Boundaries

Always:
- Keep secrets in `.env` (gitignored). Update `.env.example` with the key name.
- Keep the app runnable. The demo can be pulled at any moment.

Ask first:
- `git commit`, `git push`, opening a PR
- Deploying, sending email, calling a paid API in a loop
- Adding a dependency that isn't in a rules file
- Editing `brief/`
- Deleting or reshaping seed data

Never:
- Build what the brief says is out of scope
- Put a model in the decision path (see `ai.md`)
- Add `Co-Authored-By` or "Generated with" lines

## Gotchas

<!-- Add a line each time an agent makes the same mistake twice. Delete lines that stop being true. -->

-
