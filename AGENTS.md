# AGENTS.md

<!-- Fill the brackets at kickoff. Keep this file short. Detail lives in brief/ and .claude/rules/. -->

## Project

**Beehive**: a Redo plugin that lets a customer's own AI agent file returns and claims through PAP, gated by Rust.
Built for **Crimson Hacks 2026**, solo. Stack: Vite + React dashboard (static) + Rust (Axum) service that owns the PAP front door, gate and Redo handoff.

Read `brief/brief.md` before touching any feature. Phases live in `plans/00-general.md`.

Redo: `docs/redo-integration.md` records how the real Redo API behaves (tested against a real store; responses in `redo-fixtures/`). On Redo behavior it beats the brief. It's a record of behavior, not a spec: adopt what the demo needs. Bugs found are in `docs/redo-findings.md`. API shapes we own are in `schema/` (see `schema/README.md`).

This is a hackathon build. `.claude/rules/hackathon.md` wins over any stricter rule.

## Commands

| Task | Command |
|------|---------|
| Install | `cd web && pnpm install` (Rust deps install on first `cargo` run) |
| Dev | `cd server && cargo run` (API on `:3000`), then `cd web && pnpm dev` → `http://localhost:5173` |
| Dev session (from phase 02) | `cd server && BEEHIVE_DEMO_SESSION=1 cargo run`, token is in the log |
| Contract check | `cd server && cargo test` · after a schema change: `cd web && pnpm gen:types` |
| Build | `cd server && cargo build --release` · `cd web && pnpm build` |
| Agent script (phase 04) | `BEEHIVE_URL=http://localhost:3000 node agent/agent.ts --reset`, then approve the consent link it prints (`--reset` empties the shop first) |
| Cold-open demo (phase 07) | `BEEHIVE_DEMO_SEED=1 cargo run`: files the $36 pass, $120 block and replay block at startup. "Reset demo" on the dashboard empties it |
| Real Redo (phase 03) | `cd server && set -a; . ../.env; set +a; REDO_MODE=real cargo run` (seeds the Redo store at startup; falls back to mock with a warn) |
| Deploy | `scripts/deploy.sh 32-189-214-147.sslip.io` (Docker build, then ships to the Lightsail box; settings in `/etc/beehive.env` on the box, made from `.env` on first deploy). Mock fallback: set `REDO_MODE=mock` there, `sudo systemctl restart beehive` |
| Live URL | https://32-189-214-147.sslip.io (agent: `BEEHIVE_URL=https://32-189-214-147.sslip.io/api node agent/agent.ts --reset`) |

## Walk the demo

1. Start the API and the dashboard (see Dev above), open `http://localhost:5173`.
2. No login. The demo customer is Kaylee Jensen (`c_1`), and the agent is `muse-demo`.
3. Run the agent script (see Commands), open the consent link it prints, approve the $50 cap, and watch the dashboard. Then follow the rest of the demo path in `brief/brief.md` in a browser tool. Rerun with `--reset` (or click "Reset demo") between runs; state is in memory.
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
| `.claude/rules/vite.md` | Vite SPA (dashboard + consent screen) |
| `.claude/rules/rust.md` | `.rs`, `Cargo.toml` |
| `.claude/rules/contract.md` | `schema/`, or any code that crosses languages |

## Workflow

- `[AGT-01]` One phase per session: read the brief, the general plan and that phase's plan. Not the whole repo.
- `[AGT-02]` Plan the phase as a short checklist in `plans/phase-##-name.md`, then build it.
- `[AGT-03]` Use subagents only to run independent work in parallel. Don't use `superpowers:subagent-driven-development` or `superpowers:test-driven-development`.
- `[AGT-04]` Use `superpowers:brainstorming` to scope, `superpowers:systematic-debugging` for bugs, `superpowers:verification-before-completion` before saying it works.
- `[AGT-05]` Never assume. When something is unclear, ask me before deciding (one batch of questions, recommended option first). If you can't ask live, stop and put the questions at the top of your report.
- `[AGT-06]` Hit something a sponsor would want to hear (Redo or PAP gap, bug, surprising result)? Add an entry to `docs/findings.md` right away, using its template, and mention it in your report.

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

- `pkill -f <pattern>` matches its own shell command and kills it. Use `pkill -x beehive`, or find the pid with `ps` first.
- Edit `schema/beehive.schema.json` as text. Rewriting it with a JSON library reformats the whole file and buries the diff.
- `jsonwebtoken` 11 needs a crypto backend feature; we use `rust_crypto`.
- `.env` isn't loaded automatically (no dotenv crate yet). Export vars in the shell, or ask before adding `dotenvy`.
- `shadcn add` in `web/` is non-interactive only with `</dev/null`. It imports `cn` from the `cn` package (shadcn's own); that's expected.
- API responses are validated with zod built from `schema/beehive.schema.json` (`web/src/api/client.ts`). A schema change is picked up without hand edits; keep `$schema` draft-07 there or `definitions` refs break.
- `.gitnexus/` is gitignored, so worktrees don't have the GitNexus CLI. Run it from the main checkout.

<!-- gitnexus:start -->
# GitNexus — Code Intelligence

This project is indexed by GitNexus as **crimson-hacks-project** (69 symbols, 63 relationships, 0 execution flows).

> Index stale? Run `node .gitnexus/run.cjs analyze --index-only` from the project root — it auto-selects an available runner. No `.gitnexus/run.cjs` yet? Bootstrap with `npx`, `bunx`, or `pnpm dlx` — e.g. `bunx gitnexus@latest analyze` (npm 11 npx crash; #1939).

## Always Do

- **MUST run impact before editing.** Use `impact({target: "symbolName", direction: "upstream"})` or `node .gitnexus/run.cjs impact "symbolName" --direction upstream --repo .`; report callers, processes, and risk. Never substitute grep for graph analysis.
- **MUST analyze graph changes before committing.** Use `detect_changes({scope: "all"})` (MCP) or `node .gitnexus/run.cjs detect-changes --scope all --repo .` (CLI fallback). `partial: true` or `truncated: true` is not a clean check — a zero means unseen, not unaffected; re-run it. For regression review: `detect_changes({scope: "compare", base_ref: "main"})` or `node .gitnexus/run.cjs detect-changes --scope compare --base-ref "main" --repo .`.
- MUST warn on HIGH/CRITICAL `risk` pre-edit; never use `riskSharedAxes` to waive a HIGH/CRITICAL `risk` warning. Compare File/symbol: MCP File omits axes; Graph-RAG expands File.
- **MUST treat `risk: UNKNOWN` as unresolved, not as low.** An empty caller set is not evidence the symbol is unused — it can also mean the callers are not resolvable by the index (plain-object property access, dynamic dispatch, cross-language calls). `impact` pairs `UNKNOWN` with a `riskNote` saying so. Confirm with a text search before treating the symbol as safe to change or delete; do not proceed on the strength of a zero.
- **MUST use `query({search_query: "concept"})` for concepts/flows, `context({name: "symbolName"})` for a named symbol, or `impact` for blast radius, on read-only callers, dependencies, imports, or execution flow.** Graph first; text search only for empty/`UNKNOWN`/literals.
- For security review, `explain({target: "fileOrSymbol"})` lists taint findings (source→sink flows; needs `analyze --pdg`).

## Never Do

- NEVER edit a function, class, or method before MCP/CLI impact analysis.
- NEVER ignore HIGH or CRITICAL risk warnings from impact analysis, and never read `UNKNOWN` as an all-clear — it means the walk could not answer, which is the one verdict that requires confirming by other means.
- NEVER rename symbols with find-and-replace — use `rename` which understands the call graph.
- NEVER commit before MCP/CLI graph change analysis.

## Resources

| Resource | Use for |
| --- | --- |
| `gitnexus://repo/crimson-hacks-project/context` | Codebase overview, check index freshness |
| `gitnexus://repo/crimson-hacks-project/clusters` | All functional areas |
| `gitnexus://repo/crimson-hacks-project/processes` | All execution flows |
| `gitnexus://repo/crimson-hacks-project/process/{name}` | Step-by-step execution trace |

## CLI

| Task | Read this skill file |
| --- | --- |
| Understand architecture / "How does X work?" | `.claude/skills/gitnexus-exploring/SKILL.md` |
| Blast radius / "What breaks if I change X?" | `.claude/skills/gitnexus-impact-analysis/SKILL.md` |
| Trace bugs / "Why is X failing?" | `.claude/skills/gitnexus-debugging/SKILL.md` |
| Rename / extract / split / refactor | `.claude/skills/gitnexus-refactoring/SKILL.md` |
| Tools, resources, schema reference | `.claude/skills/gitnexus-guide/SKILL.md` |
| Index, status, clean, wiki CLI commands | `.claude/skills/gitnexus-cli/SKILL.md` |

<!-- gitnexus:end -->
