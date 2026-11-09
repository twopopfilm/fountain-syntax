//! The diagnostics pass (SPEC 7). It reads what the parse already worked out and never changes
//! it. Each group of codes is one private pass that is linear in the input; [`run`] collects them
//! and sorts the result.

use crate::diagnostic::{Diagnostic, DiagnosticCode};
use crate::hidden::Logical;
use crate::lines::Line;
use crate::parse::Role;

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
