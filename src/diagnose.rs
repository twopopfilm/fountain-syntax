//! The diagnostics pass (SPEC 7). It reads what the parse already worked out and never changes
//! it. Each group of codes is one private pass that is linear in the input; [`run`] collects them
//! and sorts the result.

use crate::classify::{self, LineShape};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::hidden::{Hidden, Logical};
use crate::lines::Line;
use crate::parse::{self, Role};
use crate::tree::{NodeKind, Span};
use crate::view::inline;

/// What the parse knows about the document, shared by every pass.
pub(crate) struct Input<'a> {
    pub(crate) src: &'a str,
    /// Where the text starts: after the BOM, if there is one.
    pub(crate) from: usize,
    pub(crate) physical: &'a [Line],
    pub(crate) lines: &'a [Logical],
    pub(crate) roles: &'a [Role],
    pub(crate) hidden: &'a [Hidden],
    /// The `[[` and `/*` that opened no span (the second result of `hidden::scan`).
    pub(crate) unclosed: &'a [Hidden],
}

/// Every diagnostic of the document, ordered by span start (then end, then SPEC 7 table order).
pub(crate) fn run(input: &Input<'_>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    line_endings(input, &mut out);
    title_page(input, &mut out);
    look_alikes(input, &mut out);
    blocks_and_layout(input, &mut out);
    hidden_marks(input, &mut out);
    emphasis(input, &mut out);
    out.sort_by_key(|d| (d.span.start, d.span.end, d.code as u8));
    out
}

/// `mixed-line-endings`: the first line break that differs from the first one.
fn line_endings(input: &Input<'_>, out: &mut Vec<Diagnostic>) {
    let text = |brk: crate::tree::Span| &input.src[brk.start..brk.end];
    let mut breaks = input.physical.iter().filter_map(|l| l.brk);
    let Some(first) = breaks.next() else { return };
    if let Some(other) = breaks.find(|&b| text(b) != text(first)) {
        out.push(Diagnostic::new(DiagnosticCode::MixedLineEndings, other));
    }
}

/// `title-page-stray-line`: every title-page line that is neither a key nor a continuation.
fn title_page(input: &Input<'_>, out: &mut Vec<Diagnostic>) {
    for (line, role) in input.lines.iter().zip(input.roles) {
        if matches!(role, Role::TitleStray) {
            out.push(Diagnostic::new(
                DiagnosticCode::TitlePageStrayLine,
                line.span,
            ));
        }
    }
}

/// The heading, transition and cue codes: lines that read as one element and are another. Each
/// trigger is tested on the cheap text or role first, so the neighbour lookup runs only for a
/// candidate.
fn look_alikes(input: &Input<'_>, out: &mut Vec<Diagnostic>) {
    use DiagnosticCode as C;
    let framing = |i| parse::framing(input.roles, i);
    for (i, (line, &role)) in input.lines.iter().zip(input.roles).enumerate() {
        let t = line.visible.trim_start_matches([' ', '\t']);
        let shaped = |s| matches!(role, Role::Shaped(x) if x == s);
        let plain = shaped(LineShape::Plain);
        let mut codes = Vec::new();
        if (plain || matches!(role, Role::Dialogue))
            && classify::shape(t) == LineShape::PrefixHeading
        {
            codes.push(C::UnframedHeading);
        }
        if shaped(LineShape::PrefixHeading)
            && classify::heading_prefix_len(t)
                .is_some_and(|n| t[n..].trim_start_matches([' ', '\t']).is_empty())
        {
            codes.push(C::EmptyHeading);
        }
        if t.strip_prefix('.')
            .and_then(|rest| rest.chars().next())
            .is_some_and(|c| !classify::is_alnum(c) && c != '.')
        {
            codes.push(C::PeriodNotForcing);
        }
        if (shaped(LineShape::PrefixHeading) || shaped(LineShape::ForcedHeading))
            && classify::rejected_scene_number(t)
        {
            codes.push(C::SceneNumberRejected);
        }
        if plain
            && matches!(
                t.trim_end_matches([' ', '\t']),
                "FADE OUT." | "FADE TO BLACK." | "CUT TO BLACK."
            )
            && framing(i).framed
        {
            codes.push(C::TransitionLookalike);
        }
        if (plain || matches!(role, Role::Cue | Role::Dialogue)) && classify::transition(t) {
            codes.push(C::UnframedTransition);
        }
        if plain && classify::to_trailing_space(t) && framing(i).blank_before {
            codes.push(C::ToTrailingSpace);
        }
        if plain && needs_force(t) {
            let f = framing(i);
            if f.blank_before && f.text_after {
                codes.push(C::CueNeedsForce);
            }
        }
        out.extend(codes.into_iter().map(|c| Diagnostic::new(c, line.span)));
    }
}

/// The dialogue-block and layout codes, in one forward pass that tracks whether a Dialogue block
/// is open, as `frame` (parse.rs) did. `left` and `lone` mirror `pair` in the view: `left` is the
/// cue line of the block that can still take a partner, and `lone` the caret blocks with none.
fn blocks_and_layout(input: &Input<'_>, out: &mut Vec<Diagnostic>) {
    use DiagnosticCode as C;
    let roles = input.roles;
    let outline = |r: Role| matches!(r, Role::Shaped(s) if parse::is_outline(s));
    let mut in_block = false;
    let mut left: Option<usize> = None;
    let mut lone: Vec<usize> = Vec::new();
    for (i, (line, &role)) in input.lines.iter().zip(roles).enumerate() {
        let t = line.visible.trim_start_matches([' ', '\t']);
        let mut push = |code| out.push(Diagnostic::new(code, line.span));
        if t.starts_with("==") && classify::shape(t) != LineShape::PageBreak {
            push(C::AmbiguousEquals);
        }
        if i > 0
            && outline(roles[i - 1])
            && !line.visible.trim_matches([' ', '\t']).is_empty()
            && !outline(role)
        {
            push(C::AdjacentOutlineLine);
        }
        match role {
            Role::Blank => {
                let blank = |j| matches!(roles[j], Role::Blank);
                if !line.visible.is_empty()
                    && 0 < i
                    && i + 1 < roles.len()
                    && !blank(i - 1)
                    && !blank(i + 1)
                {
                    push(C::BlankLookalike);
                }
                in_block = false;
            }
            Role::Cue | Role::Shaped(LineShape::ForcedCue) => {
                let forced = matches!(role, Role::Shaped(_));
                let text = t.trim_end_matches([' ', '\t']);
                let text = if forced { &text[1..] } else { text };
                if text.strip_suffix('^').unwrap_or(text).contains('^') {
                    push(C::CaretNotLast);
                }
                if in_block && forced {
                    push(C::SigilInDialogue);
                }
                if classify::cue(text, forced).is_some_and(|c| c.caret.is_some()) {
                    if let Some(j) = left.take() {
                        if lone.last() == Some(&j) {
                            lone.pop();
                        }
                    } else {
                        lone.push(i);
                        left = Some(i);
                    }
                } else {
                    left = Some(i);
                }
                in_block = true;
            }
            Role::Dialogue => {
                let text = t.trim_end_matches([' ', '\t']);
                // A trailing aside (`Call me (later)`) is ordinary dialogue: the reverse half fires
                // only when the line has no `(` at all.
                if (text.starts_with('(') && !text.ends_with(')'))
                    || (text.ends_with(')') && !text.contains('('))
                {
                    push(C::PartialParenthetical);
                }
            }
            Role::Shaped(s) if classify::ends_block(s) => {
                if in_block {
                    push(C::SigilInDialogue);
                }
                in_block = false;
                left = None;
            }
            Role::Shaped(LineShape::Lyrics) => left = left.filter(|_| in_block),
            Role::Shaped(LineShape::Plain | LineShape::PrefixHeading) | Role::Transition => {
                in_block = false;
                left = None;
            }
            _ => {}
        }
    }
    out.extend(
        lone.into_iter()
            .map(|i| Diagnostic::new(C::LoneCaret, input.lines[i].span)),
    );
}

/// The Note and Boneyard codes: unclosed openers, an opener inside a span of its own kind, and a
/// closer outside every span. Each is a single left-to-right walk over the text it covers.
fn hidden_marks(input: &Input<'_>, out: &mut Vec<Diagnostic>) {
    use DiagnosticCode as C;
    let src = input.src;
    for u in input.unclosed {
        let code = match u.kind {
            NodeKind::Note => C::UnclosedNote,
            _ => C::UnclosedBoneyard,
        };
        out.push(Diagnostic::new(code, u.span));
    }
    for h in input.hidden {
        let opener = if h.kind == NodeKind::Note { "[[" } else { "/*" };
        let inner = Span::new(h.span.start + 2, h.span.end - 2);
        marks(src, inner, opener, |s| {
            out.push(Diagnostic::new(C::NestedOpener, s));
        });
    }
    // The gaps between hidden spans are the text outside every span.
    let mut at = input.from;
    let ends = input.hidden.iter().map(|h| (h.span.start, h.span.end));
    for (start, end) in ends.chain([(src.len(), src.len())]) {
        for closer in ["]]", "*/"] {
            marks(src, Span::new(at, start), closer, |s| {
                out.push(Diagnostic::new(C::StrayCloser, s));
            });
        }
        at = end;
    }
}

/// Calls `found` with the span of each non-overlapping `mark` in `src[within]`, left to right.
fn marks(src: &str, within: Span, mark: &str, mut found: impl FnMut(Span)) {
    let mut at = within.start;
    while let Some(j) = src[at..within.end].find(mark) {
        let start = at + j;
        at = start + mark.len();
        found(Span::new(start, at));
    }
}

/// `emphasis-line-break`: an emphasis opener with no closer on its line, and a matching closer on
/// a later line of the same paragraph. The walk goes backwards through the lines, keeping the
/// closer kinds seen after the current line (at most four: `*`, `**`, `***`, `_`).
fn emphasis(input: &Input<'_>, out: &mut Vec<Diagnostic>) {
    let mut closers: Vec<(char, usize)> = Vec::new();
    let mut group = None;
    for (line, &role) in input.lines.iter().zip(input.roles).rev() {
        let (this, forced) = match role {
            Role::Shaped(LineShape::Plain) => (Some(0), false),
            Role::Shaped(LineShape::ForcedAction) => (Some(0), true),
            Role::Dialogue => (Some(1), false),
            _ => (None, false),
        };
        if this != group {
            closers.clear();
            group = this;
        }
        if this.is_none() {
            continue;
        }
        let t = line.visible.trim_start_matches([' ', '\t']);
        // The view drops the sigil (OFF-22d); left in, a `*` right after it could close.
        let t = if forced {
            t.strip_prefix('!').unwrap_or(t)
        } else {
            t
        };
        // shortcut: hidden spans are dropped from the visible text, not treated as solid atoms as
        // the view does; upgrade if a diagnostic disagrees with the view next to an inline Note
        let (open, closing) = inline::unmatched_marks(t);
        if open.iter().any(|m| closers.contains(m)) {
            out.push(Diagnostic::new(
                DiagnosticCode::EmphasisLineBreak,
                line.span,
            ));
        }
        for m in closing {
            if !closers.contains(&m) {
                closers.push(m);
            }
        }
    }
}

/// No letter with case, but a digit or a caseless letter (SPEC 7, `cue-needs-force`).
fn needs_force(t: &str) -> bool {
    let t = t.trim_end_matches([' ', '\t']);
    !t.chars().any(|c| c.is_uppercase() || c.is_lowercase())
        && t.chars().any(|c| c.is_numeric() || c.is_alphabetic())
}
