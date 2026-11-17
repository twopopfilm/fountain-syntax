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
    /// The text cannot be written as the element it was meant to be: it would not parse as that
    /// element, or it holds a mark the element cannot hold (`[[` or `]]` in a Note). Or the edit
    /// would drop a Note or Boneyard.
    Invalid,
}

impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            EditError::NotFound => "no element of the needed kind has this span",
            EditError::Invalid => {
                "the text cannot be written as the intended element, or the edit would drop a Note or Boneyard"
            }
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

/// Whether a Note or Boneyard sits anywhere inside `node`: replacing the node would drop it.
fn hides(node: Node<'_>) -> bool {
    node.children().any(|c| match c {
        Child::Node(n) => matches!(n.kind(), NodeKind::Note | NodeKind::Boneyard) || hides(n),
        Child::Token(_) => false,
    })
}

/// Whether `node` is a Gap of exactly one line break.
fn is_one_break(node: Node<'_>) -> bool {
    node.kind() == NodeKind::Gap && matches!(node.text(), "\n" | "\r\n" | "\r")
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
                Child::Node(n) if n.kind() == NodeKind::SceneNumber => {
                    if hides(n) {
                        return Err(EditError::Invalid);
                    }
                    existing = Some(n.span());
                }
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
    /// and `.`); otherwise the result is [`EditError::Invalid`]. So is a number that holds a Note
    /// or Boneyard, which replacing it would drop. A span that is not a Scene Heading gives
    /// [`EditError::NotFound`].
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

    /// Like [`Document::with_scene_number`], but writes the locked form `##number#`. It also
    /// gives [`EditError::Invalid`] for a number that holds a Note or Boneyard, which replacing it
    /// would drop.
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

    /// The root's children that are nodes, Gaps included, in source order.
    fn top_level(&self) -> Vec<Node<'_>> {
        self.root()
            .children()
            .filter_map(|c| match c {
                Child::Node(n) => Some(n),
                Child::Token(_) => None,
            })
            .collect()
    }

    /// The first line break in the source, or `"\n"` when it has none.
    fn line_break(&self) -> &str {
        let source = self.source();
        match source.find(['\r', '\n']) {
            Some(i) if source[i..].starts_with("\r\n") => "\r\n",
            Some(i) if source[i..].starts_with('\r') => "\r",
            _ => "\n",
        }
    }

    /// Sets the one-line Synopsis of the scene whose Scene Heading has the span `heading`: puts
    /// `= text` on the line right after the heading, or, when a Synopsis is already there,
    /// replaces it. Replacing keeps the whitespace before its `=` and rewrites the rest of its line
    /// as `= text`. The new line uses the document's line break. Any other Synopsis of the scene
    /// stays as it is.
    ///
    /// A `text` that holds a line break, or that would not read back as a Synopsis of this
    /// scene, gives [`EditError::Invalid`]. So does replacing a Synopsis that holds a Note or
    /// Boneyard, which would drop it. A span that is not a Scene Heading gives
    /// [`EditError::NotFound`].
    ///
    /// ```
    /// use fountain_syntax::parse;
    ///
    /// let doc = parse("INT. A\n\nHi.\n");
    /// let heading = doc.screenplay().scenes[0].heading.span.unwrap();
    /// let edited = doc.with_synopsis(heading, "Mara arrives.").unwrap();
    /// assert_eq!(edited.source(), "INT. A\n= Mara arrives.\n\nHi.\n");
    ///
    /// let again = edited.with_synopsis(heading, "Mara leaves.").unwrap();
    /// assert_eq!(again.source(), "INT. A\n= Mara leaves.\n\nHi.\n");
    /// ```
    pub fn with_synopsis(&self, heading: Span, text: &str) -> Result<Document, EditError> {
        let top = self.top_level();
        let i = top
            .iter()
            .position(|n| n.kind() == NodeKind::SceneHeading && n.span() == heading)
            .ok_or(EditError::NotFound)?;
        if text.contains(['\n', '\r']) {
            return Err(EditError::Invalid);
        }
        let line = format!("= {text}");
        let (span, edit, expected) = match (top.get(i + 1), top.get(i + 2)) {
            (Some(gap), Some(syn)) if is_one_break(*gap) && syn.kind() == NodeKind::Synopsis => {
                if hides(*syn) {
                    return Err(EditError::Invalid);
                }
                // The node's span holds the indent before its `=`; keep it and replace the rest.
                let span = syn.span();
                let old = syn.text();
                let indent = &old[..old.len() - old.trim_start_matches([' ', '\t']).len()];
                (
                    Span::new(span.start + indent.len(), span.end),
                    line.clone(),
                    format!("{indent}{line}"),
                )
            }
            _ => (
                Span::new(heading.end, heading.end),
                format!("{}{line}", self.line_break()),
                line,
            ),
        };
        let edited = self.with_replaced(span, &edit);
        // The heading must be unchanged and directly followed, across one line break, by the line.
        let top = edited.top_level();
        let ok = top
            .iter()
            .position(|n| n.span().start == heading.start)
            .is_some_and(|i| {
                top[i].kind() == NodeKind::SceneHeading
                    && top[i].span() == heading
                    && top.get(i + 1).is_some_and(|g| is_one_break(*g))
                    && top
                        .get(i + 2)
                        .is_some_and(|n| n.kind() == NodeKind::Synopsis && n.text() == expected)
            });
        if ok {
            Ok(edited)
        } else {
            Err(EditError::Invalid)
        }
    }

    /// Adds the Note `[[text]]` as its own block after the top-level element whose span is
    /// `after`, with a blank line before it (and one after it when the next element would
    /// otherwise follow on the very next line). Nothing else changes. The text is inserted as
    /// given, with its own line breaks: a multi-line note in a CRLF document keeps the breaks it
    /// was written with.
    ///
    /// After a Scene Heading that a Synopsis follows directly, the note goes between them, so
    /// the view then reads that Synopsis as an element of the scene, not as its `synopses`.
    ///
    /// A `text` that holds `[[`, `]]` or a blank line, or that would not read back as a Note,
    /// gives [`EditError::Invalid`]: Notes don't nest (`SPEC.md` OFF-15a). Empty text gives the
    /// empty note `[[]]`. A span that is not a top-level element of the document (a cue, a
    /// title page entry, the blank lines between elements) gives [`EditError::NotFound`].
    ///
    /// ```
    /// use fountain_syntax::{Child, NodeKind, parse};
    ///
    /// let doc = parse("INT. A\n\nHi.\n\nBYE.\n");
    /// let hi = doc.root().children().find_map(|c| match c {
    ///     Child::Node(n) if n.kind() == NodeKind::Action => Some(n.span()),
    ///     _ => None,
    /// });
    /// let edited = doc.with_note(hi.unwrap(), "check this").unwrap();
    /// assert_eq!(edited.source(), "INT. A\n\nHi.\n\n[[check this]]\n\nBYE.\n");
    /// ```
    pub fn with_note(&self, after: Span, text: &str) -> Result<Document, EditError> {
        let top = self.top_level();
        let i = top
            .iter()
            .position(|n| n.kind() != NodeKind::Gap && n.span() == after)
            .ok_or(EditError::NotFound)?;
        let unix = text.replace("\r\n", "\n").replace('\r', "\n");
        if text.contains("[[") || text.contains("]]") || unix.contains("\n\n") {
            return Err(EditError::Invalid);
        }
        let nl = self.line_break();
        let note = format!("[[{text}]]");
        let mut edit = format!("{nl}{nl}{note}");
        // One line break and then another element: keep a blank line between the note and it.
        if top.get(i + 2).is_some() && top.get(i + 1).is_some_and(|gap| is_one_break(*gap)) {
            edit.push_str(nl);
        }
        let edited = self.with_replaced(Span::new(after.end, after.end), &edit);
        let start = after.end + 2 * nl.len();
        let ok = edited
            .top_level()
            .iter()
            .any(|n| n.kind() == NodeKind::Note && n.span().start == start && n.text() == note);
        if ok {
            Ok(edited)
        } else {
            Err(EditError::Invalid)
        }
    }

    /// Renames the Character cue whose span is `cue` (the `span` of a `Cue` in the view): replaces
    /// the cue's name and nothing else, so its `@`, extensions, caret and the rest of its line stay
    /// as they are. Other cues of the same character are not touched.
    ///
    /// `name` must read back as exactly the name of that cue: a lowercase name on a cue without
    /// `@`, a name with a trailing space, a line break, an extension or a caret in it, or an empty
    /// name gives [`EditError::Invalid`]. So does a cue whose current name holds a Note or Boneyard
    /// (`MA[[n]]RA`), which the rename would drop. A new name may hold one (`JU[[n]]NE`): it is
    /// written as given, and the view's name leaves it out. A span that is not a Character cue
    /// gives [`EditError::NotFound`].
    ///
    /// ```
    /// use fountain_syntax::{Element, parse};
    ///
    /// let doc = parse("INT. A\n\nMARA (V.O.)\nHi.\n");
    /// let Element::Dialogue(block) = &doc.screenplay().scenes[0].elements[0] else { panic!() };
    /// let edited = doc.with_cue_renamed(block.cue.span.unwrap(), "JUNE").unwrap();
    /// assert_eq!(edited.source(), "INT. A\n\nJUNE (V.O.)\nHi.\n");
    /// ```
    pub fn with_cue_renamed(&self, cue: Span, name: &str) -> Result<Document, EditError> {
        let node = self
            .find(NodeKind::CharacterCue, cue)
            .ok_or(EditError::NotFound)?;
        let old = node
            .children()
            .find_map(|c| match c {
                Child::Node(n) if n.kind() == NodeKind::CharacterName => Some(n),
                _ => None,
            })
            .ok_or(EditError::Invalid)?;
        if hides(old) {
            return Err(EditError::Invalid);
        }
        let edited = self.with_replaced(old.span(), name);
        // The cue, now longer or shorter by the difference, must carry exactly this name.
        let end = cue.end + name.len() - old.span().len();
        let named = edited
            .find(NodeKind::CharacterCue, Span::new(cue.start, end))
            .is_some_and(|c| {
                c.children().any(|c| {
                    matches!(c, Child::Node(n) if n.kind() == NodeKind::CharacterName && n.text() == name)
                })
            });
        if named {
            Ok(edited)
        } else {
            Err(EditError::Invalid)
        }
    }

    /// Replaces the bytes of `span` with `text`, re-parses, and leaves every other byte
    /// untouched. The named edits are built on this primitive. The result is a new document:
    /// after an edit, take spans from it, not from the old document or its view, whose spans
    /// refer to the old source and can match a different element of the new one.
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
