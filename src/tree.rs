//! The lossless syntax tree (SPEC 8.2 to 8.4): public types, the arena behind them and the builder.

/// Half-open byte range into [`Document::source`], on character boundaries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Span {
    /// First byte of the range.
    pub start: usize,
    /// One past the last byte of the range.
    pub end: usize,
}

impl Span {
    /// The range `start..end`.
    pub fn new(start: usize, end: usize) -> Span {
        Span { start, end }
    }

    /// The number of bytes in the range.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// True when the range holds no bytes.
    pub fn is_empty(&self) -> bool {
        self.end <= self.start
    }
}

/// What a node is. New kinds may be added.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NodeKind {
    /// The root.
    Document,
    /// The line breaks and blank lines around top-level nodes: before the first, between two, and
    /// after the last. An empty source has one empty Gap.
    Gap,
    /// The title page block.
    TitlePage,
    /// One `Key: value` entry of the title page.
    TitleEntry,
    /// A title page key.
    TitleKey,
    /// The text of one line of a title page value: after the colon, or one continuation line.
    TitleValue,
    /// A Scene Heading.
    SceneHeading,
    /// A scene number inside a Scene Heading.
    SceneNumber,
    /// An Action paragraph.
    Action,
    /// A Character cue with its Parentheticals and Dialogue, and any Lyrics, Section, Synopsis, Page
    /// Break, Note or Boneyard line between them.
    DialogueBlock,
    /// A Character cue line.
    CharacterCue,
    /// The name part of a Character cue.
    CharacterName,
    /// A cue extension such as `(V.O.)`.
    Extension,
    /// The `^` of a dual-dialogue cue.
    Caret,
    /// A Parenthetical line.
    Parenthetical,
    /// A Dialogue line.
    Dialogue,
    /// A Transition.
    Transition,
    /// Centered text.
    Centered,
    /// A run of consecutive Lyrics lines.
    Lyrics,
    /// A Section line.
    Section,
    /// A Synopsis line.
    Synopsis,
    /// A Page Break line.
    PageBreak,
    /// A `[[ ]]` Note.
    Note,
    /// A `/* */` Boneyard.
    Boneyard,
    /// A `!!` Shot line (the `shot` feature).
    #[cfg(feature = "shot")]
    Shot,
}

/// What a token is. New kinds may be added.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenKind {
    /// A leading byte-order mark.
    Bom,
    /// Content text.
    Text,
    /// Spaces and tabs.
    Whitespace,
    /// `\n`, `\r\n` or `\r`.
    LineBreak,
    /// A forcing character such as `.` or `@`.
    Sigil,
    /// A structural marker such as `#`, `=`, `[[`, `^`, the `>` and `<` of centered text, or the
    /// parentheses of an extension.
    Marker,
    /// The colon of a title page key.
    Colon,
}

#[derive(Clone, Copy, Debug)]
enum ChildRef {
    Node(usize),
    Token(usize),
}

#[derive(Clone, Debug)]
struct NodeData {
    kind: NodeKind,
    span: Span,
    parent: Option<usize>,
    children: Vec<ChildRef>,
}

#[derive(Clone, Copy, Debug)]
struct TokenData {
    kind: TokenKind,
    span: Span,
}

/// A parsed screenplay: the source text and a tree over it that reproduces it exactly.
#[derive(Clone, Debug)]
pub struct Document {
    source: String,
    nodes: Vec<NodeData>,
    tokens: Vec<TokenData>,
}

impl Document {
    /// The text that was parsed.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// The root node, of kind [`NodeKind::Document`].
    pub fn root(&self) -> Node<'_> {
        Node { doc: self, id: 0 }
    }
}

/// A handle to a node of a [`Document`].
#[derive(Clone, Copy)]
pub struct Node<'a> {
    doc: &'a Document,
    id: usize,
}

impl<'a> Node<'a> {
    fn data(&self) -> &'a NodeData {
        &self.doc.nodes[self.id]
    }

    /// What the node is.
    pub fn kind(&self) -> NodeKind {
        self.data().kind
    }

    /// The bytes the node covers, from its first child's start to its last child's end. An empty
    /// source's Gap has no children and spans 0..0.
    pub fn span(&self) -> Span {
        self.data().span
    }

    /// The source text of the node.
    pub fn text(&self) -> &'a str {
        let s = self.span();
        &self.doc.source[s.start..s.end]
    }

    /// The node this one is a child of; `None` for the root.
    pub fn parent(&self) -> Option<Node<'a>> {
        self.data().parent.map(|id| Node { doc: self.doc, id })
    }

    /// The nodes and tokens directly inside this node, in source order.
    pub fn children(&self) -> impl Iterator<Item = Child<'a>> + use<'a> {
        let doc = self.doc;
        self.data().children.iter().map(move |c| match *c {
            ChildRef::Node(id) => Child::Node(Node { doc, id }),
            ChildRef::Token(id) => {
                let TokenData { kind, span } = doc.tokens[id];
                Child::Token(Token {
                    kind,
                    span,
                    text: &doc.source[span.start..span.end],
                })
            }
        })
    }
}

impl PartialEq for Node<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.doc, other.doc) && self.id == other.id
    }
}

impl Eq for Node<'_> {}

impl std::fmt::Debug for Node<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Node")
            .field("kind", &self.kind())
            .field("span", &self.span())
            .finish()
    }
}

/// A child of a node: a node or a token.
#[derive(Clone, Copy, Debug)]
pub enum Child<'a> {
    /// A nested node.
    Node(Node<'a>),
    /// A leaf.
    Token(Token<'a>),
}

/// A leaf of the tree: a run of source text of one kind. Never empty.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    kind: TokenKind,
    span: Span,
    text: &'a str,
}

impl<'a> Token<'a> {
    /// What the token is.
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    /// The bytes the token covers.
    pub fn span(&self) -> Span {
        self.span
    }

    /// The source text of the token.
    pub fn text(&self) -> &'a str {
        self.text
    }
}

/// Builds a [`Document`] from a stream of tokens and nested nodes.
///
/// At the top level, a token opens a `Gap` if none is open and a node closes it, so callers never
/// manage Gaps. The caller must emit tokens in source order with no gaps between them.
pub(crate) struct TreeBuilder {
    nodes: Vec<NodeData>,
    tokens: Vec<TokenData>,
    /// Open nodes; the root is always at the bottom.
    stack: Vec<usize>,
    gap_open: bool,
}

impl TreeBuilder {
    pub(crate) fn new() -> TreeBuilder {
        TreeBuilder {
            nodes: vec![NodeData {
                kind: NodeKind::Document,
                span: Span::new(0, 0),
                parent: None,
                children: Vec::new(),
            }],
            tokens: Vec::new(),
            stack: vec![0],
            gap_open: false,
        }
    }

    fn open(&mut self, kind: NodeKind) {
        let id = self.nodes.len();
        let parent = *self.stack.last().expect("the root is always open");
        self.nodes.push(NodeData {
            kind,
            span: Span::new(0, 0),
            parent: Some(parent),
            children: Vec::new(),
        });
        self.nodes[parent].children.push(ChildRef::Node(id));
        self.stack.push(id);
    }

    fn close(&mut self) {
        let id = self.stack.pop().expect("a node is open");
        let node = &self.nodes[id];
        let bounds = |c: &ChildRef| match *c {
            ChildRef::Node(i) => self.nodes[i].span,
            ChildRef::Token(i) => self.tokens[i].span,
        };
        // Only the empty document's Gap is childless; its span stays 0..0.
        if let (Some(first), Some(last)) = (node.children.first(), node.children.last()) {
            let span = Span::new(bounds(first).start, bounds(last).end);
            self.nodes[id].span = span;
        }
    }

    /// Opens an element node. Closes the open Gap first.
    pub(crate) fn start_node(&mut self, kind: NodeKind) {
        if std::mem::take(&mut self.gap_open) {
            self.close();
        }
        self.open(kind);
    }

    /// Closes the innermost open element node.
    pub(crate) fn finish_node(&mut self) {
        self.close();
    }

    /// Adds a token to the innermost open node. An empty span adds nothing.
    pub(crate) fn token(&mut self, kind: TokenKind, span: Span) {
        if span.is_empty() {
            return;
        }
        if self.stack.len() == 1 {
            self.open(NodeKind::Gap);
            self.gap_open = true;
        }
        self.push_token(kind, span);
    }

    /// Adds the leading byte-order mark to the root. Call it before any other token.
    pub(crate) fn bom(&mut self, span: Span) {
        self.push_token(TokenKind::Bom, span);
    }

    fn push_token(&mut self, kind: TokenKind, span: Span) {
        let id = self.tokens.len();
        self.tokens.push(TokenData { kind, span });
        let parent = *self.stack.last().expect("the root is always open");
        self.nodes[parent].children.push(ChildRef::Token(id));
    }

    /// Closes everything and returns the document. A completely empty source gets one empty Gap.
    pub(crate) fn finish(mut self, source: &str) -> Document {
        if source.is_empty() {
            self.open(NodeKind::Gap);
            self.close();
        }
        while !self.stack.is_empty() {
            self.close();
        }
        Document {
            source: source.to_owned(),
            nodes: self.nodes,
            tokens: self.tokens,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Span;

    #[test]
    fn an_inverted_span_is_empty_and_does_not_panic() {
        let s = Span { start: 5, end: 2 };
        assert_eq!((s.len(), s.is_empty()), (0, true));
        let ok = Span::new(2, 5);
        assert_eq!((ok.len(), ok.is_empty()), (3, false));
    }
}
