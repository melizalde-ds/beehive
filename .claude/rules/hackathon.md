# Hackathon Rules

These win over any stricter rule in other files. This is a demo, not production.

## Scope

- `[HACK-01]` The rubric decides where hours go. Read it before the brief.
- `[HACK-02]` The brief is the spec. Take its limits literally; they exist to cut work. Never build what it puts out of scope.
- `[HACK-03]` If a feature doesn't move a rubric line or the demo path, skip it.
- `[HACK-04]` Build the demo path end to end first, then widen. A thin flow that works beats a wide one that doesn't.
- `[HACK-05]` Simplest thing that demos well. No generic frameworks, config systems or abstractions for later.
- `[HACK-06]` Seed or hardcode data when a real integration would eat the clock. Make the seam obvious.

## Testing

- `[HACK-07]` No tests for coverage. Keep tests that already exist.
- `[HACK-08]` A small test is fine for tricky pure logic (scoring, parsing, date math) when it's faster than checking by hand.
- `[HACK-09]` Verify by walking the demo (see "Walk the demo" in AGENTS.md). Say what you clicked and what you saw.

## Code

- `[HACK-10]` Readable code still matters. Judges open the repo.
- `[HACK-11]` No secrets in the repo.
- `[HACK-12]` The app stays runnable and deployable at all times.

## Demo

- `[HACK-13]` Every demo-path screen has a loading, empty and friendly error state.
- `[HACK-14]` Phone width must not look broken. Judges open links on phones.
- `[HACK-15]` Realistic seed data. No "test test" or lorem ipsum on the demo path.
- `[HACK-16]` The wow moment in `brief/brief.md` gets protected first: seeded, cached, and working cold.
- `[HACK-17]` `README.md` always has the one-line identity, live link and demo steps.

## Last mile

- `[HACK-18]` Audit against the brief in a fresh session (`/audit`).
- `[HACK-19]` Persona reviews: architect, UX lead, the sponsor (`/persona`).
- `[HACK-20]` UI polish hours follow the rubric's design weight (`/polish`).
- `[HACK-21]` Ship a live URL the judge can touch. Record a backup run too.
- `[HACK-22]` Fill every submission field before the deadline. A rough submitted project beats a great unsubmitted one.
