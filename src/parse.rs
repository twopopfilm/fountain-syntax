//! The parse driver. The title page is read first, then a forward pass gives each logical line its
//! role, a block pass groups the lines into items, and each item is emitted into the tree line by
//! line. Notes and Boneyard are hidden nodes placed by the layout layer.

use std::ops::Range;

use crate::classify::{self, LineShape};
use crate::diagnose;
use crate::hidden::{self, Hidden, Logical};
use crate::layout::{self, Part};
use crate::lines;
use crate::tree::{Document, NodeKind, Span, TokenKind, TreeBuilder};

/// What a logical line is, for grouping.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Role {
    Blank,
    /// Exactly two spaces: a Dialogue line inside a block, blank elsewhere (`OFF-1b`). [`frame`]
    /// resolves it.
    TwoSpaces,
    /// Only hidden spans and whitespace (SPEC 3.4, rule 3).
    Invisible,
    Shaped(LineShape),
    /// An automatic Transition. This role and the three after it are set by [`frame`].
    Transition,
    /// An automatic Character cue; a forced one stays `Shaped(LineShape::ForcedCue)`.
    Cue,
    Dialogue,
    Parenthetical,
    /// A title-page key line. This role and the two after it are set by [`title_page`].
    TitleKey,
    /// A title-page continuation line.
    TitleValue,
    /// A title-page line that is neither (`OFF-17d`).
    TitleStray,
}

/// One part of the tree: a single line, whose own parts make its node if it has one, or an element
/// node around its parts.
enum Item {
    Line(usize),
    Wrap(NodeKind, Vec<Item>),
}

impl Item {
    fn last_line(&self) -> usize {
        match self {
            Item::Line(i) => *i,
            Item::Wrap(_, kids) => kids.last().expect("an element has a line").last_line(),
        }
    }
}

fn role(line: &Logical) -> Role {
    if line.visible == "  " && line.is_blank() {
        Role::TwoSpaces
    } else if line.is_blank() {
        Role::Blank
    } else if line.is_invisible() {
        Role::Invisible
    } else {
        Role::Shaped(classify::shape(
            line.visible.trim_start_matches([' ', '\t']),
        ))
    }
}

/// Lines that continue an Action paragraph.
fn is_action_text(shape: LineShape) -> bool {
    matches!(shape, LineShape::Plain | LineShape::ForcedAction)
}

/// Section, Synopsis and Page Break lines are transparent (SPEC 3.4, rule 3).
pub(crate) fn is_outline(shape: LineShape) -> bool {
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
        _ => true,
    }
}

/// The nearest significant line before `i`; the start of the document counts as blank.
pub(crate) fn prev_significant(roles: &[Role], i: usize) -> Role {
    let before = roles[..i].iter().rev();
    before
        .copied()
        .find(|&r| is_significant(r))
        .unwrap_or(Role::Blank)
}

/// The nearest significant line after `i`; the end of the document counts as blank.
pub(crate) fn next_significant(roles: &[Role], i: usize) -> Role {
    let after = roles[i + 1..].iter();
    after
        .copied()
        .find(|&r| is_significant(r))
        .unwrap_or(Role::Blank)
}

/// Finds the title page (SPEC 4.1) and gives its lines their roles. It returns the line range of
/// the page, which ends at the first blank line, or `None` when there is no page.
///
/// Blank, invisible and outline (Section, Synopsis, Page Break) lines before the first key line are
/// leading trivia: the page starts at the first visible non-blank line (SPEC 3.5). A bare key starts
/// a page only when the next visible line is indented as a continuation. A continuation line needs
/// a current entry; a stray line ends it, and an invisible line does not.
fn title_page(roles: &mut [Role], lines: &[Logical]) -> Option<Range<usize>> {
    let is_blank = |r: &Role| matches!(r, Role::Blank | Role::TwoSpaces);
    let start = roles
        .iter()
        .position(|r| !is_blank(r) && is_significant(*r))?;
    let end = roles[start..]
        .iter()
        .position(is_blank)
        .map_or(roles.len(), |n| start + n);
    let trimmed = |i: usize| lines[i].visible.trim_start_matches([' ', '\t']);
    let first = classify::key_line(trimmed(start))?;
    let bare = trimmed(start)[first.colon + 1..]
        .trim_start_matches([' ', '\t'])
        .is_empty();
    let under = (start + 1..end).find(|&i| !matches!(roles[i], Role::Invisible));
    if bare && !under.is_some_and(|i| classify::continuation_indent(&lines[i].visible)) {
        return None;
    }
    let mut current = false;
    for i in start..end {
        if matches!(roles[i], Role::Invisible) {
            continue;
        }
        roles[i] = if current && classify::continuation_indent(&lines[i].visible) {
            Role::TitleValue
        } else if classify::key_line(trimmed(i)).is_some() {
            current = true;
            Role::TitleKey
        } else {
            current = false;
            Role::TitleStray
        };
    }
    Some(start..end)
}

/// Assigns the roles that depend on context, in one pass that knows whether a Dialogue block is
/// open (SPEC 4.0).
///
/// Inside a block every line that no sigil claims is a Parenthetical or Dialogue line; a blank line
/// or a block-ending forced line ends it, and Lyrics, outline and invisible lines interrupt it.
/// Outside a block the framing rules apply. The lines before `i` are resolved by then, so a
/// two-space line seen there is already blank or Dialogue; one seen after `i` is outside any block
/// a framed line could open, so it counts as blank, and it is never the text line a cue needs.
fn frame(roles: &mut [Role], lines: &[Logical]) {
    let mut in_block = false;
    for i in 0..roles.len() {
        let visible = lines[i].visible.trim_start_matches([' ', '\t']);
        roles[i] = match roles[i] {
            Role::Blank => {
                in_block = false;
                Role::Blank
            }
            Role::TwoSpaces if in_block => Role::Dialogue,
            Role::TwoSpaces => Role::Blank,
            Role::Shaped(s) if s == LineShape::Lyrics || is_outline(s) => roles[i],
            Role::Shaped(s) if classify::ends_block(s) => {
                in_block = s == LineShape::ForcedCue;
                roles[i]
            }
            Role::Shaped(_) if in_block => {
                let text = visible.trim_end_matches([' ', '\t']);
                if text.starts_with('(') && text.ends_with(')') {
                    Role::Parenthetical
                } else {
                    Role::Dialogue
                }
            }
            Role::Shaped(s) => {
                let r = framed(roles, i, s, visible);
                in_block = matches!(r, Role::Cue);
                r
            }
            r => r,
        };
    }
}

/// How the significant neighbours of a line frame it.
pub(crate) struct Framing {
    /// A blank line, or the start of the document, comes before it.
    pub(crate) blank_before: bool,
    /// A blank line before and a blank line after (the end of the document counts as blank).
    pub(crate) framed: bool,
    /// A text line follows: not blank, not a two-space line, not a forced line that ends a block.
    pub(crate) text_after: bool,
}

/// How line `i` is framed. [`frame`] calls it while the later lines are still unframed, so on final
/// roles it gives the same answer only for `blank_before` and for lines whose final role is
/// `Shaped(Plain)`.
pub(crate) fn framing(roles: &[Role], i: usize) -> Framing {
    let blank_before = matches!(prev_significant(roles, i), Role::Blank);
    let after = next_significant(roles, i);
    Framing {
        blank_before,
        framed: blank_before && matches!(after, Role::Blank | Role::TwoSpaces),
        // The line after a cue must be a text line: not blank, not a two-space line, not a forced
        // line that ends a block.
        text_after: matches!(after, Role::Shaped(s) if !classify::ends_block(s)),
    }
}

/// The framing rules for a line outside a block, in the order heading, transition, cue (SPEC 4.0,
/// step 3). A line no rule takes is Action. An unframed heading-by-prefix line is Action, never a
/// cue (`OFF-6a`), and the `TO:` carve-out keeps a line from being a Transition or a cue, not a
/// heading (`OFF-23a`, `OFF-23b`).
fn framed(roles: &[Role], i: usize, shape: LineShape, visible: &str) -> Role {
    let Framing {
        blank_before,
        framed,
        text_after,
    } = framing(roles, i);
    let trimmed = visible.trim_end_matches([' ', '\t']);
    if shape == LineShape::PrefixHeading {
        Role::Shaped(if framed { shape } else { LineShape::Plain })
    } else if classify::to_trailing_space(visible) {
        Role::Shaped(LineShape::Plain)
    } else if framed && classify::transition(visible) {
        Role::Transition
    } else if blank_before && text_after && classify::cue(trimmed, false).is_some() {
        Role::Cue
    } else {
        Role::Shaped(LineShape::Plain)
    }
}

/// Whether `role` continues the open `wrap` element.
fn is_member(wrap: NodeKind, role: Role) -> bool {
    match (wrap, role) {
        (NodeKind::Action, Role::Shaped(s)) => is_action_text(s),
        (NodeKind::Lyrics, Role::Shaped(s)) => s == LineShape::Lyrics,
        (NodeKind::DialogueBlock, Role::Shaped(s)) => s == LineShape::Lyrics,
        (NodeKind::DialogueBlock, Role::Dialogue | Role::Parenthetical) => true,
        (NodeKind::TitleEntry, Role::TitleValue) => true,
        _ => false,
    }
}

/// Whether `role` may sit inside the open `wrap` element between two members. It joins the element
/// only when a member follows; otherwise it is top-level, as are leading ones.
fn is_skippable(wrap: NodeKind, role: Role) -> bool {
    match (wrap, role) {
        (_, Role::Invisible) => true,
        (NodeKind::Action | NodeKind::DialogueBlock, Role::Shaped(s)) => is_outline(s),
        _ => false,
    }
}

/// The block pass: groups the lines in `range` into the items the tree is made of.
fn items(roles: &[Role], range: Range<usize>) -> Vec<Item> {
    let mut out = Vec::new();
    // The open element and its first line; `end` is one past its last member line, and the lines
    // from `end` to the current one are skippable lines waiting for a member.
    let mut open: Option<(NodeKind, usize)> = None;
    let mut end = 0;
    for i in range.clone() {
        let role = roles[i];
        if let Some((wrap, _)) = open {
            if is_member(wrap, role) {
                end = i + 1;
                continue;
            }
            if is_skippable(wrap, role) {
                continue;
            }
        }
        close(&mut out, roles, open.take(), end, i);
        let wrap = match role {
            Role::Shaped(LineShape::Lyrics) => Some(NodeKind::Lyrics),
            Role::Shaped(s) if is_action_text(s) => Some(NodeKind::Action),
            Role::Cue | Role::Shaped(LineShape::ForcedCue) => Some(NodeKind::DialogueBlock),
            Role::TitleKey => Some(NodeKind::TitleEntry),
            _ => None,
        };
        match wrap {
            Some(wrap) => {
                open = Some((wrap, i));
                end = i + 1;
            }
            None => out.push(Item::Line(i)),
        }
    }
    close(&mut out, roles, open, end, range.end);
    out
}

/// Ends the open element after its last member, `end`. The skippable lines up to `to` that no
/// member followed are top-level. Inside a Dialogue block, the lines after the cue are grouped
/// again, so a run of Lyrics is one node there too.
fn close(
    out: &mut Vec<Item>,
    roles: &[Role],
    open: Option<(NodeKind, usize)>,
    end: usize,
    to: usize,
) {
    let Some((wrap, start)) = open else { return };
    let kids = if wrap == NodeKind::DialogueBlock {
        let mut kids = vec![Item::Line(start)];
        kids.extend(items(roles, start + 1..end));
        kids
    } else {
        (start..end).map(Item::Line).collect()
    };
    out.push(Item::Wrap(wrap, kids));
    out.extend((end..to).map(Item::Line));
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

/// `[ws] TitleKey(Text) [ws] Colon [ws] TitleValue(Text)? [ws]`: a title-page key line.
fn key_parts(visible: &str) -> Vec<Part> {
    let lead = visible.len() - visible.trim_start_matches([' ', '\t']).len();
    let key = classify::key_line(&visible[lead..]).expect("the title pass read this key line");
    let (key, colon) = (lead + key.key.start..lead + key.key.end, lead + key.colon);
    let end = visible.trim_end_matches([' ', '\t']).len();
    let rest = &visible[colon + 1..end];
    let value = end - rest.trim_start_matches([' ', '\t']).len();
    let mut parts = vec![
        Part::Tok(TokenKind::Whitespace, 0..lead),
        Part::Node(
            NodeKind::TitleKey,
            vec![Part::Tok(TokenKind::Text, key.clone())],
        ),
        Part::Tok(TokenKind::Whitespace, key.end..colon),
        Part::Tok(TokenKind::Colon, colon..colon + 1),
        Part::Tok(TokenKind::Whitespace, colon + 1..value),
    ];
    if value < end {
        parts.push(Part::Node(
            NodeKind::TitleValue,
            vec![Part::Tok(TokenKind::Text, value..end)],
        ));
    }
    parts.push(Part::Tok(TokenKind::Whitespace, end..visible.len()));
    parts.retain(|p| !matches!(p, Part::Tok(_, r) if r.is_empty()));
    parts
}

/// `[ws] TitleValue(Text) [ws]`: the layout of a title-page continuation line.
fn value_parts(visible: &str) -> Vec<Part> {
    let mut parts = text_parts(visible, None);
    for part in &mut parts {
        if let Part::Tok(TokenKind::Text, _) = part {
            *part = Part::Node(NodeKind::TitleValue, vec![part.clone()]);
        }
    }
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

/// `[ws] Sigil"@"? CharacterName(Text) ([ws] Extension(Marker"(" Text? Marker")"))* [ws]
/// Caret(Marker"^")? [ws]`: the layout of a Character cue line.
fn cue_parts(visible: &str, forced: bool) -> Vec<Part> {
    let lead = visible.len() - visible.trim_start_matches([' ', '\t']).len();
    let at = lead + usize::from(forced);
    let end = visible.trim_end_matches([' ', '\t']).len();
    let cue = classify::cue(&visible[at..end], forced).expect("the framing pass read this cue");
    let shift = |r: &Range<usize>| at + r.start..at + r.end;
    let mut parts = vec![Part::Tok(TokenKind::Whitespace, 0..lead)];
    if forced {
        parts.push(Part::Tok(TokenKind::Sigil, lead..at));
    }
    let name = shift(&cue.name);
    let mut after = name.end;
    parts.push(Part::Node(
        NodeKind::CharacterName,
        vec![Part::Tok(TokenKind::Text, name)],
    ));
    for ext in cue.extensions.iter().map(shift) {
        let mut kids = vec![
            Part::Tok(TokenKind::Marker, ext.start..ext.start + 1),
            Part::Tok(TokenKind::Text, ext.start + 1..ext.end - 1),
            Part::Tok(TokenKind::Marker, ext.end - 1..ext.end),
        ];
        kids.retain(|p| !matches!(p, Part::Tok(_, r) if r.is_empty()));
        parts.push(Part::Tok(TokenKind::Whitespace, after..ext.start));
        parts.push(Part::Node(NodeKind::Extension, kids));
        after = ext.end;
    }
    if let Some(c) = cue.caret.map(|c| at + c) {
        parts.push(Part::Tok(TokenKind::Whitespace, after..c));
        parts.push(Part::Node(
            NodeKind::Caret,
            vec![Part::Tok(TokenKind::Marker, c..c + 1)],
        ));
        after = c + 1;
    }
    parts.push(Part::Tok(TokenKind::Whitespace, after..visible.len()));
    parts.retain(|p| !matches!(p, Part::Tok(_, r) if r.is_empty()));
    parts
}

/// The layout of one logical line.
fn parts(visible: &str, role: Role) -> Vec<Part> {
    let sigil = |n| Some((TokenKind::Sigil, n));
    let node = |kind, parts| vec![Part::Node(kind, parts)];
    let shape = match role {
        Role::Shaped(shape) => shape,
        Role::Cue => return node(NodeKind::CharacterCue, cue_parts(visible, false)),
        Role::Transition => return node(NodeKind::Transition, text_parts(visible, None)),
        Role::Dialogue => return node(NodeKind::Dialogue, text_parts(visible, None)),
        Role::Parenthetical => return node(NodeKind::Parenthetical, text_parts(visible, None)),
        Role::TitleKey => return key_parts(visible),
        Role::TitleValue => return value_parts(visible),
        Role::Blank | Role::TwoSpaces | Role::Invisible | Role::TitleStray => {
            return text_parts(visible, None);
        }
    };
    match shape {
        LineShape::Plain => text_parts(visible, None),
        LineShape::ForcedCue => node(NodeKind::CharacterCue, cue_parts(visible, true)),
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

/// What [`emit`] reads from.
struct Lines<'a> {
    src: &'a str,
    lines: &'a [Logical],
    roles: &'a [Role],
    hidden: &'a [Hidden],
}

/// Emits `item`. The line break after each of its lines but the last belongs to it; the last one
/// belongs to whatever holds the item.
fn emit(b: &mut TreeBuilder, at: &Lines<'_>, item: &Item) {
    match item {
        Item::Line(i) => {
            let line = &at.lines[*i];
            let parts = parts(&line.visible, at.roles[*i]);
            // A line that is one element node holds its edge hidden spans as children.
            if let [Part::Node(kind, kids)] = parts.as_slice() {
                b.start_node(*kind);
                layout::emit_line(b, at.src, line, kids, at.hidden);
                b.finish_node();
            } else {
                layout::emit_line(b, at.src, line, &parts, at.hidden);
            }
        }
        Item::Wrap(kind, kids) => {
            b.start_node(*kind);
            for (k, kid) in kids.iter().enumerate() {
                emit(b, at, kid);
                if k + 1 < kids.len() {
                    line_break(b, &at.lines[kid.last_line()]);
                }
            }
            b.finish_node();
        }
    }
}

fn line_break(b: &mut TreeBuilder, line: &Logical) {
    if let Some(brk) = line.brk {
        b.token(TokenKind::LineBreak, brk);
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
    let (hidden, unclosed) = hidden::scan(src, from, &physical);
    let lines = hidden::logical(src, &physical, &hidden);
    let mut roles: Vec<Role> = lines.iter().map(role).collect();
    let page = title_page(&mut roles, &lines);
    frame(&mut roles, &lines);
    let at = Lines {
        src,
        lines: &lines,
        roles: &roles,
        hidden: &hidden,
    };
    let all = 0..roles.len();
    let top = match page {
        Some(page) => {
            let mut top = items(&roles, 0..page.start);
            top.push(Item::Wrap(NodeKind::TitlePage, items(&roles, page.clone())));
            top.extend(items(&roles, page.end..all.end));
            top
        }
        None => items(&roles, all),
    };
    for item in top {
        emit(&mut b, &at, &item);
        // The break after an element's last line belongs to the Gap, not to the element.
        line_break(&mut b, &lines[item.last_line()]);
    }
    let diagnostics = diagnose::run(&diagnose::Input {
        src,
        from,
        physical: &physical,
        lines: &lines,
        roles: &roles,
        hidden: &hidden,
        unclosed: &unclosed,
    });
    b.finish(src, diagnostics)
}
