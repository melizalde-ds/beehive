# AI Rules

The model sits at the edges, never in the middle.

```
messy input → [model] → structured data → plain code decides → [model] → explanation
```

- `[AI-01]` Use a model for messy input to structured data (a sentence into fields, a document into values).
- `[AI-02]` Use a model for structured data to text (an explanation, a summary, a draft).
- `[AI-03]` Scoring, rules, filters, ranking, math and validation are plain code. No model in that path.
- `[AI-04]` Already structured? Use code. Parsing, dates and file generation use libraries.
- `[AI-05]` Ask for JSON and validate it against a schema or type before use. Retry once, then show a friendly error.
- `[AI-06]` Call models from the server, never the client. Keys live in `.env`.
- `[AI-07]` Prompts live in their own file per pack (see the stack rules), not inline in logic.
- `[AI-08]` Cache or seed AI results on the demo path. A slow or failed call can't break the wow moment.
- `[AI-09]` Where it's cheap, give each AI input a non-AI path into the same logic (a form next to the "describe it" box).
- `[AI-10]` Be ready to say which model you used and how you check its output.
