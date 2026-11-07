# Beehive — Brand & Architecture Reference
**Crimson Hacks 2026 · Vite + React + Rust (Axum) · Personal Agent Protocol (PAP) Gate for Redo**

---

## 1. Core Identity & Positioning
* **One-Liner:** Beehive is a plugin for Redo that lets a customer's own AI agent handle returns and claims through the Personal Agent Protocol (PAP). It checks who the agent is and what the customer allowed, then hands the work to Redo.
* **Positioning (Plugin, Not Competitor):** *"Redo's MCP server lets the merchant's AI in. Beehive opens the other door: the customer's own AI agent. Redo's review and fraud agents still make the call."*
* **Brand Archetype:** *Utah Craft meets Cryptographic Proof* — Warm, local Utah honey farm aesthetics (**Cache Valley Honey Co.**) paired with high-contrast, deterministic Rust security and SHA-256 hash-chain verification.
* **Tagline:** *The Personal Agent Gate for Redo Merchants.*

---

## 2. Color Palette & Semantic Tokens

| Token Name | Hex | Tailwind Class | CSS Variable | Usage in Dashboard |
| :--- | :--- | :--- | :--- | :--- |
| **Basalt Slate** | `#0F172A` | `bg-slate-900` | `--bg-slate` | Primary app background (matches terminal demo) |
| **Hive Surface** | `#1E293B` | `bg-slate-800` | `--card-slate` | Table cards, modals, and SVG inner fills |
| **Comb Amber** | `#F59E0B` | `amber-500` | `--comb-amber` | Primary brand mark, active PAP session badges |
| **Wildflower Gold**| `#FDE68A` | `amber-200` | `--wildflower-gold`| Customer consent cap pills (`$50.00 CAP`), highlights |
| **Iron Rust** | `#EA580C` | `orange-600` | `--iron-rust` | Rust gate / Axum badges, Redo handoff accents |
| **Gate Pass** | `#10B981` | `emerald-500` | `--gate-pass` | `gate: "pass"`, `risk: "LOW"`, verified SHA-256 chain |
| **Gate Block** | `#EF4444` | `red-500` | `--gate-block` | `gate: "block"` ($120 cap / replay), broken hash link |

---

## 3. Typography
* **UI & Merchant Data:** `Inter` or `Plus Jakarta Sans` — Used for merchant headers, customer names, product titles (`Beeswax Candle Set`), and dollar amounts (`$36.00`, `$120.00`).
* **Protocol & Cryptographic Data:** `JetBrains Mono` or `Fira Code` — Mandatory for `agent_id` (`muse-demo`), `request_id`, `poppy:write` scopes, and `sha256...` evidence chain heads.

---

## 4. Brand SVGs

### Concept 1: The Hex-Key Gate (`public/favicon.svg` & Navbar Logo)
Combines the Utah beehive hexagon with a cryptographic keyhole and Rust gate circuit pins:

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 120 120" width="100%" height="100%" fill="none">
  <rect width="120" height="120" rx="24" fill="#0F172A"/>
  <polygon points="60,16 98,38 98,82 60,104 22,82 22,38" stroke="#F59E0B" stroke-width="6" stroke-linejoin="round" fill="#1E293B"/>
  <line x1="22" y1="60" x2="38" y2="60" stroke="#EA580C" stroke-width="5" stroke-linecap="round"/>
  <line x1="82" y1="60" x2="98" y2="60" stroke="#EA580C" stroke-width="5" stroke-linecap="round"/>
  <circle cx="60" cy="52" r="10" fill="#FDE68A"/>
  <path d="M54 58 L66 58 L69 78 L51 78 Z" fill="#FDE68A" stroke-linejoin="round"/>
</svg>
```

### Concept 2: The Hash-Chain Comb (Architecture & Interactive Status Indicator)
Three interlocking hexagons representing **Personal Agent (Gold) → Beehive Rust Gate (Amber) → Redo (Rust)**:

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 180 100" width="100%" height="100%" fill="none">
  <rect width="180" height="100" rx="20" fill="#0F172A"/>
  <polygon points="45,24 68,37 68,63 45,76 22,63 22,37" stroke="#FDE68A" stroke-width="5" stroke-linejoin="round" fill="#0F172A"/>
  <polygon points="90,24 113,37 113,63 90,76 67,63 67,37" stroke="#F59E0B" stroke-width="6" stroke-linejoin="round" fill="#1E293B"/>
  <polygon points="135,24 158,37 158,63 135,76 112,63 112,37" stroke="#EA580C" stroke-width="5" stroke-linejoin="round" fill="#0F172A"/>
  <circle cx="45" cy="50" r="4" fill="#FDE68A"/>
  <circle cx="90" cy="50" r="5" fill="#F59E0B"/>
  <circle cx="135" cy="50" r="4" fill="#EA580C"/>
  <line x1="49" y1="50" x2="85" y2="50" stroke="#F59E0B" stroke-width="3" stroke-dasharray="4 3"/>
  <line x1="95" y1="50" x2="131" y2="50" stroke="#EA580C" stroke-width="3" stroke-dasharray="4 3"/>
</svg>
```

---

## 5. Seed Data & 3-Minute Demo Flow (`Cache Valley Honey Co.`)

### Seed Catalog
1. **Beeswax Candle Set** — `$36.00` (Demo Step 1: Passes `$50` cap → Redo `LOW` risk → Approved + Label)
2. **Honeycomb Gift Crate** — `$120.00` (Demo Step 2: Blocked by Rust Gate — Exceeds `$50` customer cap)
3. **Honey Subscription Box** — `$28.00/mo`
4. **Raw Wildflower Honey** — `$14.00`

### Live Demo Script Checklist
* [ ] **1. Discovery & Session (`0:25`):** Personal agent `muse-demo` reads `/.well-known/poppy.json`, starts a session at `POST /poppy/session` with `poppy:write` and a `$50.00` refund cap.
* [ ] **2. Valid Return (`0:45`):** Agent files `$36.00` return for *Beeswax Candle Set* (`damaged`). Rust gate passes it → Redo approves (`LOW` risk) and issues label.
* [ ] **3. Cap Block (`1:10`):** Same agent attempts `$120.00` return for *Honeycomb Gift Crate*. Rust gate blocks (`Exceeds $50.00 customer consent cap`). Never reaches Redo.
* [ ] **4. Replay Block (`1:25`):** Agent replays the `$36.00` request (`req_candle_01`). Rust gate blocks (`Duplicate request_id / token_id replay`). No double refund.
* [ ] **5. Verify & Tamper Proof (`1:35`):** Click **Verify Proof** → SHA-256 chain turns Emerald (`#10B981`). Click **Tamper 1 Byte** → Chain breaks at Link #3 (Consent/Cap payload) and flashes Crimson (`#EF4444`).