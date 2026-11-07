---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---

# Rust Service Pack

- `[RS-01]` axum + tokio. One binary unless the brief needs more.
- `[RS-02]` serde for all payloads. Shared shapes come from `schema/` (see `contract.md`).
- `[RS-03]` Errors: `thiserror` in library code, `anyhow` at the edges. Handlers map errors to a JSON body with a friendly message.
- `[RS-04]` `tracing` for logs. No `println!` outside quick scripts.
- `[RS-05]` No `unwrap()` or `expect()` on the request path. Fine in setup code and tests.
- `[RS-06]` Streaming to the frontend uses SSE.
- `[RS-07]` Prompts live in `src/prompts.rs`. Model output is deserialized into a typed struct before use.
- `[RS-08]` `cargo clippy` and `cargo fmt` clean before you say done.
- `[RS-09]` Config comes from env with sane defaults, so `cargo run` works with only `.env` filled in.
