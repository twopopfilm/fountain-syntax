# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), with one extra section, Deferred, for work postponed past a
release. 0.1.0 is the first release.

## [Unreleased]

## [0.1.0] - 2026-11-20

### Added

- A lossless syntax tree: `parse` never fails, puts a span on every node and keeps every byte of the source,
  including the title page and every block element.
- A typed screenplay view: `Document::screenplay()` gives the title page, the scenes and their elements, including
  dialogue, parentheticals and dual dialogue.
- Diagnostics: `Document::diagnostics()` reports the 22 codes of `SPEC.md` section 7 without changing the tree.
- A writer: `Document::write` gives the source back byte for byte, and `canonical` writes a view built in memory as
  Fountain that parses back to the same view, except for the shapes listed in `SPEC.md` section 10.3.
- Targeted edits: `Document::with_replaced` and the named edits `with_scene_number`, `with_synopsis`, `with_note` and
  `with_cue_renamed` (and `with_locked_scene_number` with the `locked-scene-numbers` feature), which return an
  `EditError` instead of an edit that would not read as intended.
- Hashes behind the `hash` feature: scheme 2 hashes of elements, scenes, the document, the title page and source spans,
  and the importer-compatible scheme 1 hash of a scene (`Scene::hash_v1`).
- Four Cargo features, all off by default: `locked-scene-numbers`, `shot`, `serde` and `hash`.
- Usage examples in the README, compiled and run by `cargo test`, and examples in the API documentation.
- Minimum supported Rust version 1.85, declared in `Cargo.toml`.

### Deferred

- A WASM build for a browser playground, until after 0.1.

[Unreleased]: https://github.com/twopopfilm/fountain-syntax/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/twopopfilm/fountain-syntax/releases/tag/v0.1.0
