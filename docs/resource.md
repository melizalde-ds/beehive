# Resources

What we built on and cite, plus where our findings for sponsors live. The README's citations come from here.

## PAP (Personal Agent Protocol, "Poppy")

- [Sierra: Poppy announcement](https://sierra.ai/blog/poppy): the primary source. Discovery at `/.well-known/poppy.json`, guest sessions, sign-in through OAuth with a session token, and the Muse + Rocket demo at Sierra Summit.
- [Descope: Personal Agent Protocol explainer](https://www.descope.com/learn/post/personal-agent-protocol): third-party summary. It mentions CIMD and DPoP, which the Sierra posts don't.
- PAP is a draft, and Sierra says its reference implementation is still to come. Beehive builds a subset (see `brief/brief.md`).

## PAP client SDK (datalayer, Python)

- [PyPI: personal-agent-protocol](https://pypi.org/project/personal-agent-protocol/): `DiscoveryClient().discover(domain)`. Used only in stretch S1 (`plans/stretch-muse.md`).
- [0.30.0](https://pypi.org/project/personal-agent-protocol/0.30.0/) targets draft 0.3 (2026-10-30). Early releases such as [0.3.0](https://pypi.org/project/personal-agent-protocol/0.3.0/) target draft 0.1, so pin the version.
- It doesn't provide production signing keys, token storage, DPoP proofs or browser handoff.

## Redo

- [Redo API docs for agents (`llms.txt`)](https://developers.redo.com/llms.txt)
- [Redo OpenAPI spec](https://developers.redo.com/api-schema/openapi.yaml), with a local copy at `docs/redo/openapi.yaml`
- API base: `https://api.getredo.com/v2.2`
- How the real API behaves, tested against a real store: `docs/redo-integration.md`, with responses in `redo-fixtures/`

## Findings to report

- `docs/findings.md`: every sponsor finding, with a pitch line for the table. Newest first.
- `docs/redo-findings.md`: the report to hand Redo, with bugs, steps, return ids and questions.
