//! The diagnostics pass (SPEC 7). It reads what the parse already worked out and never changes
//! it. Each group of codes is one private pass that is linear in the input; [`run`] collects them
//! and sorts the result.

use crate::classify::{self, LineShape};
use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::hidden::Logical;
use crate::lines::Line;
use crate::parse::{self, Role};

/// What the parse knows about the document, shared by every pass.
pub(crate) struct Input<'a> {
    pub(crate) src: &'a str,
    pub(crate) physical: &'a [Line],
    pub(crate) lines: &'a [Logical],
    pub(crate) roles: &'a [Role],
}

/// Every diagnostic of the document, ordered by span start (then end, then SPEC 7 table order).
pub(crate) fn run(input: &Input<'_>) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    line_endings(input, &mut out);
    title_page(input, &mut out);
    look_alikes(input, &mut out);
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
                "FADE IN:" | "FADE OUT." | "FADE TO BLACK." | "CUT TO BLACK."
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

/// No letter with case, but a digit or a caseless letter (SPEC 7, `cue-needs-force`).
fn needs_force(t: &str) -> bool {
    let t = t.trim_end_matches([' ', '\t']);
    !t.chars().any(|c| c.is_uppercase() || c.is_lowercase())
        && t.chars().any(|c| c.is_numeric() || c.is_alphabetic())
}
