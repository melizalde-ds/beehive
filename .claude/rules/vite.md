---
paths:
  - "vite.config.*"
  - "src/**/*.{ts,tsx}"
---

# Vite React Pack

Delete this file if the project uses Next.js. Use this pack when a Rust or Python service owns the logic and the frontend is a dashboard.

- `[VT-01]` Single-page app. Routing with React Router only if there is more than one screen.
- `[VT-02]` All server data through TanStack Query, one hook per endpoint in `src/api/`.
- `[VT-03]` No secrets in the client. Every model call goes through the backend service.
- `[VT-04]` Backend URL comes from `VITE_API_URL`. Streaming uses SSE.
- `[VT-05]` Every live view has a replay or seeded mode, so the demo runs without the backend under load.
- `[VT-06]` Deploy the static build on Vercel (or any static host). The backend URL is set per environment.
