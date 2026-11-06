#![allow(dead_code)] // each test crate uses a subset of these helpers

use fountain_syntax::{Child, Document, Node, NodeKind, Token, parse};

fn tokens<'a>(node: Node<'a>, out: &mut Vec<Token<'a>>) {
    for child in node.children() {
        match child {
            Child::Node(n) => tokens(n, out),
            Child::Token(t) => out.push(t),
        }
    }
}

fn check_node(doc: &Document, node: Node<'_>) {
    let src = doc.source();
    let span = node.span();
    assert!(
        src.is_char_boundary(span.start) && src.is_char_boundary(span.end),
        "{:?} span {span:?} is off a character boundary",
        node.kind()
    );
    let children: Vec<_> = node.children().collect();
    let (Some(first), Some(last)) = (children.first(), children.last()) else {
        assert!(
            node.kind() == NodeKind::Gap && src.is_empty(),
            "childless {:?}",
            node.kind()
        );
        assert_eq!((span.start, span.end), (0, 0));
        return;
    };
    let bounds = |c: &Child<'_>| match c {
        Child::Node(n) => n.span(),
        Child::Token(t) => t.span(),
    };
    assert_eq!(span.start, bounds(first).start, "{:?} start", node.kind());
    assert_eq!(span.end, bounds(last).end, "{:?} end", node.kind());
    for child in children {
        if let Child::Node(n) = child {
            assert_eq!(n.parent(), Some(node), "parent of {:?}", n.kind());
            check_node(doc, n);
        }
    }
}

/// Parses `src` and asserts the tree invariants of SPEC 8.3 (plus no zero-length tokens).
pub fn parse_checked(src: &str) -> Document {
    let doc = parse(src);
    assert_eq!(doc.source(), src);
    let root = doc.root();
    assert_eq!(root.kind(), NodeKind::Document);
    assert_eq!(root.parent(), None);
    let mut toks = Vec::new();
    tokens(root, &mut toks);
    let mut at = 0;
    let mut joined = String::new();
    for t in &toks {
        assert_eq!(t.span().start, at, "tokens are contiguous at {t:?}");
        assert!(!t.span().is_empty(), "zero-length token {t:?}");
        assert_eq!(t.text(), &src[t.span().start..t.span().end]);
        at = t.span().end;
        joined.push_str(t.text());
    }
    assert_eq!(at, src.len());
    assert_eq!(joined, src);
    assert_eq!((root.span().start, root.span().end), (0, src.len()));
    check_node(&doc, root);
    doc
}

/// The kinds of the root's child nodes, without the `Gap`s.
pub fn top(doc: &Document) -> Vec<NodeKind> {
    doc.root()
        .children()
        .filter_map(|c| match c {
            Child::Node(n) if n.kind() != NodeKind::Gap => Some(n.kind()),
            _ => None,
        })
        .collect()
}

/// `Kind(child child ...)`, each token as `Kind"text"`.
pub fn dump(node: Node<'_>) -> String {
    let parts: Vec<String> = node
        .children()
        .map(|c| match c {
            Child::Node(n) => dump(n),
            Child::Token(t) => format!("{:?}{:?}", t.kind(), t.text()),
        })
        .collect();
    format!("{:?}({})", node.kind(), parts.join(" "))
}

/// Every node of `kind`, depth-first.
pub fn find(doc: &Document, kind: NodeKind) -> Vec<Node<'_>> {
    fn walk<'a>(node: Node<'a>, kind: NodeKind, out: &mut Vec<Node<'a>>) {
        if node.kind() == kind {
            out.push(node);
        }
        for child in node.children() {
            if let Child::Node(n) = child {
                walk(n, kind, out);
            }
        }
    }
    let mut out = Vec::new();
    walk(doc.root(), kind, &mut out);
    out
}
