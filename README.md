# fountain-syntax

**A lossless, spec-complete parser and writer for [Fountain](https://fountain.io) screenplays, in Rust.**

Fountain is the plain-text markup screenwriters use to write scripts in any text editor. `fountain-syntax` reads it
into a syntax tree that keeps every byte of the source, gives you a typed screenplay view on top, and writes it
back out unchanged. That makes it a foundation for editors, linters, converters and production tools that need to
change one detail of a script without disturbing the rest.

> **Status: pre-alpha.** There is no code yet, and the crate is not on crates.io. Milestone F1, the standards study,
> is done: [`SPEC.md`](SPEC.md) is written, and parser work (F2) starts from it. Everything below describes the
> planned 0.1 release. See the [roadmap](#roadmap).

## Goals

- **Lossless.** A concrete syntax tree with a source span on every node. Unchanged input round-trips
  byte-for-byte, including whitespace, notes, boneyard and odd formatting.
- **Spec-complete.** Every Fountain element:
  - title page;
  - scene headings and scene numbers;
  - action, characters, dialogue, parentheticals and dual dialogue;
  - transitions, centered text and lyrics;
  - sections, synopses, notes and boneyard;
  - page breaks and forced elements.
- **Extensions where tools actually use them (planned).** [`SPEC.md`](SPEC.md) rules on each one. Open title-page keys,
  sections of any depth, `(CONT'D)` kept as written, Beat's revision block (kept as an ordinary boneyard) and note
  conventions (an opt-in typed view) change no parse, so they are always on. Slugline's locked scene numbers (`##12A#`)
  and Beat's `!!` Shot change how some valid input reads, so each sits behind its own Cargo feature flag. Everything
  else surveyed, such as brace directives, endnotes and stage-play variants, is not supported in 0.1: the text is kept
  and written back as it is.
- **Never fails.** Fountain has no invalid input: any text is a screenplay. The parser always returns a tree, and
  reports ambiguous cases as diagnostics instead of errors.
- **A typed view.** Title page, scenes and elements as plain Rust types, with optional `serde` support for both
  the tree and the view.
- **Targeted edits.** Lock a scene number as `#12A#` or insert a `= synopsis` line, and only that text changes.
- **Draft comparison.** Stable per-scene text normalisation and hashes, so tools can match scenes across drafts and
  tell real changes from whitespace.
- **Small and safe.** No `unsafe`. The core needs no dependencies beyond the optional `serde`. An optional WASM build
  for a browser playground is being considered.

## Correctness

Fountain's [official syntax](https://fountain.io/syntax) leaves real cases open, and apps disagree on them. This
crate treats correctness as something to measure:

- **A written spec first.** Milestone F1 studied the official syntax and the extensions apps use in practice, and
  ended in [`SPEC.md`](SPEC.md). It states what is standard, which extensions are supported, and how each ambiguous
  case is decided.
- **Every decision is tested.** Each ruling in [`SPEC.md`](SPEC.md) has a test ID that the test will carry in its
  name. The tests themselves are planned for F2 to F4.
- **A conformance corpus.** It holds original examples modelled on the spec's own (the spec's text has no licence, so
  it is not copied) and edge cases from the study, each with its source and licence recorded. The spec's examples
  and sample scripts are used only with written permission. Every script must round-trip byte-for-byte.
- **Differential testing.** Parses are compared with `screenplay-tools`, an established Fountain parser. Every
  difference is classified as theirs-wrong, ours-wrong or a documented choice.

## Roadmap

Work is tracked on the [Twopop Studio project board](https://github.com/orgs/twopopfilm/projects/2). Goals and the
"done when" checks are in [docs/brief.md](docs/brief.md).

| Milestone | Scope                                                       | State           |
|-----------|-------------------------------------------------------------|-----------------|
| F1        | Standards study: the official syntax, extensions, `SPEC.md` | **Done**        |
| F2        | Lossless parser, typed view, diagnostics, scene hashes      | Planned         |
| F3        | Writer with byte-for-byte round-trip; targeted edit API     | Planned         |
| F4        | Conformance corpus and differential testing                 | Planned         |
| F5        | Docs, compiled examples, and the 0.1 release on crates.io   | Planned         |

Final Draft (FDX) interchange is planned as a separate crate after 0.1.

## Used by

[Twopop Studio](https://github.com/twopopfilm/twopop-studio) is a local-first production studio for AI-generated
film. It imports screenplays through `fountain-syntax`. The crate has no dependency on it and is meant to be
useful on its own.

## Contributing

The most valuable contributions right now are real-world Fountain. Send edge cases that apps handle differently, or
scripts you can share under a licence that allows redistribution (original, public-domain or CC0). Open an issue
with the script and the behaviour you expect. Guidelines for code contributions will follow once the parser exists (F2).

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
