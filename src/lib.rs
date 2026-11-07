//! A lossless, spec-complete parser and writer for [Fountain](https://fountain.io) screenplays.
//!
//! Pre-alpha. [`parse`] returns a [`Document`]: the source text and a syntax tree over it whose
//! tokens, in order, reproduce the source exactly. The tree covers every block element: the title
//! page, scene headings with their scene numbers, action, cues, dialogue and parentheticals,
//! dual-dialogue carets, transitions, centered text, lyrics, sections, synopses and page breaks
//! (and Shot with the `shot` feature). Notes and Boneyard are `Note` and `Boneyard` nodes, and a `Gap`
//! holds the line breaks and blank lines around top-level nodes: before the first, between two, and
//! after the last. The `locked-scene-numbers` feature reads `##12A#` as a scene number.
//!
//! [`Document::screenplay`] builds the typed view, a [`Screenplay`]: the title page, the preamble
//! and the scenes, as plain owned data. Its Dialogue blocks hold the cue, the dialogue lines, the
//! parentheticals and the lines that interrupt them, and a caret cue is paired with its partner
//! as dual dialogue. The diagnostics, the writer and the hashes are not written yet. The `serde`
//! feature serialises a [`Document`] as its source and parses it again when it is read back, and
//! serialises the view as it is. The design is in `SPEC.md`.
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

#![warn(missing_docs)]

mod classify;
mod hidden;
mod layout;
mod lines;
mod parse;
mod tree;
mod view;

pub use tree::{Child, Document, Node, NodeKind, Span, Token, TokenKind};
#[cfg(feature = "shot")]
pub use view::Shot;
pub use view::{
    Action, Boneyard, Cue, DialogueBlock, DialogueItem, DualDialogue, Element, Extension, Heading,
    Lyrics, Note, NoteConvention, PageBreak, Preamble, Run, Scene, SceneNumber, Screenplay,
    Section, Synopsis, Text, TitleEntry, TitlePage, Transition,
};

/// Parses any text. It never fails and never panics.
/// The returned tree reproduces `source` exactly.
pub fn parse(source: &str) -> Document {
    parse::document(source)
}
