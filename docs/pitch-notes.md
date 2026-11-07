# Pitch notes: 2-minute presentation and demo

Live: https://32-189-214-147.sslip.io

Rehearse once out loud with a timer. Over 2:00? Cut the lines marked ✂️.

## The 2-minute script (~290 words)

**0:00–0:15 · Hook**
"I had to dig through AllTrails' support chat to get a refund. Soon my own AI agent will do that for me, but only if the shop speaks PAP, the new Personal Agent Protocol from Sierra and Meta. Big brands will build for it. A Cache Valley honey farm can't."

**0:15–0:30 · What it is**
"Beehive is a plugin for Redo. Redo's MCP server lets the *merchant's* AI in. Beehive opens the other door, for the *customer's* own agent. A Rust gate checks who the agent is and what the customer allowed. Only then does Redo get the return."

**0:30–1:15 · Demo (dashboard on screen)**
"Kaylee's agent has a $50 refund cap she approved. Three requests:
- $36 broken candle set. Under the cap, so it passed, and Redo opened a real return.
- $120 honeycomb crate. Over the cap, so it was blocked and never reached Redo.
- The same $36 request sent again. A duplicate, so it was blocked. No double refund.

Every step, from discovery to Redo's answer, is a link in a SHA-256 hash chain. → *click Verify proof* → 6 of 6 links check out. → *click Edit one byte* → Broken, and it points at the exact link I changed."

**1:15–1:40 · It's real**
"This isn't only my script. Tonight I pointed Meta's Muse at the live shop. On its own, it found the shop's PAP discovery file, opened a session, and filed the return into a real Redo store. ✂️ It also approved its own consent screen, so binding consent to the customer's sign-in is the first thing I'll build next."

**1:40–2:00 · Close**
"There's no AI in the decision path. The agent can only ask, plain Rust code decides, and Redo still makes the return call. ✂️ I also wrote up what I found in Redo's API, like having no field to say an agent filed a return. Beehive makes every Redo shop agent-ready, down to a honey farm. It's live, so try it."

## Why I chose this idea

1. **The timing is new.** PAP came out as a draft in Oct 2026. Customers' agents are about to show up at shops, and no shop is ready for them.
2. **It fits the Redo sponsor.** It extends Redo instead of competing with it. Redo keeps the return decision and its fraud review.
3. **It helps Utah.** Small Utah shops can't build agent integrations or hire support staff. If they can't, Utahns default to Amazon for easy returns, and the money leaves the state.
4. **It targets 3 prizes.** Commerce (Redo merchants), Technical (a Rust gate plus provable evidence), Overall (it works live, with a Utah story).

## Why I built it this way

**Safety on the money path**
- **Rust for the gate:** it decides refunds, so it has to be correct and testable. 9 checks run in a fixed order and the first failure blocks. 7 tests cover the gate.
- **No model in the gate:** prompt injection can't talk its way past a cap. The demo agent is a script, so a slow model call can't break the demo.
- **The price comes from the order, not the agent:** otherwise an agent could claim $36 on the $120 crate to slip under the cap. Found and fixed during the event.
- **Replays are blocked twice:** the gate checks the token id and the request id, and Redo's `idempotencyKey` is the backup. Tested: the same key returns the same Redo return.
- **Rate limit counts only after the token checks out:** junk tokens can't use up a real agent's limit.

**Proof**
- **Hash chain:** the shop can prove what the customer allowed, and any edit shows up. No database or blockchain needed.
- **Proof goes in Redo's `notes`:** Redo has no field for "an agent filed this", so the agent, customer, cap and gate hash go into the return's notes.

**Demo reliability**
- **Real Redo, with a mock fallback:** if Redo is down at startup, the demo still runs.
- **Pre-loaded opening plus a "Reset demo" button:** the key moment works cold, without waiting on a live agent run.
- **One JSON schema generates both the Rust and the TypeScript types:** the frontend and backend can't drift apart.

**Cut on purpose:** real OAuth, DPoP, my own return decisions (Redo's job), a risk score (Redo has none), return labels (Redo returns a 500 error when you read an imported return).

## Likely judge questions

| Question | Answer |
|---|---|
| Doesn't Redo already do AI returns? | "Yes, and Beehive uses it. Redo decides the return. Beehive decides whether the agent asking is allowed to ask." |
| Why not Redo's MCP server? | "That's for the merchant's own tools. A customer's agent needs its own door, with the customer's consent." |
| Prompt injection? | "The agent can only send requests. Rust checks consent, the cap and replays before anything reaches Redo." |
| Stolen token? | "Tokens expire fast and each one has a unique id, which blocks replays. DPoP is next." |
| Muse approved its own consent? | "Yes. The brief left out customer sign-in. PAP's OAuth sign-in closes that, and it's first on my list." |
| Why does Redo show $0.00? | "Redo's import has no field for price or refund type. I logged it as a finding. My dashboard shows the real $36." |
| Is PAP real? | "It's a draft from Sierra and Meta, Oct 2026. I built a subset: discovery, sessions with consent, filing returns." |
| Why Rust? | "It's on the money path. It has to be correct and testable." |

## Before you go up (5 minutes)

1. Open https://32-189-214-147.sslip.io and click **Reset demo**. The live state is shared, so someone else may have changed it.
2. Zoom the browser to about 125% so the cards are readable from the table.
3. Keep the backup recording open in a second tab.
4. Don't run the agent script live. Approving consent eats about 30 seconds you don't have. The pre-loaded opening already shows the same three results.
5. After rehearsing the tamper step, click **Reset demo** again.
