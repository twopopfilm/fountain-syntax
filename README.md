# fountain-syntax

**A lossless, spec-complete parser and writer for [Fountain](https://fountain.io) screenplays, in Rust.**

Fountain is the plain-text markup screenwriters use to write scripts in any text editor. `fountain-syntax` reads it
into a syntax tree that keeps every byte of the source, with a typed screenplay view on top. A writer that puts it
back out unchanged is planned. That makes it a foundation for editors, linters, converters and production tools
that need to change one detail of a script without disturbing the rest.

> **Status: pre-alpha.** The lossless syntax tree exists: `parse` turns any text into a tree with a span on every
> node whose tokens reproduce the source exactly, and it covers every block element, including the title page.
> `Document::screenplay()` builds the typed view: the title page, the scenes and their elements, including
> dialogue lines, parentheticals and dual-dialogue pairs. Diagnostics, the writer and the hashes are still
> planned, and the crate is not on crates.io. Milestone F1, the standards study, is done
> ([`SPEC.md`](SPEC.md)); F2 is in progress. The goals below describe the planned 0.1 release. See the
> [roadmap](#roadmap).

```rust
use fountain_syntax::{Child, NodeKind, parse};

let doc = parse("Title: Big Fish\n\nINT. HOUSE - DAY\n");
let kinds: Vec<NodeKind> = doc
    .root()
    .children()
    .filter_map(|child| match child {
        Child::Node(node) if node.kind() != NodeKind::Gap => Some(node.kind()),
        _ => None,
    })
    .collect();
assert_eq!(kinds, [NodeKind::TitlePage, NodeKind::SceneHeading]);
assert_eq!(doc.source(), "Title: Big Fish\n\nINT. HOUSE - DAY\n");
```

The Cargo features `locked-scene-numbers` and `shot` exist and change how the tree reads those two extensions.
The `serde` feature exists too: a `Document` serialises as its source text and is parsed again when it is read back.
`hash` does not exist yet.

## Goals

- **Lossless.** A concrete syntax tree with a source span on every node. Its tokens reproduce the input
  exactly, including whitespace, notes, boneyard and odd formatting. Writing it back byte-for-byte is planned.
- **Spec-complete.** Every Fountain element:
  - title page;
  - scene headings and scene numbers;
  - action, characters, dialogue, parentheticals and dual dialogue;
  - transitions, centered text and lyrics;
  - sections, synopses, notes and boneyard;
  - page breaks and forced elements.
- **Extensions where tools actually use them.** [`SPEC.md`](SPEC.md) rules on each one. Open title-page keys,
  sections of any depth, `(CONT'D)` kept as written, Beat's revision block (kept as an ordinary boneyard) and note
  conventions (an opt-in typed view) change no parse, so they are always on. Slugline's locked scene numbers (`##12A#`)
  and Beat's `!!` Shot change how some valid input reads, so each sits behind its own Cargo feature flag
  (`locked-scene-numbers` and `shot`). Everything else surveyed, such as brace directives, endnotes and stage-play
  variants, is not supported in 0.1: the text is kept and written back as it is. For the always-on extensions the tree
  keeps the text, and the typed view adds typed title-page accessors (`TitlePage::title()` and others) and the opt-in
  note-conventions reader (`Note::conventions`).
- **Never fails.** Fountain has no invalid input: any text is a screenplay. The parser always returns a tree.
  Reporting ambiguous cases as diagnostics instead of errors is planned.
- **A typed view.** Title page, scenes and elements as plain Rust types, with optional `serde` support for both
  the tree and the view.
- **Targeted edits (planned).** Set a scene number as `#12A#` or insert a `= synopsis` line, and only that text changes.
- **Draft comparison (planned).** Self-describing hashes at three levels (element, scene and document) and over the
  exact bytes of any span, so tools can match scenes across drafts, tell real changes from whitespace and see which
  element changed. A second scheme, compatible with the Twopop importer, is also available, so hashes it stored can be
  checked and migrated.
- **Small and safe.** No `unsafe`. The core needs no dependencies. The optional `serde` feature adds `serde`, and the
  optional `hash` feature adds `sha2` and `unicode-normalization` (for the hashes). The `serde` feature exists; the
  `hash` feature is planned. An optional WASM build for a browser playground is being considered.

## Correctness

Fountain's [official syntax](https://fountain.io/syntax) leaves real cases open, and apps disagree on them. This
crate treats correctness as something to measure:

- **A written spec first.** Milestone F1 studied the official syntax and the extensions apps use in practice, and
  ended in [`SPEC.md`](SPEC.md). It states what is standard, which extensions are supported, and how each ambiguous
  case is decided.
- **Every decision is tested.** Each ruling in [`SPEC.md`](SPEC.md) has a test ID that the test will carry in its
  name. The tests for the syntax tree and for the typed view (elements, scenes, dialogue and `serde`) are written;
  the rest are planned for F2 to F4.
- **A conformance corpus (planned, F4).** It will hold original examples modelled on the spec's own (the spec's text
  has no licence, so it will not be copied) and edge cases from the study, each with its source and licence recorded.
  The spec's examples and sample scripts will be used only with written permission. Once the writer exists, every
  script must round-trip byte-for-byte.
- **Differential testing (planned, F4).** Parses will be compared with `screenplay-tools`, an established Fountain
  parser. Every difference will be classified as theirs-wrong, ours-wrong or a documented choice.

## Roadmap

Work is tracked on the [Twopop Studio project board](https://github.com/orgs/twopopfilm/projects/2). Goals and the
"done when" checks are in [docs/brief.md](docs/brief.md).

| Milestone | Scope                                                       | State           |
|-----------|-------------------------------------------------------------|-----------------|
| F1        | Standards study: the official syntax, extensions, `SPEC.md` | **Done**        |
| F2        | Lossless parser, typed view, diagnostics, scene hashes      | In progress     |
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
with the script and the behaviour you expect. Guidelines for code contributions will follow when F2 is done.

## License

Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.
