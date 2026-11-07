---
paths:
  - "**/*.{ts,tsx}"
---

# React + TypeScript Rules

Shared by the Next and Vite packs.

- `[RX-01]` Strict TypeScript. No `any`; use `unknown` and narrow.
- `[RX-02]` pnpm only, exact versions (`pnpm add -E`).
- `[RX-03]` Tailwind + shadcn/ui. Add components with `pnpm dlx shadcn@latest add [name]`; don't hand-write what shadcn has.
- `[RX-04]` Theme lives in CSS variables (shadcn tokens). No hardcoded colors in components.
- `[RX-05]` Function components and hooks. Server data through one fetch layer, not scattered `fetch` calls.
- `[RX-06]` Validate anything from outside (model output, API responses, forms) with zod.
- `[RX-07]` Errors show in the UI as a friendly message (shadcn `Alert` or a toast), and log the real error to the console.
- `[RX-08]` Descriptive names. `user`, not `u`; `error`, not `e`.
- `[RX-09]` Mobile first. Check 375px before calling a screen done.
- `[RX-10]` Generated types from `schema/` live in `src/types/generated/` and are never edited by hand.
