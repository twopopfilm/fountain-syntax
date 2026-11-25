# Security policy

fountain-syntax parses text it does not control, so a malicious or malformed script is the realistic threat. The
crate promises that `parse` never fails or panics, and a report that breaks that promise is a security report.

## Supported versions

Only the latest released 0.x minor version gets fixes. A fix ships as a new patch release.

| Version | Supported |
|---|---|
| 0.1.x | Yes |
| Older | No |

## Reporting a vulnerability

Report privately through GitHub: open the repository's Security tab, choose "Report a vulnerability", or go straight
to <https://github.com/twopopfilm/fountain-syntax/security/advisories/new>. Don't open a public issue or pull
request for a vulnerability.

Please include:

- the crate version or commit;
- the input that triggers the problem (attach the file, or minimise it if you can);
- what happens: the panic message, or the time or memory you measured.

## What counts

- A panic, a hang, or unbounded or disproportionate time or memory use caused by the input text, through any public
  API: `parse`, the typed view, the writer, the edits and the hashes.
- Any `unsafe` code. The crate forbids it (`unsafe_code = "forbid"`).

Not a vulnerability:

- A panic that a function documents under `# Panics`, such as `Document::with_replaced`, `Document::source_hash` or
  `Scene::source_hash` given a span that is out of range, inverted or not on a character boundary. That is a caller
  error; report it as a bug in a public issue.
- The hashes panicking on a string over 4 GiB (a count or length over `u32::MAX`). `SPEC.md` allows it, since no real
  screenplay is that large.
- A wrong but bounded parse result. Report it as a bug in a public issue.

## What to expect

This is a single-maintainer project, so every time below is best effort:

- an acknowledgement within 7 days;
- a first assessment within 30 days;
- a fix and a disclosure within 90 days, as the aim.

You will be kept informed, and credited in the advisory if you want to be.

## How fixes are announced

A fix is published as a GitHub security advisory and a new patch release. A serious issue also gets a RustSec
advisory.
