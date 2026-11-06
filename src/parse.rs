//! The parse driver. A block pass groups the logical lines into items, and each item is emitted
//! into the tree line by line. Notes and Boneyard are hidden nodes placed by the layout layer.

use std::ops::Range;

use crate::classify::{self, LineShape};
use crate::hidden::{self, Logical};
use crate::layout::{self, Part};
use crate::lines;
use crate::tree::{Document, NodeKind, Span, TokenKind, TreeBuilder};

/// What a logical line is, for grouping.
#[derive(Clone, Copy, Debug)]
enum Role {
    Blank,
    /// Only hidden spans and whitespace (SPEC 3.4, rule 3).
    Invisible,
    Shaped(LineShape),
}

/// A run of logical lines emitted together: inside `wrap` when it is set, else a single line whose
/// own parts make its node, if it has one.
struct Item {
    wrap: Option<NodeKind>,
    lines: Range<usize>,
}

fn role(line: &Logical) -> Role {
    if line.is_blank() {
        Role::Blank
    } else if line.is_invisible() {
        Role::Invisible
    } else {
        Role::Shaped(classify::shape(
            line.visible.trim_start_matches([' ', '\t']),
        ))
    }
}

/// Lines that continue an Action paragraph. A forced `@` cue is plain text for now.
fn is_action_text(shape: LineShape) -> bool {
    matches!(
        shape,
        LineShape::Plain | LineShape::ForcedAction | LineShape::ForcedCue
    )
}

/// Section, Synopsis and Page Break lines are transparent (SPEC 3.4, rule 3).
fn is_outline(shape: LineShape) -> bool {
    matches!(
        shape,
        LineShape::Section(_) | LineShape::Synopsis | LineShape::PageBreak
    )
}

/// Whether a line counts when framing looks at its neighbours: invisible lines and Section, Synopsis
/// and Page Break lines are transparent (SPEC 3.4, rule 3).
fn is_significant(role: Role) -> bool {
    match role {
        Role::Invisible => false,
        Role::Shaped(s) => !is_outline(s),
        Role::Blank => true,
    }
}

/// The nearest significant line before `i`; the start of the document counts as blank.
fn prev_significant(roles: &[Role], i: usize) -> Role {
    let before = roles[..i].iter().rev();
    before
        .copied()
        .find(|&r| is_significant(r))
        .unwrap_or(Role::Blank)
}

/// The nearest significant line after `i`; the end of the document counts as blank.
fn next_significant(roles: &[Role], i: usize) -> Role {
    let after = roles[i + 1..].iter();
    after
        .copied()
        .find(|&r| is_significant(r))
        .unwrap_or(Role::Blank)
}

/// Demotes the lines that are headings only by prefix to plain text unless a blank line comes before
/// and after them (SPEC 4.2).
fn frame(roles: &mut [Role]) {
    for i in 0..roles.len() {
        if matches!(roles[i], Role::Shaped(LineShape::PrefixHeading))
            && !(matches!(prev_significant(roles, i), Role::Blank)
                && matches!(next_significant(roles, i), Role::Blank))
        {
            roles[i] = Role::Shaped(LineShape::Plain);
        }
    }
}

/// Whether `role` continues the open `wrap` element.
fn is_member(wrap: NodeKind, role: Role) -> bool {
    match (wrap, role) {
        (NodeKind::Action, Role::Shaped(s)) => is_action_text(s),
        (NodeKind::Lyrics, Role::Shaped(s)) => s == LineShape::Lyrics,
        _ => false,
    }
}

/// Whether `role` may sit inside the open `wrap` element between two members. It joins the element
/// only when a member follows; otherwise it is top-level, as are leading ones.
fn is_skippable(wrap: NodeKind, role: Role) -> bool {
    match (wrap, role) {
        (_, Role::Invisible) => true,
        (NodeKind::Action, Role::Shaped(s)) => is_outline(s),
        _ => false,
    }
}

/// The block pass: groups lines into the items the tree is made of.
fn items(roles: &[Role]) -> Vec<Item> {
    let mut out = Vec::new();
    // The open element and its first line; `end` is one past its last member line, and the lines
    // from `end` to the current one are skippable lines waiting for a member.
    let mut open: Option<(NodeKind, usize)> = None;
    let mut end = 0;
    for (i, &role) in roles.iter().enumerate() {
        if let Some((wrap, _)) = open {
            if is_member(wrap, role) {
                end = i + 1;
                continue;
            }
            if is_skippable(wrap, role) {
                continue;
            }
        }
        close(&mut out, open.take(), end, i);
        let wrap = match role {
            Role::Shaped(LineShape::Lyrics) => Some(NodeKind::Lyrics),
            Role::Shaped(s) if is_action_text(s) => Some(NodeKind::Action),
            _ => None,
        };
        match wrap {
            Some(wrap) => {
                open = Some((wrap, i));
                end = i + 1;
            }
            None => out.push(Item {
                wrap: None,
                lines: i..i + 1,
            }),
        }
    }
    close(&mut out, open, end, roles.len());
    out
}

/// Ends the open element after its last member, `end`. The skippable lines up to `to` that no
/// member followed are top-level.
fn close(out: &mut Vec<Item>, open: Option<(NodeKind, usize)>, end: usize, to: usize) {
    let Some((wrap, start)) = open else { return };
    out.push(Item {
        wrap: Some(wrap),
        lines: start..end,
    });
    out.extend((end..to).map(|i| Item {
        wrap: None,
        lines: i..i + 1,
    }));
}

/// The leading whitespace, an optional `mark` of `(kind, byte length)` right after it, the text up
/// to the trailing whitespace, and the trailing whitespace. Empty parts are left out.
fn text_parts(visible: &str, mark: Option<(TokenKind, usize)>) -> Vec<Part> {
    let body = visible.trim_matches([' ', '\t']);
    let lead = visible.len() - visible.trim_start_matches([' ', '\t']).len();
    let trail = lead + body.len();
    let after = lead + mark.map_or(0, |(_, n)| n);
    let mut parts = vec![Part::Tok(TokenKind::Whitespace, 0..lead)];
    if let Some((kind, _)) = mark {
        parts.push(Part::Tok(kind, lead..after));
    }
    parts.push(Part::Tok(TokenKind::Text, after..trail));
    parts.push(Part::Tok(TokenKind::Whitespace, trail..visible.len()));
    parts.retain(|p| !matches!(p, Part::Tok(_, r) if r.is_empty()));
    parts
}

/// A marker of `n` bytes after the leading whitespace, then optional whitespace, text and trailing
/// whitespace: the layout of a Section and a Synopsis.
fn marked_parts(visible: &str, n: usize) -> Vec<Part> {
    let lead = visible.len() - visible.trim_start_matches([' ', '\t']).len();
    let trail = lead + visible.trim_matches([' ', '\t']).len();
    let after = lead + n;
    let rest = &visible[after..trail];
    let gap = after + rest.len() - rest.trim_start_matches([' ', '\t']).len();
    let mut parts = vec![
        Part::Tok(TokenKind::Whitespace, 0..lead),
        Part::Tok(TokenKind::Marker, lead..after),
        Part::Tok(TokenKind::Whitespace, after..gap),
        Part::Tok(TokenKind::Text, gap..trail),
        Part::Tok(TokenKind::Whitespace, trail..visible.len()),
    ];
    parts.retain(|p| !matches!(p, Part::Tok(_, r) if r.is_empty()));
    parts
}

/// `>`, optional whitespace, text, optional whitespace and `<`, inside leading and trailing
/// whitespace. Only the outermost marks are markers.
fn centered_parts(visible: &str) -> Vec<Part> {
    let lead = visible.len() - visible.trim_start_matches([' ', '\t']).len();
    let trail = lead + visible.trim_matches([' ', '\t']).len();
    let inner = &visible[lead + 1..trail - 1];
    let text_start = lead + 1 + inner.len() - inner.trim_start_matches([' ', '\t']).len();
    let text_end = trail - 1 - (inner.len() - inner.trim_end_matches([' ', '\t']).len());
    let mut parts = vec![
        Part::Tok(TokenKind::Whitespace, 0..lead),
        Part::Tok(TokenKind::Marker, lead..lead + 1),
        Part::Tok(TokenKind::Whitespace, lead + 1..text_start),
        Part::Tok(TokenKind::Text, text_start..text_end.max(text_start)),
        Part::Tok(TokenKind::Whitespace, text_end.max(text_start)..trail - 1),
        Part::Tok(TokenKind::Marker, trail - 1..trail),
        Part::Tok(TokenKind::Whitespace, trail..visible.len()),
    ];
    parts.retain(|p| !matches!(p, Part::Tok(_, r) if r.is_empty()));
    parts
}

/// `[ws] Sigil"."? Text [ws] SceneNumber(Marker Text Marker)? [ws]`: the layout of a Scene Heading.
fn heading_parts(visible: &str, forced: bool) -> Vec<Part> {
    let lead = visible.len() - visible.trim_start_matches([' ', '\t']).len();
    let end = visible.trim_end_matches([' ', '\t']).len();
    let text = lead + usize::from(forced);
    let number = classify::scene_number(&visible[..end]);
    let text_end = number.as_ref().map_or(end, |(r, _)| {
        visible[..r.start].trim_end_matches([' ', '\t']).len()
    });
    let mut parts = vec![Part::Tok(TokenKind::Whitespace, 0..lead)];
    if forced {
        parts.push(Part::Tok(TokenKind::Sigil, lead..text));
    }
    parts.push(Part::Tok(TokenKind::Text, text..text_end));
    if let Some((r, locked)) = number {
        let open = r.start + if locked { 2 } else { 1 };
        parts.push(Part::Tok(TokenKind::Whitespace, text_end..r.start));
        parts.push(Part::Node(
            NodeKind::SceneNumber,
            vec![
                Part::Tok(TokenKind::Marker, r.start..open),
                Part::Tok(TokenKind::Text, open..r.end - 1),
                Part::Tok(TokenKind::Marker, r.end - 1..r.end),
            ],
        ));
    }
    parts.push(Part::Tok(TokenKind::Whitespace, end..visible.len()));
    parts.retain(|p| !matches!(p, Part::Tok(_, r) if r.is_empty()));
    parts
}

/// The layout of one logical line.
fn parts(visible: &str, role: Role) -> Vec<Part> {
    let sigil = |n| Some((TokenKind::Sigil, n));
    let node = |kind, parts| vec![Part::Node(kind, parts)];
    let Role::Shaped(shape) = role else {
        return text_parts(visible, None);
    };
    match shape {
        LineShape::Plain | LineShape::ForcedCue => text_parts(visible, None),
        LineShape::ForcedHeading | LineShape::PrefixHeading => node(
            NodeKind::SceneHeading,
            heading_parts(visible, shape == LineShape::ForcedHeading),
        ),
        LineShape::Lyrics | LineShape::ForcedAction => text_parts(visible, sigil(1)),
        #[cfg(feature = "shot")]
        LineShape::Shot => node(NodeKind::Shot, text_parts(visible, sigil(2))),
        LineShape::ForcedTransition => node(NodeKind::Transition, text_parts(visible, sigil(1))),
        LineShape::Centered => node(NodeKind::Centered, centered_parts(visible)),
        LineShape::Section(depth) => node(NodeKind::Section, marked_parts(visible, depth)),
        LineShape::Synopsis => node(NodeKind::Synopsis, marked_parts(visible, 1)),
        LineShape::PageBreak => node(
            NodeKind::PageBreak,
            text_parts(visible, Some((TokenKind::Marker, visible.trim().len()))),
        ),
    }
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
    let mut roles: Vec<Role> = lines.iter().map(role).collect();
    frame(&mut roles);
    for item in items(&roles) {
        if let Some(kind) = item.wrap {
            b.start_node(kind);
        }
        for i in item.lines.clone() {
            let line = &lines[i];
            layout::emit_line(&mut b, src, line, &parts(&line.visible, roles[i]), &hidden);
            // The break after an element's last line belongs to the Gap, not to the element.
            if item.wrap.is_some() && i + 1 == item.lines.end {
                b.finish_node();
            }
            if let Some(brk) = line.brk {
                b.token(TokenKind::LineBreak, brk);
            }
        }
    }
    b.finish(src)
}
