# Test corpus

Scripts that the integration tests read: `WR-1` lists the directory, and other tests use `include_str!` or
`corpus_files()`. **Do not edit the `twopop/` files**: they are byte-exact copies, and the tests pin the importer's
digests for them. The `off/` files are original to this project, neither copies nor digest-pinned: you may edit one, as
long as its row's test and the round-trip test (`WR-1`) still pass. The root `.gitattributes` sets
`tests/corpus/** -text`, so line endings, trailing whitespace and a missing final newline survive any checkout:
the `off/` scripts for the line-ending rows (`OFF-2a`, `OFF-2b`, `OFF-23e`) keep their CR and CRLF bytes and their
missing final newline on every platform. The `twopop/` files are stored with LF line endings, and the hash tests would
pass even if a checkout converted them, because scheme 1 ignores line endings (`SPEC.md` 11.7, G1).

Every file here is original to the Twopop project, or used under terms that allow redistribution. Record each new
file's source and licence in the table below.

## `twopop/`

Three scripts from the Python importer's captured fixtures, used to prove that scheme 1 agrees with the importer
(`HASH1-21`, `HASH1-22`, in `tests/hash_v1_digest.rs`; `SPEC.md` 11.7) and to prove that the writer gives each back
byte for byte (`WR-1`, in `tests/writer.rs`; `SPEC.md` 10.4).

| File | Source | Licence | Checked by |
|---|---|---|---|
| `twopop/edge_cases.fountain` | Twopop Studio, `tests/oracle/fountain/edge_cases.fountain`; authored in the Python oracle's capture script | MIT OR Apache-2.0, original to the Twopop project | `HASH1-21, WR-1` |
| `twopop/two_scene.fountain` | Twopop Studio, `tests/oracle/fountain/two_scene.fountain`; copied from the Python oracle's `tests/fixtures/two_scene.fountain` | MIT OR Apache-2.0, original to the Twopop project | `HASH1-22, WR-1` |
| `twopop/blue_rev2.fountain` | Twopop Studio, `tests/oracle/fountain/blue_rev2.fountain`; authored in the Python oracle's capture script | MIT OR Apache-2.0, original to the Twopop project | `HASH1-22, WR-1` |

The expected `hash_v1` values in the tests are the importer's stored `text_sha` values, taken from Twopop Studio's
`tests/oracle/parse/<name>.json`, field `.twopop.scenes[].text_sha`, where `<name>` is the file's name without
`.fountain`.

## `off/`

One script per `OFF` row of the decision table (`SPEC.md` 6.2), named by the row: `off/off-<n><letter>.fountain`, in
lowercase and without zero-padding. Each script keeps its first block an ordinary Action paragraph (plain prose, no
colon, no sigil), so that a diff of two parsers finds the difference in the row's case and not in the start of the
file.

| File | Source | Licence | Checked by |
|---|---|---|---|
| `off/off-*.fountain` | Written for this project, issue #15 | MIT OR Apache-2.0, original to the Twopop project | `WR-1`, `tests/corpus.rs` |

The exceptions are the rows about the start of the document, whose case has to be the first block. They are
`OFF-6e`, `OFF-15f` (a Note in the title page) and `OFF-17a` to `OFF-17f`. The list is `START_OF_DOCUMENT` in
`tests/corpus.rs`, which skips the first-block check for them.
