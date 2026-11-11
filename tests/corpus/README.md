# Test corpus

Scripts that the integration tests read with `include_str!`. **Do not edit these files**: they are byte-exact
copies, and the tests pin the importer's digests for them. The stored files use LF line endings, and the repository sets no
attribute that would change them. A checkout that converts line endings still passes the tests, because scheme 1
ignores line endings (`SPEC.md` 11.7, G1).

Every file here is original to the Twopop project, or used under terms that allow redistribution. Record each new
file's source and licence in the table below.

## `twopop/`

Three scripts from the Python importer's captured fixtures, used to prove that scheme 1 agrees with the importer
(`HASH1-21`, `HASH1-22`, in `tests/hash_v1_digest.rs`; `SPEC.md` 11.7). Issue #15 will extend the corpus.

| File | Source | Licence | Checked by |
|---|---|---|---|
| `twopop/edge_cases.fountain` | Twopop Studio, `tests/oracle/fountain/edge_cases.fountain`; authored in the Python oracle's capture script | MIT OR Apache-2.0, original to the Twopop project | `HASH1-21` |
| `twopop/two_scene.fountain` | Twopop Studio, `tests/oracle/fountain/two_scene.fountain`; copied from the Python oracle's `tests/fixtures/two_scene.fountain` | MIT OR Apache-2.0, original to the Twopop project | `HASH1-22` |
| `twopop/blue_rev2.fountain` | Twopop Studio, `tests/oracle/fountain/blue_rev2.fountain`; authored in the Python oracle's capture script | MIT OR Apache-2.0, original to the Twopop project | `HASH1-22` |

The expected `hash_v1` values in the tests are the importer's stored `text_sha` values, taken from Twopop Studio's
`tests/oracle/parse/<name>.json`, field `.twopop.scenes[].text_sha`, where `<name>` is the file's name without
`.fountain`.
