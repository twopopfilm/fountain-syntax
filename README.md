# fountain-syntax

**A lossless, spec-complete parser and writer for [Fountain](https://fountain.io) screenplays, in Rust.**

Fountain is the plain-text markup screenwriters use to write scripts in any text editor. `fountain-syntax` reads it
into a syntax tree that keeps every byte of the source, with a typed screenplay view on top. A writer puts it
back out unchanged, and writes a screenplay built in memory as canonical Fountain. That makes it a foundation for
editors, linters, converters and production tools that need to change one detail of a script without disturbing the
rest.

> **Status: pre-alpha.** The lossless syntax tree exists: `parse` turns any text into a tree with a span on every
> node whose tokens reproduce the source exactly, and it covers every block element, including the title page.
> `Document::screenplay()` builds the typed view: the title page, the scenes and their elements, including
> dialogue lines, parentheticals and dual-dialogue pairs. `Document::diagnostics()` reports the 22 diagnostic
> codes `SPEC.md` §7 defines. With the `hash` feature, the scheme 2 hashes work: element, scene, document, title page
> and source spans, and the importer-compatible scheme 1 hashes (`Scene::hash_v1`) reproduce the Twopop importer's
> values on its captured scripts. `Document::write` gives the source back byte for byte, and `canonical` writes a
> view built in memory as Fountain that parses back to the same view (the shapes it cannot write back are listed in
> `SPEC.md` §10.3). Targeted edits work: `with_replaced` and the named edits (`with_scene_number`, `with_synopsis`,
> `with_note` and `with_cue_renamed`). The crate is not on crates.io. Milestone F1, the
> standards study, is done ([`SPEC.md`](SPEC.md)). F4 is in progress: the conformance corpus and the comparison with
> `screenplay-tools` are in place, and some rulings are still open. F2 and F3 are in progress. The usage examples are
> compiled and run as tests, and [`CHANGELOG.md`](CHANGELOG.md) lists what works so far. The goals below describe the
> planned 0.1 release. See the [roadmap](#roadmap).

## Usage

Every example here is compiled and run by `cargo test`, in the default build and with all features.

### Parse

`parse` never fails. The tree keeps every byte, so its source is the input.

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

### Inspect

`screenplay()` builds the typed view, and `diagnostics()` lists the cases the spec calls ambiguous, each with a code.
Neither changes the tree.

```rust
use fountain_syntax::{DiagnosticCode, Element, parse};

let doc = parse("Title: Big Fish\n\nINT. HOUSE - DAY\n\nMARA\nHello.\n\nEXT. YARD - NIGHT\n\nBob waits.\n");
let screenplay = doc.screenplay();

let title = screenplay.title_page.as_ref().and_then(|page| page.title());
assert_eq!(title.unwrap().values[0].plain(), "Big Fish");

assert_eq!(screenplay.scenes.len(), 2);
assert_eq!(screenplay.scenes[0].heading.text.plain(), "INT. HOUSE - DAY");
assert_eq!(screenplay.scenes[1].heading.text.plain(), "EXT. YARD - NIGHT");
let Element::Dialogue(block) = &screenplay.scenes[0].elements[0] else { panic!("a cue") };
assert_eq!(block.cue.name.plain(), "MARA");
assert!(doc.diagnostics().is_empty());

// A note that is never closed is kept as text and reported.
let doc = parse("INT. HOUSE - DAY\n\nBob [[walks in.\n");
assert_eq!(doc.diagnostics()[0].code, DiagnosticCode::UnclosedNote);
```

### Edit

A named edit changes one span of the source and re-parses. It returns an `EditError` instead of an edit that would not
read as what it was meant to be (`EditError::Invalid`), or `EditError::NotFound` for a span that is not an element of
the kind the edit needs. The original `Document` is not changed.

```rust
use fountain_syntax::{Element, parse};

let doc = parse("INT. HOUSE - DAY\n\nMARA\nHello.\n");
let screenplay = doc.screenplay();

let Element::Dialogue(block) = &screenplay.scenes[0].elements[0] else { panic!("a cue") };
let renamed = doc.with_cue_renamed(block.cue.span.unwrap(), "JUNE").unwrap();
assert_eq!(renamed.source(), "INT. HOUSE - DAY\n\nJUNE\nHello.\n");

// After an edit, take spans from the edited document, not the old one (SPEC.md section 10.2).
let after = renamed.screenplay();
let heading = after.scenes[0].heading.span.unwrap();
let numbered = renamed.with_scene_number(heading, "12A").unwrap();
assert_eq!(numbered.source(), "INT. HOUSE - DAY #12A#\n\nJUNE\nHello.\n");

assert_eq!(doc.source(), "INT. HOUSE - DAY\n\nMARA\nHello.\n");
```

### Write

`write` gives the source back byte for byte, whatever its line endings or forced elements. `canonical` writes a view
as Fountain, with a sigil only where one is needed and LF line endings, so its text can differ from the source.

```rust
use fountain_syntax::{canonical, parse};

// CRLF line endings, a forced Action (`!`) and a forced scene heading (`.`).
let src = "INT. HOUSE - DAY\r\n\r\n!QUIET\r\n\r\n.roof\r\n";
let doc = parse(src);
assert_eq!(doc.write(), src);

let view = doc.screenplay();
let text = canonical(&view);
assert_eq!(text, "INT. HOUSE - DAY\n\nQUIET\n\n.roof\n");
assert_eq!(canonical(&parse(&text).screenplay()), text);
```

### Features and hashes

The Cargo features `locked-scene-numbers` and `shot` exist and change how the tree reads those two extensions;
`locked-scene-numbers` also adds `Document::with_locked_scene_number`.
The `serde` feature exists too: a `Document` serialises as its source text and is parsed again when it is read back.
The `hash` feature adds the scheme 2 hashes and the scheme 1 hash of a scene (and `sha2` and `unicode-normalization`):

```rust
use fountain_syntax::parse;

let doc = parse("INT. HOUSE - DAY\n\nBob walks in.\n");
let screenplay = doc.screenplay();
assert_eq!(screenplay.scenes.len(), 1);
// The methods below need the `hash` feature.
#[cfg(feature = "hash")]
{
    // A self-describing digest: `fs2:` plus 64 hexadecimal digits.
    assert!(screenplay.hash().to_string().starts_with("fs2:"));
    // A scene has one hash per element, to see which element changed.
    assert_eq!(screenplay.scenes[0].element_hashes().len(), 1);
    // The importer-compatible scheme 1 hash of a scene is the bare 64 digits.
    assert_eq!(screenplay.scenes[0].hash_v1().to_string().len(), 64);
}
```

The text that scheme 1 hashes is available in every build, without the `hash` feature: `Scene::normalized_text_v1`,
`heading_line_v1` and `element_lines_v1`.

## Goals

- **Lossless.** A concrete syntax tree with a source span on every node. Its tokens reproduce the input
  exactly, including whitespace, notes, boneyard and odd formatting. `Document::write` gives it back byte for byte.
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
  `Document::diagnostics()` reports ambiguous cases instead of failing, each with a code, a severity and a span, and
  never changes the tree.
- **A typed view.** Title page, scenes and elements as plain Rust types, with optional `serde` support for both
  the tree and the view.
- **Targeted edits.** Set a scene number as `#12A#` (`with_scene_number`), set a `= synopsis` line (`with_synopsis`),
  add a `[[note]]` block (`with_note`) or rename a character cue (`with_cue_renamed`), and only that text changes.
  The named edits return an `EditError` instead of an edit that would not parse as what it was meant to be;
  `with_replaced` replaces any span, checks nothing, and panics on a span that is out of range
  or not on character boundaries.
- **Draft comparison.** With the `hash` feature, self-describing scheme 2 hashes at three levels (element, scene and
  document) and over the exact bytes of any span, so tools can match scenes across drafts, tell real changes from
  whitespace and see which element changed. A second scheme, compatible with the Twopop importer (`Scene::hash_v1`,
  scenes only), lets hashes it stored be checked and migrated.
- **Small and safe.** No `unsafe`. The core needs no dependencies. The optional `serde` feature adds `serde`, and the
  optional `hash` feature adds `sha2` and `unicode-normalization` (for the hashes). Both features exist. A WASM build
  for a browser playground is deferred until after 0.1. It builds on Rust 1.85 or later.

## Correctness

Fountain's [official syntax](https://fountain.io/syntax) leaves real cases open, and apps disagree on them. This
crate treats correctness as something to measure:

- **A written spec first.** Milestone F1 studied the official syntax and the extensions apps use in practice, and
  ended in [`SPEC.md`](SPEC.md). It states what is standard, which extensions are supported, and how each ambiguous
  case is decided.
- **Every decision is tested.** Each ruling in [`SPEC.md`](SPEC.md) has a test ID that the test will carry in its
  name. The tests for the syntax tree, the typed view (elements, scenes, dialogue and `serde`), the diagnostics and
  both hash schemes are written, which is every HASH1 and HASH2 test, and so are the writer's WR-1 to WR-13 and the
  corpus's OFF rows. Two extension IDs, `EXT-contd-2` and `EXT-locked-scene-numbers-4`, have no test that carries
  their name yet.
- **A conformance corpus.** `tests/corpus/` holds at least one original script for each row of the decision table in
  [`SPEC.md`](SPEC.md) section 6.2, plus Twopop's own fixtures, each with its source and licence recorded. The spec's
  text has no licence, so it is not copied, and sample scripts would need written permission. Every script round-trips
  byte-for-byte through the writer (`WR-1`).
- **Differential testing.** The typed view of every corpus script is compared with the reading of
  [`screenplay-tools`](https://pypi.org/project/screenplay-tools/) 0.0.10, an established Fountain parser. Every
  difference is classified as theirs-wrong or a documented choice in
  [`docs/conformance/screenplay-tools.md`](docs/conformance/screenplay-tools.md), and `cargo test` fails on one that
  is not recorded. The test reads `tests/corpus/screenplay-tools.json`, a committed snapshot of their output, so it
  needs neither Python nor a network. `uv run --script scripts/screenplay-tools-diff.py` regenerates the snapshot after
  a corpus or parser change, and needs `uv` and a network. The test is skipped under `--all-features`, because the
  `shot` and `locked-scene-numbers` features change the view.

## Roadmap

Work is tracked on the [Twopop Studio project board](https://github.com/orgs/twopopfilm/projects/2). Goals and the
"done when" checks are in [docs/brief.md](docs/brief.md).

| Milestone | Scope                                                                              | State       |
|-----------|------------------------------------------------------------------------------------|-------------|
| F1        | Standards study: the official syntax, extensions, `SPEC.md`                        | **Done**    |
| F2        | Lossless parser, typed view, diagnostics, hashes (both schemes done)               | In progress |
| F3        | Writer with byte-for-byte round-trip; targeted edit API                            | In progress |
| F4        | Conformance corpus and differential testing                                        | In progress |
| F5        | Docs, compiled examples, and the 0.1 release on crates.io                          | In progress |

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
