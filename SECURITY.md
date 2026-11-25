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

- A panic, a hang, or unbounded or disproportionate time or memory use on any input, through any public API: `parse`,
  the typed view, the writer, the edits and the hashes.
- Any `unsafe` code. The crate forbids it (`unsafe_code = "forbid"`).

A wrong but bounded parse result is not a vulnerability. Report it as a bug in a public issue.

## What to expect

This is a single-maintainer project, so every time below is best effort:

- an acknowledgement within 7 days;
- a first assessment within 30 days;
- a fix and a disclosure within 90 days, as the aim.

You will be kept informed, and credited in the advisory if you want to be.

## How fixes are announced

A fix is published as a GitHub security advisory and a new patch release. A serious issue also gets a RustSec
advisory.
