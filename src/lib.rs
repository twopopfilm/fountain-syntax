//! A lossless, spec-complete parser and writer for [Fountain](https://fountain.io) screenplays.
//!
//! Pre-alpha. [`parse`] returns a [`Document`]: the source text and a flat syntax tree over it whose
//! tokens, in order, reproduce the source exactly. For now every run of non-blank lines is one
//! `Action` node and everything between two nodes is a `Gap`; the other elements arrive in later
//! milestones. The typed view, the diagnostics and the writer are not written yet. The design is in
//! `SPEC.md`.

mod lines;
mod parse;
mod tree;

pub use tree::{Child, Document, Node, NodeKind, Span, Token, TokenKind};

/// Parses any text. It never fails and never panics.
/// The returned tree reproduces `source` exactly.
pub fn parse(source: &str) -> Document {
    parse::document(source)
}
