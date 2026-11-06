//! The parse driver. For now every paragraph of text lines is one `Action`, and Notes and Boneyard
//! are hidden nodes inside it or at the top level.

use crate::hidden::{self, Logical};
use crate::layout::{self, Part};
use crate::lines;
use crate::tree::{Document, NodeKind, Span, TokenKind, TreeBuilder};

/// A line's leading whitespace, text and trailing whitespace, in visible offsets.
fn line_parts(visible: &str) -> Vec<Part> {
    let body = visible.trim_matches([' ', '\t']);
    let lead = visible.len() - visible.trim_start_matches([' ', '\t']).len();
    let trail = lead + body.len();
    [
        Part::Tok(TokenKind::Whitespace, 0..lead),
        Part::Tok(TokenKind::Text, lead..trail),
        Part::Tok(TokenKind::Whitespace, trail..visible.len()),
    ]
    .into_iter()
    .filter(|p| !matches!(p, Part::Tok(_, r) if r.is_empty()))
    .collect()
}

/// Which lines belong to an Action: a non-blank line with a text line at or before it and at or
/// after it, with no blank line between (decision 2: invisible lines at a paragraph's ends are
/// top-level).
fn action_members(lines: &[Logical]) -> Vec<bool> {
    let is_text = |l: &Logical| !l.is_blank() && !l.is_invisible();
    let mut member = vec![false; lines.len()];
    let mut seen = false;
    for (m, l) in member.iter_mut().zip(lines) {
        seen = !l.is_blank() && (seen || is_text(l));
        *m = seen;
    }
    let mut ahead = false;
    for (m, l) in member.iter_mut().zip(lines).rev() {
        ahead = !l.is_blank() && (ahead || is_text(l));
        *m &= ahead;
    }
    member
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
    let physical = lines::physical(src, from);
    let hidden = hidden::scan(src, from, &physical);
    let lines = hidden::logical(src, &physical, &hidden);
    let member = action_members(&lines);
    for (i, line) in lines.iter().enumerate() {
        if member[i] && !(i > 0 && member[i - 1]) {
            b.start_node(NodeKind::Action);
        }
        layout::emit_line(&mut b, src, line, &line_parts(&line.visible), &hidden);
        // The break after a paragraph's last line belongs to the Gap, not to the Action.
        if member[i] && !member.get(i + 1).copied().unwrap_or(false) {
            b.finish_node();
        }
        if let Some(brk) = line.brk {
            b.token(TokenKind::LineBreak, brk);
        }
    }
    b.finish(src)
}
