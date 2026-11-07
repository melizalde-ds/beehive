---
description: Review the build as a judge persona
argument-hint: "[architect | ux | sponsor <name>] [live URL]"
---

Persona and URL: $ARGUMENTS

Read `brief/brief.md` first. Then review as the persona. Rank issues by impact on a five-minute judging. Don't fix anything yet.

- **architect:** a staff engineer judging the repo. Structure, readability, obvious bugs, security holes (exposed keys, unvalidated input, missing auth). What would a technical judge wince at? Only what's fixable in under an hour.
- **ux:** a UX lead using the live app, including at 375px. Confusing flows, unclear copy, weak hierarchy, missing feedback. What loses a first-time user?
- **sponsor:** you are that sponsor and you wrote the track. Research what they sell and care about. Does this solve your problem? Is your tool the hero of a moment, or bolted on? What looks subtly wrong to someone in this domain? What would make you say "we need this"?
