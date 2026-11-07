# Hackathon Starter

My kit for solo hackathons with AI agents. Short in-person events, 24 to 48 hour weekends, and online multi-day builds.

Inspired by [Juan Lizarazo's hackathon-starter-kit](https://github.com/juanlizarazo/hackathon-starter-kit). My fork of it stays as a reference.

## Use it

1. **Use this template** → new repo for the event.
2. Find the idea in the **Hackathon Bank** Claude Project (or `/idea` here).
3. Fill `brief/brief.md` and the Project section of `AGENTS.md`.
4. Delete the stack rules you won't use (`next.md` or `vite.md`, `rust.md`, `python.md`, `contract.md`).
5. Run the commands in order.

## Flow

| Step | What | Command | File |
|------|------|---------|------|
| 1 | Find the idea or the angle | `/idea` or `/bounty` | `start/` |
| 2 | Stack sponsor tracks, talk to sponsors | | `start/sponsors.md` |
| 3 | Brief: identity, wow moment, rubric, scope | | `brief/brief.md` |
| 4 | General plan, blocking phase first | `/plan` | `plans/00-general.md` |
| 5 | One phase per session, parallel when possible | `/phase 02 src/feature` | `plans/phase-template.md` |
| 6 | Check the wow moment | `/wow` | |
| 7 | Fresh-session audit | `/audit` | |
| 8 | Persona reviews | `/persona sponsor Elastic https://...` | |
| 9 | UI polish, sized by the rubric | `/polish` | |

Agents never commit on their own. `.claude/settings.json` makes `git commit`, `git push`, `gh pr create` and `vercel` ask first.

## Day 0

Costs zero event hours if done before.

- [ ] Event repo made from this template
- [ ] Idea picked, brief filled
- [ ] Sponsor tools tried in a tiny practice project
- [ ] Sponsor plugins turned on in `.claude/settings.json` (or added)
- [ ] Model API keys with credits, plus a second provider for when one hits limits
- [ ] Deploy target live with a hello world (Vercel for Next or Vite)
- [ ] Rust or Python service: a hosted spot ready (Fly, Railway, or the homelab)

## Layout

```text
AGENTS.md            index: project, commands, demo walk, boundaries, gotchas
CLAUDE.md            imports AGENTS.md
.claude/
  settings.json      plugins, commit guard, attribution off
  commands/          /idea /bounty /plan /phase /wow /audit /persona /polish
  rules/             core rules always load; stack rules load by file type
start/               find-an-idea, sponsors, pick-a-bounty
brief/brief.md       the spec
plans/               general plan + phase template
```

Stack rules: `react.md` (shared TS), `next.md` (Next + Vercel), `vite.md` (Vite SPA), `rust.md` (axum service), `python.md` (FastAPI), `contract.md` (JSON Schema shared across languages). Claude loads them by file type. Other agents use the table in `AGENTS.md`.

## Plugins

- `superpowers`: brainstorming, debugging, parallel agents, verification. Skip its subagent-driven development and TDD skills.
- `impeccable`: UI audit and polish. If you already installed it globally, set it to `false` here so it doesn't load twice.
