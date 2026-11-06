//! A lossless, spec-complete parser and writer for [Fountain](https://fountain.io) screenplays.
//!
//! Pre-alpha. [`parse`] returns a [`Document`]: the source text and a flat syntax tree over it whose
//! tokens, in order, reproduce the source exactly. So far the tree has Action, Lyrics, Centered,
//! forced Transition, Section, Synopsis and Page Break nodes (and Shot with the `shot` feature),
//! Notes and Boneyard as `Note` and `Boneyard` nodes, and a `Gap` for everything between two
//! top-level nodes; headings, cues, dialogue and the title page arrive in later milestones. The
//! typed view, the diagnostics and the writer are not written yet. The design is in `SPEC.md`.

mod classify;
mod hidden;
mod layout;
mod lines;
mod parse;
mod tree;

pub use tree::{Child, Document, Node, NodeKind, Span, Token, TokenKind};

/// Parses any text. It never fails and never panics.
/// The returned tree reproduces `source` exactly.
pub fn parse(source: &str) -> Document {
    parse::document(source)
}
