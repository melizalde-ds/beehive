---
paths:
  - "**/*.py"
  - "**/pyproject.toml"
---

# Python AI Service Pack

- `[PY-01]` uv for everything: `uv add`, `uv run`. No pip installs into the system.
- `[PY-02]` FastAPI. Request and response bodies are pydantic models.
- `[PY-03]` Model output is parsed into a pydantic model before use. Invalid: retry once, then return a friendly error.
- `[PY-04]` Prompts live in `prompts.py`.
- `[PY-05]` Type hints on every function. `ruff check` and `ruff format` clean before you say done.
- `[PY-06]` Long jobs stream with SSE or return a job id. Never block a request for minutes.
- `[PY-07]` Notebooks are for exploring only. Anything the demo runs lives in a module.
