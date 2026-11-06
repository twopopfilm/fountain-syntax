//! The parse driver. For now every run of non-blank lines is one `Action`.

use crate::lines::{self, Line};
use crate::tree::{Document, NodeKind, Span, TokenKind, TreeBuilder};

/// Emits the line's leading whitespace, text and trailing whitespace as tokens.
fn line_tokens(b: &mut TreeBuilder, src: &str, text: Span) {
    let line = &src[text.start..text.end];
    let body = line.trim_matches([' ', '\t']);
    let lead = line.len() - line.trim_start_matches([' ', '\t']).len();
    let body_start = text.start + lead;
    b.token(TokenKind::Whitespace, Span::new(text.start, body_start));
    b.token(
        TokenKind::Text,
        Span::new(body_start, body_start + body.len()),
    );
    b.token(
        TokenKind::Whitespace,
        Span::new(body_start + body.len(), text.end),
    );
}

pub(crate) fn document(src: &str) -> Document {
    let mut b = TreeBuilder::new();
    let from = if src.starts_with('\u{FEFF}') {
        let bom = Span::new(0, '\u{FEFF}'.len_utf8());
        b.bom(bom);
        bom.end
    } else {
        0
    };
    let lines = lines::physical(src, from);
    let blank = |l: &Line| lines::is_blank(&src[l.text.start..l.text.end]);
    let mut in_action = false;
    for (i, line) in lines.iter().enumerate() {
        if blank(line) {
            b.token(TokenKind::Whitespace, line.text);
        } else {
            if !in_action {
                b.start_node(NodeKind::Action);
                in_action = true;
            }
            line_tokens(&mut b, src, line.text);
            // The break after the run's last line belongs to the Gap, not to the Action.
            if lines.get(i + 1).is_some_and(|next| !blank(next)) {
                if let Some(brk) = line.brk {
                    b.token(TokenKind::LineBreak, brk);
                }
                continue;
            }
            b.finish_node();
            in_action = false;
        }
        if let Some(brk) = line.brk {
            b.token(TokenKind::LineBreak, brk);
        }
    }
    b.finish(src)
}
