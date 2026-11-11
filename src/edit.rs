//! Targeted edits (`SPEC.md` 10.2): replace one span of the source and re-parse.

use core::fmt;

use crate::{Child, Document, Node, NodeKind, Span, TokenKind, parse};

/// Why a named edit (`SPEC.md` 10.2) was refused. The document it was called on is unchanged.
///
/// ```
/// use fountain_syntax::{EditError, Span, parse};
///
/// let doc = parse("INT. A\n");
/// assert_eq!(doc.with_scene_number(Span::new(0, 1), "1").err(), Some(EditError::NotFound));
/// assert_eq!(doc.with_scene_number(Span::new(0, 6), "1 2").err(), Some(EditError::Invalid));
/// ```
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditError {
    /// No element of the kind the edit needs has exactly this span.
    NotFound,
    /// The edited text would not parse as the element it was meant to be, or the edit would drop
    /// a Note or Boneyard.
    Invalid,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EditError::NotFound => "no element of the needed kind has this span",
            EditError::Invalid => "the edited text would not parse as the intended element",
        })
    }
}

impl std::error::Error for EditError {}

/// Whether the heading's scene number opens with `open` and has the text `number`.
fn carries_number(heading: Node<'_>, open: &str, number: &str) -> bool {
    heading.children().any(|c| match c {
        Child::Node(n) if n.kind() == NodeKind::SceneNumber => {
            let mut tokens = n.children().filter_map(|c| match c {
                Child::Token(t) => Some(t),
                Child::Node(_) => None,
            });
            let first = tokens.next().map(|t| t.text());
            let text = tokens
                .find(|t| t.kind() == TokenKind::Text)
                .map(|t| t.text());
            first == Some(open) && text == Some(number)
        }
        _ => false,
    })
}

impl Document {
    /// The node of `kind` whose span is exactly `span`, at any depth.
    fn find(&self, kind: NodeKind, span: Span) -> Option<Node<'_>> {
        fn walk(node: Node<'_>, kind: NodeKind, span: Span) -> Option<Node<'_>> {
            if node.kind() == kind && node.span() == span {
                return Some(node);
            }
            node.children().find_map(|c| match c {
                Child::Node(n) => walk(n, kind, span),
                Child::Token(_) => None,
            })
        }
        walk(self.root(), kind, span)
    }

    /// Writes `{open}{number}#` on the Scene Heading with span `heading`, replacing its scene
    /// number or adding one after its text, then checks that it parses back as that number.
    fn with_number_marked(
        &self,
        heading: Span,
        open: &str,
        number: &str,
    ) -> Result<Document, EditError> {
        let node = self
            .find(NodeKind::SceneHeading, heading)
            .ok_or(EditError::NotFound)?;
        let mut at = heading.end;
        let mut existing = None;
        for child in node.children() {
            match child {
                Child::Node(n) if n.kind() == NodeKind::SceneNumber => existing = Some(n.span()),
                Child::Token(t) if t.kind() == TokenKind::Whitespace => {}
                Child::Node(n) => at = n.span().end,
                Child::Token(t) => at = t.span().end,
            }
        }
        let (span, text) = match existing {
            Some(span) => (span, format!("{open}{number}#")),
            None => (Span::new(at, at), format!(" {open}{number}#")),
        };
        let edited = self.with_replaced(span, &text);
        // The parser's scene number rule (SPEC.md OFF-8a) is the only validation of `number`:
        // the heading, now longer or shorter by the difference, must carry exactly this number.
        let end = heading.end + text.len() - span.len();
        let carries = edited
            .find(NodeKind::SceneHeading, Span::new(heading.start, end))
            .is_some_and(|h| carries_number(h, open, number));
        if carries {
            Ok(edited)
        } else {
            Err(EditError::Invalid)
        }
    }

    /// Sets the scene number of the Scene Heading whose span is `heading` (the `span` of a
    /// scene's `heading` in the view): replaces its number or adds ` #number#` after the
    /// heading's text. Nothing else changes. A locked number becomes a plain one.
    ///
    /// `number` must be a scene number the parser reads (`SPEC.md` OFF-8a: letters, digits, `-`
    /// and `.`); otherwise the result is [`EditError::Invalid`]. A span that is not a Scene
    /// Heading gives [`EditError::NotFound`].
    ///
    /// ```
    /// use fountain_syntax::parse;
    ///
    /// let doc = parse("INT. A\n\nHi.\n");
    /// let heading = doc.screenplay().scenes[0].heading.span.unwrap();
    /// let edited = doc.with_scene_number(heading, "12A").unwrap();
    /// assert_eq!(edited.source(), "INT. A #12A#\n\nHi.\n");
    /// ```
    pub fn with_scene_number(&self, heading: Span, number: &str) -> Result<Document, EditError> {
        self.with_number_marked(heading, "#", number)
    }

    /// Like [`Document::with_scene_number`], but writes the locked form `##number#`.
    ///
    /// ```
    /// use fountain_syntax::parse;
    ///
    /// let doc = parse("INT. A\n");
    /// let heading = doc.screenplay().scenes[0].heading.span.unwrap();
    /// let edited = doc.with_locked_scene_number(heading, "3").unwrap();
    /// assert_eq!(edited.source(), "INT. A ##3#\n");
    /// assert!(edited.screenplay().scenes[0].number.as_ref().unwrap().locked);
    /// ```
    #[cfg(feature = "locked-scene-numbers")]
    pub fn with_locked_scene_number(
        &self,
        heading: Span,
        number: &str,
    ) -> Result<Document, EditError> {
        self.with_number_marked(heading, "##", number)
    }

    /// Replaces the bytes of `span` with `text`, re-parses, and leaves every other byte
    /// untouched. The named edits are built on this primitive.
    ///
    /// ```
    /// use fountain_syntax::{Span, parse};
    ///
    /// let doc = parse("INT. A #1#\n\nHello.\n");
    /// let edited = doc.with_replaced(Span::new(7, 10), "#12A#");
    /// assert_eq!(edited.source(), "INT. A #12A#\n\nHello.\n");
    /// ```
    ///
    /// # Panics
    ///
    /// When the span is out of range, inverted, or not on character boundaries, as
    /// [`String::replace_range`] does.
    pub fn with_replaced(&self, span: Span, text: &str) -> Document {
        let mut source = self.source().to_owned();
        source.replace_range(span.start..span.end, text);
        parse(&source)
    }
}
