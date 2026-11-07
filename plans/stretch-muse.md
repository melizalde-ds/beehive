# Stretch S1: third-party PAP client + real Meta Muse

**Goal:** Prove that agents we didn't write can find the shop. First the datalayer SDK, then Meta's Muse.
**Depends on:** phase 06 (end to end works) and a deployed HTTPS domain.
**Never cut for this:** the dashboard slot, the $120 block, the replay block, verify + tamper.
**Before starting:** open decisions 8-12 in `00-general.md`.

## Update 2026-11-06 21:20: on, run as tracks A + E

Meta confirmed Muse speaks PAP. The live URL is up (`https://32-189-214-147.sslip.io`, real Redo). Muse prep runs as track A and the Muse test as track E, in `phase-08-last-mile.md` under "Parallel tracks". The old 10:30 hard stop moves to **10:40**. After that, the time goes to the backup recording.

**Rungs reached:**
- 2026-11-06 ~22:30: rung 1. Muse loaded the dashboard, `/api/shop`, `/api/returns` and `/.well-known/poppy.json` (all 200) through a browser tool (IP 104.29.192.79, desktop Chrome UA), not a PAP client. It then suggested testing a PAP flow. Before that, an `ERR_CERT_COMMON_NAME_INVALID` came from somewhere other than our box (our TLS was valid; no mismatched cert is ever served).

- 2026-11-06 21:58 MST: rungs 2 and 3. From the same IP, in a browser (Hoppscotch, per the user), Muse sent `POST /poppy/session` as `muse-demo` ($50, read+write), the consent screen was opened and approved, Muse polled the session for its token, then `POST /poppy/returns` (o_1, damaged, $36). The gate blocked it at Order (`bh_0004`): the cold-open `bh_0001` already returned o_1. Who clicked Approve (Muse or the user inside Muse's browser) is still to confirm.

- 2026-11-06 22:05 MST: **top rung.** After `demo/reset?empty=1`, Muse opened a new session (after 3 `GET /poppy/session` 405s it switched to POST), approved consent from its browser again, and filed o_1 damaged $36. Gate **passed** `bh_0001`; real Redo return `6aeeb19bebc57cda8dab553f` (open); `GET /api/verify/bh_0001`: "All 6 links check out".

**Muse ladder (each rung is a win; record it as soon as it's reached):**
1. Muse reads `/.well-known/poppy.json`. Seen in the Caddy log.
2. Muse opens a session, and Kaylee approves the $50 cap on the consent link.
3. Muse files the $36 (Kaylee's prompt names order `o_1`, see D2). A "Muse" card shows on the dashboard and the return opens in Redo. Blocked at `agent_identity` until Muse's id is in the seed (D3). A block like that is still a usable rung: a real third-party agent stopped by the gate.

**Gaps known before the test:**
- No order lookup in our PAP: Muse can't find `o_1` by itself (D2).
- The gate only knows `muse-demo` (D3).
- The return fields and `reason` values must be in discovery, or Muse has to guess them (track A).

On stage: Muse files the $36 (live or recorded), then `agent.ts --skip-candle` runs the $120 block and the replay (track B).

## Validate with a third-party PAP client (datalayer SDK)

Goal: prove a client I didn't write can discover the shop. Run against the deployed HTTPS domain, not localhost (the SDK's URL safety policy may block local hosts).

```bash
pip install --break-system-packages personal-agent-protocol   # targets draft 0.3
```

`scripts/validate_pap.py`
```python
import asyncio, sys
from personal_agent_protocol import DiscoveryClient

async def main(domain: str) -> None:
    async with DiscoveryClient() as client:
        company = await client.discover(domain)
        print("org:", company.document.organization.name)
        print("protocols:", company.document.agent.protocols)
        print("OK: discovery passed")

asyncio.run(main(sys.argv[1]))
```

```bash
python scripts/validate_pap.py beehive.<domain>
```

Pass = prints org name, no exception. Fail = fix poppy.json or issuer metadata, rerun.
Checks the SDK makes (fix these first if it fails):
- `/.well-known/poppy.json` returns JSON over HTTPS (no HTML soft 404)
- `protocol_version` matches the SDK's draft
- `auth.issuer` metadata at `/.well-known/oauth-authorization-server` has an `issuer` that matches exactly and lists our domain in `poppy_domains`

Pin the SDK version once it passes. Early releases target draft 0.1 and newer ones draft 0.3, so an unpinned install can change the result.

## Does Muse need to sign in? (test before 10:00)

PAP lets an agent start as a **guest** (enough for a returns-policy question), and **sign-in is OAuth** through the `auth.issuer` in `poppy.json`. Filing a return touches a customer's order, so Muse may insist on sign-in. The brief cuts real OAuth, so this test decides whether Muse can reach our gate at all.

No code needed: Caddy's access log shows every path Muse hits. Run `tail -f` on it during each step.

1. Ask Muse a guest-only question: "What's Cache Valley Honey Co.'s return policy?" Expect a `GET /.well-known/poppy.json`. If nothing shows up, Muse isn't discovering the domain and the test is over.
2. Ask Muse to "return my broken candle set". Then read the log:
   - `POST /poppy/session` with `mode: guest`, then waiting on our `consent_url` means **no sign-in needed**. Our flow works, so go on to step 3.
   - `GET /.well-known/oauth-authorization-server`, then a hit on its `authorization_endpoint` / `token_endpoint` means **Muse wants real OAuth sign-in**. Stop: that's out of scope. Pitch it as "Next".
   - A request with a `DPoP` header means Muse wants DPoP, which is also cut ("DPoP is next"). Stop.
3. Approve on the consent screen with the $50 cap. Check the gate's `agent_id` in the server log. If it's blocked by `agent_identity`, Muse's id isn't in the seed (open decision 11).

## Stretch: real Meta Muse (timeboxed)
- 9:30: deployed on AWS (Lightsail/EC2 + Caddy TLS + Axum + static Vite)
- 9:30-10:00: SDK validation passes
- 10:00-10:30: try Muse. If it discovers + starts a session, record it immediately as backup
- 10:30: hard stop. If not working, move to "Next" in the pitch
- On stage: Muse files the candle return (live or recorded). The script does the $120 block and replay (deterministic)
- Never let Muse eat the dashboard slot

## Clashes with what's built (recommended answers, not decided yet)

Open decisions 8-12 in `00-general.md`. Each needs a yes before S1 starts.

| # | Clash | Recommended |
|---|---|---|
| 8 | The SDK checks the draft 0.3 discovery shape (`protocol_version`, `organization`, `auth.issuer`). Our `PoppyDiscovery` is our own `0.1-draft` shape, so the SDK fails until the schema changes. Phase 04 builds against the current shape. | Change the schema now, before phase 04 builds much on it |
| 9 | AWS deploy at 9:30 overlaps phase 07 (wow hardening) | Deploy only once 07 is solid |
| 10 | The general plan says no Python | Allow it for this one script, off the demo path |
| 11 | The gate only knows `muse-demo`, so real Muse is blocked by `agent_identity` | Add Muse's real id to the seed after the sign-in test shows it |
| 12 | Needs a static OAuth metadata file; the brief cuts real OAuth | Serve the file only. If Muse follows it to sign in, stop |

Sources for PAP, the SDK and Muse: `docs/resource.md`.

## Done when

- [ ] SDK prints the org name against the live domain
- [ ] Sign-in test result written here (guest works / needs OAuth / needs DPoP / no discovery)
- [ ] Muse recording saved, or S1 moved to "Next" by 10:30
- [ ] Any Muse or SDK surprise logged in `docs/findings.md`
