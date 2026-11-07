// The personal agent: Kaylee's agent finds the shop through PAP, asks for consent,
// then files returns. It only asks; the Rust gate decides.
// Run: node agent/agent.ts   (Node 24 runs TypeScript directly, no install)

import type {
  GateRequest,
  GateResponse,
  PoppyDiscovery,
  SessionRequest,
  SessionResponse,
} from "../web/src/types/generated/beehive.ts";

const BASE = process.env.BEEHIVE_URL || "http://localhost:3000";
const AGENT_ID = "muse-demo";
const CONSENT_TIMEOUT_MS = 5 * 60_000;

async function call<T>(path: string, body?: unknown): Promise<T> {
  let res: Response;
  try {
    res = await fetch(BASE + path, {
      method: body === undefined ? "GET" : "POST",
      headers: { "content-type": "application/json" },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
  } catch {
    throw new Error(`Can't reach Beehive at ${BASE}. Is the server running?`);
  }
  const json = await res.json().catch(() => null);
  if (!res.ok) throw new Error(json?.message ?? `${path} answered ${res.status}`);
  return json as T;
}

function show(label: string, r: GateResponse) {
  const mark = r.gate === "pass" ? "PASS " : "BLOCK";
  console.log(`  ${mark} ${label}: ${r.reason}`);
  if (r.redo) console.log(`        Redo return ${r.redo.return_id} is ${r.redo.status}`);
  console.log(`        evidence ${r.return_id}, chain head ${r.chain_head.slice(0, 12)}…`);
}

async function main() {
  console.log(`1. Discover ${BASE}/.well-known/poppy.json`);
  const shop = await call<PoppyDiscovery>("/.well-known/poppy.json");
  console.log(`  ${shop.merchant.name} speaks PAP ${shop.pap_version}, returns run by ${shop.merchant.platform}`);

  console.log("2. Ask Kaylee for write access with a $50 refund cap");
  const ask: SessionRequest = {
    agent_id: AGENT_ID,
    mode: "signed_in",
    customer_email: "kaylee.jensen@example.com",
    customer_id: "c_1",
    scopes: ["poppy:read", "poppy:write"],
    refund_cap: 50,
  };
  let session = await call<SessionResponse>(shop.endpoints.session, ask);
  console.log(`  Waiting for Kaylee to approve: ${session.consent_url}`);
  const deadline = Date.now() + CONSENT_TIMEOUT_MS;
  while (session.status === "pending_consent") {
    if (Date.now() > deadline) throw new Error("Kaylee didn't answer within 5 minutes.");
    await new Promise((r) => setTimeout(r, 1000));
    session = await call<SessionResponse>(`${shop.endpoints.session}/${session.session_id}`);
  }
  if (session.status === "denied" || !session.session_token) {
    console.log("  Kaylee said no. The agent stops here.");
    return;
  }
  console.log(`  Approved: ${session.scopes.join(", ")}, cap $${session.refund_cap}, token ${session.token_id}`);

  const file = (order_id: string, reason: GateRequest["reason"], amount: number): GateRequest => ({
    request_id: crypto.randomUUID(),
    session_token: session.session_token!,
    agent_id: AGENT_ID,
    order_id,
    reason,
    amount,
  });

  console.log("3. File the returns");
  const candles = file("o_1", "damaged", 36);
  show("$36 candle set arrived broken", await call<GateResponse>(shop.endpoints.returns, candles));
  show("$120 honeycomb crate", await call<GateResponse>(shop.endpoints.returns, file("o_2", "changed_mind", 120)));
  show("replay of the candle request", await call<GateResponse>(shop.endpoints.returns, candles));
}

main().catch((e: Error) => {
  console.error(`Agent stopped: ${e.message}`);
  process.exit(1);
});
