//! Builds the typed view from the tree (SPEC 9).
//!
//! Three passes: [`items`] turns the root's nodes into a flat list of elements with their spans,
//! [`pair`] joins the dual-dialogue blocks, then [`group`] sorts the list into the preamble and the
//! scenes. The tree is flat (SPEC 8.4), so every walk below has a fixed depth, and all three
//! passes are linear in the size of the tree.

use super::inline::{self, Piece, Trim};
use super::{
    Action, Boneyard, Cue, DialogueBlock, DialogueItem, DualDialogue, Element, Extension, Heading,
    Lyrics, Note, PageBreak, Preamble, Scene, SceneNumber, Screenplay, Section, Synopsis, Text,
    TitleEntry, TitlePage, Transition,
};
use crate::{Child, Document, Node, NodeKind, Span, TokenKind};

impl Document {
    /// The typed view of the screenplay (SPEC 9), built from the tree on each call.
    ///
    /// The first title page becomes [`Screenplay::title_page`]. Each Scene Heading starts a
    /// [`Scene`], which also takes the run of Sections and Synopses directly before the heading
    /// and the Synopses directly after it; blank lines do not break either run. What comes before
    /// the first heading, apart from those, is the [`Preamble`]. A Dialogue block whose cue has a
    /// caret joins the Dialogue block before it as a [`DualDialogue`](super::DualDialogue), when
    /// only Sections, Synopses, Page Breaks, Notes and Boneyards come between them; those lines
    /// end the left block's items. Otherwise it stays a block with `unpaired_caret` set. Sigils
    /// and structural marks are left out of every [`Text`]. Notes and Boneyard are kept: as
    /// elements when they stand alone, and as runs of the text they sit in otherwise.
    ///
    /// Like [`parse`](crate::parse), it never fails and never panics.
    ///
    /// ```
    /// use fountain_syntax::parse;
    ///
    /// let view = parse("Title: Big Fish\n\n= The set-up\n\nINT. HOUSE - DAY #1#\n").screenplay();
    /// let scene = &view.scenes[0];
    /// assert_eq!(scene.heading.text.plain(), "INT. HOUSE - DAY");
    /// assert_eq!(scene.number.as_ref().map(|n| n.text.as_str()), Some("1"));
    /// assert_eq!(scene.synopses[0].text.plain(), "The set-up");
    /// ```
    pub fn screenplay(&self) -> Screenplay {
        let (title_page, items) = items(self);
        let (preamble, scenes) = group(pair(items));
        Screenplay {
            title_page,
            preamble,
            scenes,
        }
    }
}

/// One top-level element of the tree, with the bytes its node covers.
struct Item {
    block: Block,
    span: Span,
}

enum Block {
    Heading(Heading, Option<SceneNumber>),
    Element(Element),
}

/// The title page and the other top-level nodes, in source order. Gaps and the BOM are skipped.
fn items(doc: &Document) -> (Option<TitlePage>, Vec<Item>) {
    let mut title_page = None;
    let mut items = Vec::new();
    for child in doc.root().children() {
        let Child::Node(node) = child else { continue };
        let block = match node.kind() {
            NodeKind::TitlePage => {
                if title_page.is_none() {
                    title_page = Some(title(doc, node));
                }
                continue;
            }
            NodeKind::SceneHeading => heading(doc, node),
            _ => match element(doc, node) {
                Some(e) => Block::Element(e),
                None => continue,
            },
        };
        items.push(Item {
            block,
            span: node.span(),
        });
    }
    (title_page, items)
}

/// The element for a top-level node, or for a Section, Synopsis or Page Break inside an Action.
fn element(doc: &Document, node: Node<'_>) -> Option<Element> {
    Some(match node.kind() {
        NodeKind::Action => Element::Action(action(doc, node)),
        NodeKind::Centered => Element::Action(Action {
            text: leaf(doc, node),
            centered: true,
            forced: false,
            interjections: Vec::new(),
        }),
        NodeKind::DialogueBlock => Element::Dialogue(dialogue(doc, node)),
        NodeKind::Transition => Element::Transition(Transition {
            text: leaf(doc, node),
            forced: has_sigil(node),
        }),
        NodeKind::Lyrics => Element::Lyrics(lyrics(doc, node)),
        NodeKind::Section => Element::Section(Section {
            depth: marker(node).map_or(0, |m| m.matches('#').count()),
            text: leaf(doc, node),
        }),
        NodeKind::Synopsis => Element::Synopsis(Synopsis {
            text: leaf(doc, node),
        }),
        NodeKind::PageBreak => Element::PageBreak(PageBreak {
            span: Some(node.span()),
        }),
        NodeKind::Note => Element::Note(note(doc, node)),
        NodeKind::Boneyard => Element::Boneyard(boneyard(doc, node)),
        #[cfg(feature = "shot")]
        NodeKind::Shot => Element::Shot(super::Shot {
            text: leaf(doc, node),
        }),
        NodeKind::Document
        | NodeKind::Gap
        | NodeKind::TitlePage
        | NodeKind::TitleEntry
        | NodeKind::TitleKey
        | NodeKind::TitleValue
        | NodeKind::SceneHeading
        | NodeKind::SceneNumber
        | NodeKind::CharacterCue
        | NodeKind::CharacterName
        | NodeKind::Extension
        | NodeKind::Caret
        | NodeKind::Parenthetical
        | NodeKind::Dialogue => return None,
    })
}

fn has_sigil(node: Node<'_>) -> bool {
    node.children()
        .any(|c| matches!(c, Child::Token(t) if t.kind() == TokenKind::Sigil))
}

/// The text of the node's first Marker token.
fn marker<'a>(node: Node<'a>) -> Option<&'a str> {
    node.children().find_map(|c| match c {
        Child::Token(t) if t.kind() == TokenKind::Marker => Some(t.text()),
        _ => None,
    })
}

/// The hidden atom for a Note or Boneyard node; `None` for any other node.
fn hidden<'a>(doc: &Document, node: Node<'_>) -> Option<Piece<'a>> {
    match node.kind() {
        NodeKind::Note => Some(Piece::Note(note(doc, node))),
        NodeKind::Boneyard => Some(Piece::Boneyard(boneyard(doc, node))),
        _ => None,
    }
}

fn note(doc: &Document, node: Node<'_>) -> Note {
    Note {
        text: inside(doc, node),
        span: Some(node.span()),
    }
}

fn boneyard(doc: &Document, node: Node<'_>) -> Boneyard {
    Boneyard {
        text: inside(doc, node),
        span: Some(node.span()),
    }
}

/// The raw text between a Note's or Boneyard's opening and closing markers, as written.
fn inside(doc: &Document, node: Node<'_>) -> String {
    let span = node.span();
    let marker_span = |c: Option<Child<'_>>| match c {
        Some(Child::Token(t)) if t.kind() == TokenKind::Marker => Some(t.span()),
        _ => None,
    };
    let mut children = node.children();
    let start = marker_span(children.next()).map_or(span.start, |s| s.end);
    // `last` of what is left, so a lone marker is not used as both ends.
    let end = marker_span(children.last()).map_or(span.end, |s| s.start);
    doc.source()
        .get(start..end.max(start))
        .unwrap_or_default()
        .to_owned()
}

fn text_piece<'a>(t: crate::Token<'a>) -> Piece<'a> {
    Piece::Text {
        text: t.text(),
        span: t.span(),
    }
}

/// The pieces of a one-line node: its Text and Whitespace tokens, line breaks and hidden atoms.
/// Sigils, markers, colons and any other sub-node (such as a scene number) are left out.
fn pieces<'a>(doc: &Document, node: Node<'a>) -> Vec<Piece<'a>> {
    let mut out = Vec::new();
    for child in node.children() {
        match child {
            Child::Token(t) => match t.kind() {
                TokenKind::Text | TokenKind::Whitespace => out.push(text_piece(t)),
                TokenKind::LineBreak => out.push(Piece::Break {
                    text: t.text(),
                    span: t.span(),
                }),
                TokenKind::Bom | TokenKind::Sigil | TokenKind::Marker | TokenKind::Colon => {}
            },
            Child::Node(n) => out.extend(hidden(doc, n)),
        }
    }
    out
}

/// The trimmed text of a one-line element: a heading, Section, Synopsis, Transition, centered
/// text or Shot.
fn leaf(doc: &Document, node: Node<'_>) -> Text {
    inline::text(pieces(doc, node), Trim::Both)
}

fn heading(doc: &Document, node: Node<'_>) -> Block {
    let number = node.children().find_map(|c| match c {
        Child::Node(n) if n.kind() == NodeKind::SceneNumber => Some(SceneNumber {
            text: n
                .children()
                .filter_map(|c| match c {
                    Child::Token(t) if t.kind() == TokenKind::Text => Some(t.text()),
                    _ => None,
                })
                .collect(),
            #[cfg(feature = "locked-scene-numbers")]
            locked: marker(n) == Some("##"),
            span: Some(n.span()),
        }),
        _ => None,
    });
    let heading = Heading {
        text: leaf(doc, node),
        forced: has_sigil(node),
        span: Some(node.span()),
    };
    Block::Heading(heading, number)
}

/// An Action paragraph. Each line that holds only an interjection (a Section, Synopsis or Page
/// Break) or only hidden spans is left out with its line break; its hidden spans stay, as runs
/// at the start of the next line (or the end of the last one).
fn action(doc: &Document, node: Node<'_>) -> Action {
    let mut out = Vec::new();
    let mut interjections = Vec::new();
    let mut forced = false;
    // The current line: where its pieces start, whether it has visible text, and whether it
    // holds an interjection or a hidden span.
    let mut line_start = 0;
    let mut visible = false;
    let mut marked = false;
    // Only the first line's `!` forces the paragraph, whatever indent or hidden span sits before
    // it (OFF-22a); a `!` on a later line is left out of the text and forces nothing.
    let mut first_line = true;
    for child in node.children() {
        match child {
            Child::Token(t) => match t.kind() {
                TokenKind::Sigil => forced |= first_line,
                TokenKind::Text | TokenKind::Whitespace => {
                    visible |= !t.text().trim_matches([' ', '\t']).is_empty();
                    out.push(text_piece(t));
                }
                TokenKind::LineBreak => {
                    first_line = false;
                    if marked && !visible {
                        keep_hidden(&mut out, line_start);
                    } else {
                        out.push(Piece::Break {
                            text: t.text(),
                            span: t.span(),
                        });
                    }
                    line_start = out.len();
                    visible = false;
                    marked = false;
                }
                TokenKind::Bom | TokenKind::Marker | TokenKind::Colon => {}
            },
            Child::Node(n) => {
                if let Some(p) = hidden(doc, n) {
                    out.push(p);
                    marked = true;
                } else if let Some(e) = element(doc, n) {
                    interjections.push(e);
                    marked = true;
                }
            }
        }
    }
    // Only a one-line Action such as `! [[n]]` ends here with no visible text: the parser ends an
    // Action before a trailing hidden or interjection line, so no break comes before this one.
    if marked && !visible {
        keep_hidden(&mut out, line_start);
    }
    Action {
        text: inline::text(out, Trim::ActionLines),
        centered: false,
        forced,
        interjections,
    }
}

/// Drop the whitespace pieces of a line that has no visible text, keeping its hidden atoms.
fn keep_hidden(out: &mut Vec<Piece<'_>>, line_start: usize) {
    let start = line_start.min(out.len());
    let atoms: Vec<Piece<'_>> = out
        .drain(start..)
        .filter(|p| !matches!(p, Piece::Text { .. }))
        .collect();
    out.extend(atoms);
}

/// The texts of an element with several (a title page, a Lyrics run, a cue), collected as
/// piece lists before they are built. A hidden atom joins the most recent text (the nearest
/// before it, Decision 4), or waits for the next one when there is none yet.
#[derive(Default)]
struct Slots<'a> {
    slots: Vec<Vec<Piece<'a>>>,
    pending: Vec<Piece<'a>>,
}

impl<'a> Slots<'a> {
    /// Start a text with `pieces`, after any waiting hidden atoms. Returns its index.
    fn open(&mut self, pieces: Vec<Piece<'a>>) -> usize {
        let mut slot = std::mem::take(&mut self.pending);
        slot.extend(pieces);
        self.slots.push(slot);
        self.slots.len() - 1
    }

    /// Add a piece to the most recent text.
    fn push(&mut self, piece: Piece<'a>) {
        match self.slots.last_mut() {
            Some(slot) => slot.push(piece),
            None => self.pending.push(piece),
        }
    }

    /// [`Slots::push`] for the hidden atom of a node, if it is a Note or Boneyard.
    fn push_hidden(&mut self, piece: Option<Piece<'a>>) {
        if let Some(p) = piece {
            self.push(p);
        }
    }

    /// At a line break: drop trailing whitespace from the most recent text, so a hidden atom on
    /// a later line lands right after its visible text.
    fn end_line(&mut self) {
        if let Some(slot) = self.slots.last_mut() {
            while matches!(slot.last(), Some(Piece::Text { text, .. }) if text.trim_matches([' ', '\t']).is_empty())
            {
                slot.pop();
            }
        }
    }

    /// A text of its own for hidden atoms that found no text to join, if there are any.
    fn flush(&mut self) -> Option<usize> {
        (!self.pending.is_empty()).then(|| self.open(Vec::new()))
    }

    /// Build the texts, trimmed at both ends.
    fn finish(self) -> Vec<Text> {
        self.slots
            .into_iter()
            .map(|s| inline::text(s, Trim::Both))
            .collect()
    }
}

/// Take the text at `i`, or an empty one if there is none.
fn take(texts: &mut [Text], i: usize) -> Text {
    texts.get_mut(i).map(std::mem::take).unwrap_or_default()
}

fn title(doc: &Document, node: Node<'_>) -> TitlePage {
    let mut slots = Slots::default();
    // Each entry: the index of its first text (the key), how many texts it has, and its span.
    let mut entries = Vec::new();
    let mut strays = Vec::new();
    let mut in_stray = false;
    for child in node.children() {
        match child {
            Child::Token(t) => match t.kind() {
                TokenKind::Text | TokenKind::Whitespace => {
                    if t.kind() == TokenKind::Text && !in_stray {
                        strays.push(slots.open(Vec::new()));
                        in_stray = true;
                    }
                    if in_stray {
                        slots.push(text_piece(t));
                    }
                }
                TokenKind::LineBreak => {
                    in_stray = false;
                    slots.end_line();
                }
                TokenKind::Bom | TokenKind::Sigil | TokenKind::Marker | TokenKind::Colon => {}
            },
            Child::Node(n) if n.kind() == NodeKind::TitleEntry => {
                in_stray = false;
                let first = slots.slots.len();
                for c in n.children() {
                    match c {
                        Child::Node(k)
                            if matches!(k.kind(), NodeKind::TitleKey | NodeKind::TitleValue) =>
                        {
                            slots.open(pieces(doc, k));
                        }
                        Child::Node(h) => slots.push_hidden(hidden(doc, h)),
                        Child::Token(t) if t.kind() == TokenKind::LineBreak => slots.end_line(),
                        Child::Token(_) => {}
                    }
                }
                entries.push((first, slots.slots.len() - first, n.span()));
            }
            Child::Node(h) => slots.push_hidden(hidden(doc, h)),
        }
    }
    strays.extend(slots.flush());
    let mut texts = slots.finish();
    let entries = entries
        .into_iter()
        .map(|(first, count, span)| TitleEntry {
            key: take(&mut texts, first),
            values: (first + 1..first + count)
                .map(|i| take(&mut texts, i))
                .collect(),
            span: Some(span),
        })
        .collect();
    let stray_lines = strays.into_iter().map(|i| take(&mut texts, i)).collect();
    TitlePage {
        entries,
        stray_lines,
        span: Some(node.span()),
    }
}

/// A Lyrics run: one text per `~` line. A line of hidden spans joins the line before it.
fn lyrics(doc: &Document, node: Node<'_>) -> Lyrics {
    let mut slots = Slots::default();
    let mut in_line = false;
    for child in node.children() {
        match child {
            Child::Token(t) => match t.kind() {
                TokenKind::Sigil => {
                    slots.open(Vec::new());
                    in_line = true;
                }
                TokenKind::Text | TokenKind::Whitespace if in_line => slots.push(text_piece(t)),
                TokenKind::LineBreak => {
                    in_line = false;
                    slots.end_line();
                }
                _ => {}
            },
            Child::Node(n) => slots.push_hidden(hidden(doc, n)),
        }
    }
    slots.flush();
    Lyrics {
        lines: slots.finish(),
    }
}

/// A Dialogue block: its cue and its lines. `unpaired_caret` starts as "the cue has a caret";
/// [`pair`] clears it for the blocks it joins.
fn dialogue(doc: &Document, node: Node<'_>) -> DialogueBlock {
    let mut head = None;
    let mut caret = false;
    let mut items = Vec::new();
    // Stray Whitespace tokens and line breaks directly under the block carry nothing.
    for child in node.children() {
        let Child::Node(n) = child else { continue };
        match n.kind() {
            NodeKind::CharacterCue if head.is_none() => {
                caret = n
                    .children()
                    .any(|c| matches!(c, Child::Node(k) if k.kind() == NodeKind::Caret));
                head = Some(cue(doc, n));
            }
            NodeKind::Dialogue => items.push(DialogueItem::Line(leaf(doc, n))),
            NodeKind::Parenthetical => items.push(DialogueItem::Parenthetical(leaf(doc, n))),
            NodeKind::Lyrics
            | NodeKind::Section
            | NodeKind::Synopsis
            | NodeKind::PageBreak
            | NodeKind::Note
            | NodeKind::Boneyard => {
                items.extend(element(doc, n).map(|e| DialogueItem::Interjection(Box::new(e))));
            }
            _ => {}
        }
    }
    DialogueBlock {
        cue: head.unwrap_or(Cue {
            name: Text::default(),
            extensions: Vec::new(),
            forced: false,
            span: None,
        }),
        items,
        unpaired_caret: caret,
    }
}

/// Join each caret block to its partner (OFF-11): the nearest earlier Dialogue block, when only
/// Sections, Synopses, Page Breaks, Notes and Boneyards come between and the partner is not
/// already a right half. Those lines then end `left.items`. One forward scan: the only state is
/// the index of the block that can still take a partner. A caret block that finds none stays
/// unpaired, and can itself take the next one.
fn pair(items: Vec<Item>) -> Vec<Item> {
    let mut out: Vec<Item> = Vec::with_capacity(items.len());
    let mut partner: Option<usize> = None;
    for Item { block, span } in items {
        let block = match block {
            Block::Element(Element::Dialogue(right)) => {
                let at = partner.take().filter(|_| right.unpaired_caret);
                let joined = match at {
                    Some(i) => join(&mut out, i, right, span),
                    None => Item {
                        block: Block::Element(Element::Dialogue(right)),
                        span,
                    },
                };
                if matches!(joined.block, Block::Element(Element::Dialogue(_))) {
                    partner = Some(out.len());
                }
                out.push(joined);
                continue;
            }
            Block::Element(
                e @ (Element::Section(_)
                | Element::Synopsis(_)
                | Element::PageBreak(_)
                | Element::Note(_)
                | Element::Boneyard(_)),
            ) => Block::Element(e),
            other => {
                partner = None;
                other
            }
        };
        out.push(Item { block, span });
    }
    out
}

/// Pair the block at `out[i]` with `right`, moving the lines after it into its items. The result
/// is the pair; if `out[i]` is not a Dialogue block (it always is), `out` is left as it was and
/// the result is `right` alone.
fn join(out: &mut Vec<Item>, i: usize, mut right: DialogueBlock, span: Span) -> Item {
    let skipped = out.split_off((i + 1).min(out.len()));
    match out.pop() {
        Some(Item {
            block: Block::Element(Element::Dialogue(mut left)),
            span: left_span,
        }) => {
            left.unpaired_caret = false;
            right.unpaired_caret = false;
            left.items
                .extend(skipped.into_iter().filter_map(|s| match s.block {
                    Block::Element(e) => Some(DialogueItem::Interjection(Box::new(e))),
                    Block::Heading(..) => None,
                }));
            Item {
                block: Block::Element(Element::DualDialogue(DualDialogue { left, right })),
                span: Span::new(left_span.start, span.end),
            }
        }
        other => {
            out.extend(other);
            out.extend(skipped);
            Item {
                block: Block::Element(Element::Dialogue(right)),
                span,
            }
        }
    }
}

fn cue(doc: &Document, node: Node<'_>) -> Cue {
    let mut slots = Slots::default();
    let mut name = None;
    let mut extensions = Vec::new();
    let mut forced = false;
    for child in node.children() {
        match child {
            Child::Token(t) if t.kind() == TokenKind::Sigil => forced = true,
            Child::Token(_) => {}
            Child::Node(n) => match n.kind() {
                NodeKind::CharacterName => name = Some(slots.open(pieces(doc, n))),
                NodeKind::Extension => extensions.push(slots.open(pieces(doc, n))),
                _ => slots.push_hidden(hidden(doc, n)),
            },
        }
    }
    let name = name.or_else(|| slots.flush());
    let mut texts = slots.finish();
    Cue {
        name: name.map(|i| take(&mut texts, i)).unwrap_or_default(),
        extensions: extensions
            .into_iter()
            .map(|i| Extension {
                text: take(&mut texts, i),
            })
            .collect(),
        forced,
        span: Some(node.span()),
    }
}

/// A scene whose heading has been seen and whose end has not.
struct OpenScene {
    heading: Heading,
    number: Option<SceneNumber>,
    span: Span,
    /// The Sections and Synopses directly before the heading.
    before: Vec<(Element, Span)>,
    body: Vec<(Element, Span)>,
}

fn is_outline(e: &Element) -> bool {
    matches!(e, Element::Section(_) | Element::Synopsis(_))
}

/// Sort the items into the preamble and the scenes.
fn group(items: Vec<Item>) -> (Preamble, Vec<Scene>) {
    let mut preamble = Vec::new();
    let mut scenes = Vec::new();
    let mut open: Option<OpenScene> = None;
    for Item { block, span } in items {
        let sink = match &mut open {
            Some(s) => &mut s.body,
            None => &mut preamble,
        };
        match block {
            Block::Element(e) => sink.push((e, span)),
            Block::Heading(heading, number) => {
                // The trailing run of Sections and Synopses moves to the new scene. Each
                // element moves at most once, so the scan back stays linear overall.
                let k = sink
                    .iter()
                    .rposition(|(e, _)| !is_outline(e))
                    .map_or(0, |p| p + 1);
                let before = sink.split_off(k);
                scenes.extend(open.take().map(OpenScene::finish));
                open = Some(OpenScene {
                    heading,
                    number,
                    span,
                    before,
                    body: Vec::new(),
                });
            }
        }
    }
    scenes.extend(open.map(OpenScene::finish));
    let preamble = Preamble {
        elements: preamble.into_iter().map(|(e, _)| e).collect(),
    };
    (preamble, scenes)
}

impl OpenScene {
    fn finish(self) -> Scene {
        let start = self
            .before
            .first()
            .map_or(self.span.start, |(_, s)| s.start);
        let end = self.body.last().map_or(self.span.end, |(_, s)| s.end);
        let mut sections = Vec::new();
        let mut synopses = Vec::new();
        let mut elements = Vec::new();
        for (e, _) in self.before {
            match e {
                Element::Section(s) => sections.push(s),
                Element::Synopsis(s) => synopses.push(s),
                other => elements.push(other),
            }
        }
        // The Synopses directly after the heading.
        let mut leading = true;
        for (e, _) in self.body {
            match e {
                Element::Synopsis(s) if leading => synopses.push(s),
                other => {
                    leading = false;
                    elements.push(other);
                }
            }
        }
        Scene {
            heading: self.heading,
            number: self.number,
            sections,
            synopses,
            elements,
            span: Some(Span::new(start, end)),
        }
    }
}
