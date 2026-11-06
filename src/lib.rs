//! A lossless, spec-complete parser and writer for [Fountain](https://fountain.io) screenplays.
//!
//! Pre-alpha. [`parse`] returns a [`Document`]: the source text and a syntax tree over it whose
//! tokens, in order, reproduce the source exactly. The tree covers every block element: the title
//! page, scene headings with their scene numbers, action, cues, dialogue and parentheticals,
//! dual-dialogue carets, transitions, centered text, lyrics, sections, synopses and page breaks
//! (and Shot with the `shot` feature). Notes and Boneyard are `Note` and `Boneyard` nodes, and a `Gap`
//! holds everything between two top-level nodes. The typed view, the diagnostics and the writer are
//! not written yet. The design is in `SPEC.md`.
//!
//! ```
//! use fountain_syntax::{Child, NodeKind, parse};
//!
//! let doc = parse("Title: Big Fish\n\nINT. HOUSE - DAY\n");
//! let kinds: Vec<NodeKind> = doc
//!     .root()
//!     .children()
//!     .filter_map(|child| match child {
//!         Child::Node(node) if node.kind() != NodeKind::Gap => Some(node.kind()),
//!         _ => None,
//!     })
//!     .collect();
//! assert_eq!(kinds, [NodeKind::TitlePage, NodeKind::SceneHeading]);
//! assert_eq!(doc.source(), "Title: Big Fish\n\nINT. HOUSE - DAY\n");
//! ```

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
