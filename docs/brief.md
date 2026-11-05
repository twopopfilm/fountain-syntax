# fountain-syntax: brief

A Rust library for parsing and writing [Fountain](https://fountain.io) screenplays, written from scratch. It was
started for [Twopop Studio](https://github.com/twopopfilm/twopop-studio), which imports scripts through it, and is
meant to be useful on its own.

## Why it exists

- **No maintained, complete Rust parser exists** (as surveyed in 2026):
  - `fountain` (fountain-rs) covers part of the spec and was last updated in 2019;
  - `fountain_parser_rs` is alpha and handles only visible elements;
  - `jumpcut` is a converter to FDX/HTML;
  - `rustwell` (0.1, December 2025) is new and formatting-focused.
- **Tools that edit scripts need more than a typical parser gives:**
  - **Lossless parsing with source spans.** An editor can highlight, and a program can rewrite one detail without
    disturbing the rest of the text. Examples: locking scene numbers as `#12A#`, inserting a `= synopsis` line.
  - **Every element type:** scene headings and scene numbers, action, character, dialogue, parentheticals, dual
    dialogue, transitions, centered text, lyrics, sections, synopses, notes, boneyard, page breaks, the title page and
    forced elements.
  - **Stable per-scene text normalisation and hashes**, so a tool can compare script drafts scene by scene.
  - **A parser that never fails.** Fountain has no invalid input, so ambiguous cases are reported as diagnostics.

## Research first, then the spec (milestone F1)

The first milestone is a written standards study:
- **The official syntax:** fountain.io/syntax. Version 1.1 (2014) is the current revision, confirmed on
  2026-11-04. The study is in [research/official.md](research/official.md).
- **The reference parser:** how the Objective-C reference implementation treats each open case, as evidence for
  `SPEC.md`: [research/reference-parser.md](research/reference-parser.md).
- **Extensions apps use in practice:** Beat, Highland, Slugline, Afterwriting, Fade In's Fountain export,
  screenplay-tools. Examples are revision marks (the spec lists "marking changes" as a future goal), notes
  conventions and stage-play variants. The survey is in [research/extensions.md](research/extensions.md).
- **Interchange:** Final Draft FDX (a separate crate later). The mapping notes are in
  [research/extensions.md](research/extensions.md#fdx-mapping-notes).

The study ends in [`SPEC.md`](../SPEC.md). It states what the library treats as standard and which extensions it
supports (some always on, two behind feature flags), and records every decision on an ambiguous case with a test ID
for each.

## Proposed design (refined by the study)

- A lossless syntax tree with spans, plus a typed screenplay view (title page, scenes, elements); serde for both.
- A Fountain writer that round-trips byte-for-byte when nothing has changed.
- An optional WASM build for a web playground.
- MIT/Apache-2.0.

## Conformance

The test corpus is built from:
- fountain.io's sample scripts (only with written permission; see research/official.md);
- original examples modelled on the spec's own (the spec's text is copyrighted and has no licence; see
  research/official.md);
- edge cases from the study;
- Twopop's fixtures.

It is diffed against `screenplay-tools` output (a Python library, used as an oracle only, not ported). Every
difference is classified as theirs-wrong, ours-wrong or a documented choice. Check the licence of any other
project's test files before reusing them.

## Milestones

- **F1:** standards study → `SPEC.md`.
- **F2:** lossless parser and typed view.
- **F3:** writer and round-trip.
- **F4:** conformance corpus and `screenplay-tools` diff.
- **F5:** publish 0.1 to crates.io (on the maintainer's OK).

## Done when

- Every corpus script round-trips byte-for-byte.
- Every `SPEC.md` decision has a passing test.
- Diffs against `screenplay-tools` are all classified.
- Scene hashes are stable across whitespace-only edits.
