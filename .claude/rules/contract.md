---
paths:
  - "schema/**"
---

# Contract Rules

Use when more than one language shares data (Rust or Python service + TypeScript frontend). Delete if the project is Next.js only.

- `[CT-01]` `schema/*.json` (JSON Schema) is the single source of truth for every shape that crosses a boundary.
- `[CT-02]` Generate types, never hand-copy them:
  - TypeScript: `json-schema-to-typescript` into `src/types/generated/`
  - Rust: `typify` (or `schemars` if Rust owns the shape and emits the schema)
  - Python: `datamodel-code-generator` into pydantic models
- `[CT-03]` Generated files are never edited. Change the schema, then regenerate.
- `[CT-04]` The contract is the blocking phase. Write it first, with one seeded example per schema in `schema/examples/`.
- `[CT-05]` A schema change in a parallel phase is "ask first". Other phases build against it.
