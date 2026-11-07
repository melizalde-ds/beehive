---
paths:
  - "next.config.*"
  - "**/app/**/*.{ts,tsx}"
  - "**/middleware.ts"
---

# Next.js + Vercel Pack

Delete this file if the project uses Vite.

- `[NX-01]` App Router. Server components by default; `"use client"` only where there is state or events.
- `[NX-02]` Model and secret calls run in route handlers (`app/api/**/route.ts`) or server actions. Never in client components.
- `[NX-03]` Prompts live in `lib/prompts.ts`.
- `[NX-04]` Env: server keys without a prefix, browser values only with `NEXT_PUBLIC_`. Mirror every key in `.env.example`.
- `[NX-05]` Stream long model responses so the UI shows progress.
- `[NX-06]` Deploy on Vercel. Every PR gets a preview URL; put it in the PR body.
- `[NX-07]` Data: start with seeded JSON in `data/`. Add a database only if the demo needs writes that survive a refresh.
