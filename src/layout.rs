//! Line layout: turns a logical line's structure, given in visible offsets, into tree tokens and
//! nodes, and places its hidden spans by the rule of the plan's "Hidden-node placement".

use std::ops::Range;

use crate::hidden::{Hidden, Logical, Piece};
use crate::lines;
use crate::tree::{Span, TokenKind, TreeBuilder};

/// One part of a line's layout, in offsets into [`Logical::visible`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    /// A token over a non-empty visible range.
    Tok(TokenKind, Range<usize>),
    /// A node over its parts.
    Node(crate::tree::NodeKind, Vec<Part>),
}

/// Emits `line` as `parts`, which cover `0..line.visible.len()` in order with no empty range.
///
/// A hidden piece at visible offset `v` splits a token that strictly contains `v`. Otherwise it goes
/// at the level of the innermost node that strictly contains `v`, otherwise at the line's level.
pub(crate) fn emit_line(
    b: &mut TreeBuilder,
    src: &str,
    line: &Logical,
    parts: &[Part],
    hidden: &[Hidden],
) {
    let mut c = Cursor {
        b,
        src,
        line,
        hidden,
        piece: 0,
        off: 0,
    };
    c.parts(parts);
    c.flush();
}

/// Walks a line's pieces while its parts are emitted.
struct Cursor<'a> {
    b: &'a mut TreeBuilder,
    src: &'a str,
    line: &'a Logical,
    hidden: &'a [Hidden],
    /// The current piece, and the bytes of it already emitted.
    piece: usize,
    off: usize,
}

impl Cursor<'_> {
    fn parts(&mut self, parts: &[Part]) {
        for part in parts {
            // Hidden pieces at a part's start belong to this level, not inside the part.
            self.flush();
            match part {
                Part::Tok(kind, r) => self.take(*kind, r.len()),
                Part::Node(kind, kids) => {
                    self.b.start_node(*kind);
                    self.parts(kids);
                    self.b.finish_node();
                }
            }
        }
    }

    /// Emits the hidden pieces at the cursor and stops at the next unread visible byte.
    fn flush(&mut self) {
        while let Some(&piece) = self.line.pieces.get(self.piece) {
            match piece {
                Piece::Visible(s) if self.off < s.len() => return,
                Piece::Visible(_) => {}
                Piece::Hidden(h) => emit_hidden(self.b, self.src, self.hidden[h]),
            }
            self.piece += 1;
            self.off = 0;
        }
    }

    /// Emits the next `n` visible bytes as `kind` tokens, split by the hidden pieces inside them.
    fn take(&mut self, kind: TokenKind, mut n: usize) {
        while n > 0 {
            self.flush();
            let Some(&Piece::Visible(s)) = self.line.pieces.get(self.piece) else {
                unreachable!("the parts cover more than the visible text");
            };
            let at = s.start + self.off;
            let len = n.min(s.end - at);
            self.b.token(kind, Span::new(at, at + len));
            self.off += len;
            n -= len;
        }
    }
}

/// `Note(Marker"[[" (Text|LineBreak)* Marker"]]")`, and the same for Boneyard.
fn emit_hidden(b: &mut TreeBuilder, src: &str, h: Hidden) {
    let Span { start, end } = h.span;
    b.start_node(h.kind);
    b.token(TokenKind::Marker, Span::new(start, start + 2));
    // The closer follows the inside, so a `\r\n` is never cut at its end.
    for l in lines::physical(&src[..end - 2], start + 2) {
        b.token(TokenKind::Text, l.text);
        if let Some(brk) = l.brk {
            b.token(TokenKind::LineBreak, brk);
        }
    }
    b.token(TokenKind::Marker, Span::new(end - 2, end));
    b.finish_node();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hidden;
    use crate::tree::{Child, Node, NodeKind};

    fn dump(node: Node<'_>) -> String {
        let parts: Vec<String> = node
            .children()
            .map(|c| match c {
                Child::Node(n) => dump(n),
                Child::Token(t) => format!("{:?}{:?}", t.kind(), t.text()),
            })
            .collect();
        format!("{:?}({})", node.kind(), parts.join(" "))
    }

    /// Emits `src` as one Action line laid out as `parts`.
    fn emit(src: &str, parts: &[Part]) -> String {
        let lines = lines::physical(src, 0);
        let hidden = hidden::scan(src, 0, &lines);
        let logical = hidden::logical(src, &lines, &hidden);
        assert_eq!(logical.len(), 1);
        let mut b = TreeBuilder::new();
        b.start_node(NodeKind::Action);
        emit_line(&mut b, src, &logical[0], parts, &hidden);
        b.finish_node();
        dump(b.finish(src).root())
    }

    fn name(parts: Vec<Part>) -> Part {
        Part::Node(NodeKind::CharacterName, parts)
    }

    /// Hidden-node placement: at a boundary strictly inside a node, the hidden node is its child.
    #[test]
    fn hidden_placement_between_parts_of_a_node() {
        let parts = [name(vec![
            Part::Tok(TokenKind::Text, 0..2),
            Part::Tok(TokenKind::Whitespace, 2..3),
        ])];
        assert_eq!(
            emit("MA/*x*/ ", &parts),
            r#"Document(Action(CharacterName(Text"MA" Boneyard(Marker"/*" Text"x" Marker"*/") Whitespace" ")))"#
        );
    }

    /// Hidden-node placement: a hidden span strictly inside a nested token splits it.
    #[test]
    fn hidden_placement_splits_token_in_node() {
        let parts = [name(vec![Part::Tok(TokenKind::Text, 0..4)])];
        assert_eq!(
            emit("MA/*x*/RA", &parts),
            r#"Document(Action(CharacterName(Text"MA" Boneyard(Marker"/*" Text"x" Marker"*/") Text"RA")))"#
        );
    }

    /// Hidden-node placement: at a node's edges, the hidden nodes go at the level around it.
    #[test]
    fn hidden_placement_node_edges() {
        let parts = [
            Part::Tok(TokenKind::Whitespace, 0..1),
            name(vec![Part::Tok(TokenKind::Text, 1..3)]),
            Part::Tok(TokenKind::Whitespace, 3..4),
        ];
        assert_eq!(
            emit(" [[a]]MA[[b]] [[c]]", &parts),
            r#"Document(Action(Whitespace" " Note(Marker"[[" Text"a" Marker"]]") CharacterName(Text"MA") Note(Marker"[[" Text"b" Marker"]]") Whitespace" " Note(Marker"[[" Text"c" Marker"]]")))"#
        );
    }
}
