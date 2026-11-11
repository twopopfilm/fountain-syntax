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
//! as dual dialogue. [`Document::diagnostics`] reports the 22 diagnostic codes of `SPEC.md` §7
//! without changing the tree or the view.
//!
//! [`Document::write`] (and `Display`) gives the source back byte for byte. [`canonical`] writes a
//! view built in memory as Fountain that parses back to the same view, except for the shapes its
//! docs list, with a sigil only where one is needed. The `serde` feature serialises a [`Document`]
//! as its source and parses it again when it is read back, and serialises the view as it is. The
//! [`Digest`] type exists in every build; a scheme 2 digest prints as `fs2:` and 64 lowercase
//! hexadecimal digits, and a scheme 1 digest as the bare 64 digits. The `hash` feature
//! adds the methods that compute scheme 2 digests (`SPEC.md` section 11): for elements, scenes,
//! the preamble, the title page, the whole screenplay and the exact bytes of any span (for example
//! `Screenplay::hash`). Scheme 1, the importer-compatible scheme, hashes scenes only: `Scene::hash_v1`
//! with the `hash` feature, and in every build the line methods `Scene::normalized_text_v1`,
//! `heading_line_v1` and `element_lines_v1` that give the text it hashes. The design is in `SPEC.md`.
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
mod diagnose;
mod diagnostic;
mod digest;
#[cfg(feature = "hash")]
mod hash;
mod hidden;
mod layout;
mod lines;
mod parse;
mod tree;
mod view;
mod write;

pub use diagnostic::{Diagnostic, DiagnosticCode, Severity};
pub use digest::{Digest, HASH_SCHEME_CURRENT, HashScheme, ParseDigestError};
pub use tree::{Child, Document, Node, NodeKind, Span, Token, TokenKind};
#[cfg(feature = "shot")]
pub use view::Shot;
pub use view::{
    Action, Boneyard, Cue, DialogueBlock, DialogueItem, DualDialogue, Element, Extension, Heading,
    Lyrics, Note, NoteConvention, PageBreak, Preamble, Run, Scene, SceneNumber, Screenplay,
    Section, Synopsis, Text, TitleEntry, TitlePage, Transition,
};
pub use write::canonical;

/// Parses any text. It never fails and never panics.
/// The returned tree reproduces `source` exactly.
pub fn parse(source: &str) -> Document {
    parse::document(source)
}
