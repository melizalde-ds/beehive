# Stretch S1: third-party PAP client + real Meta Muse

**Goal:** Prove that agents we didn't write can find the shop. First the datalayer SDK, then Meta's Muse.
**Depends on:** phase 06 (end to end works) and a deployed HTTPS domain.
**Never cut for this:** the dashboard slot, the $120 block, the replay block, verify + tamper.
**Before starting:** open decisions 8-12 in `00-general.md`.

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
