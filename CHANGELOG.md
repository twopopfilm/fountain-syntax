# Changelog

All notable changes to this project are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). The crate is not released yet.

## [Unreleased]

### Added

- A lossless syntax tree: `parse` never fails, puts a span on every node and keeps every byte of the source,
  including the title page and every block element.
- A typed screenplay view: `Document::screenplay()` gives the title page, the scenes and their elements, including
  dialogue, parentheticals and dual dialogue.
- Diagnostics: `Document::diagnostics()` reports the 22 codes of `SPEC.md` section 7 without changing the tree.
- A writer: `Document::write` gives the source back byte for byte, and `canonical` writes a view built in memory as
  Fountain that parses back to the same view.
- Targeted edits: `Document::with_replaced` and the named edits `with_scene_number`, `with_synopsis`, `with_note` and
  `with_cue_renamed`, which return an `EditError` instead of an edit that would not read as intended.
- Hashes behind the `hash` feature: scheme 2 hashes of elements, scenes, the document, the title page and source spans,
  and the importer-compatible scheme 1 hash of a scene (`Scene::hash_v1`).
- Four Cargo features, all off by default: `locked-scene-numbers`, `shot`, `serde` and `hash`.
- Usage examples in the README, compiled and run by `cargo test`, and examples in the API documentation.

### Deferred

- A WASM build for a browser playground, until after 0.1.
