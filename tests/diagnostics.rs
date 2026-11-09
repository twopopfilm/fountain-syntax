mod common;

use common::parse_checked;
use fountain_syntax::{DiagnosticCode as C, Severity};

/// Each diagnostic as its code and the source text of its span.
fn diags(src: &str) -> Vec<(C, String)> {
    parse_checked(src)
        .diagnostics()
        .iter()
        .map(|d| (d.code, src[d.span.start..d.span.end].to_owned()))
        .collect()
}

/// Asserts that `(code, text)` is among the diagnostics of `src`.
fn fires(src: &str, code: C, text: &str) {
    let all = diags(src);
    assert!(
        all.iter().any(|(c, t)| *c == code && t == text),
        "{src:?}: expected {code:?} on {text:?}, got {all:?}"
    );
}

/// Asserts that no diagnostic of `code` is among those of `src`.
fn quiet(src: &str, code: C) {
    let all = diags(src);
    assert!(
        all.iter().all(|(c, _)| *c != code),
        "{src:?}: unexpected {code:?} in {all:?}"
    );
}

#[test]
fn diag_codes_match_section_7() {
    let rows = [
        (C::BlankLookalike, "blank-lookalike", Severity::Info),
        (C::MixedLineEndings, "mixed-line-endings", Severity::Info),
        (C::CueNeedsForce, "cue-needs-force", Severity::Info),
        (C::UnframedHeading, "unframed-heading", Severity::Info),
        (C::EmptyHeading, "empty-heading", Severity::Info),
        (C::PeriodNotForcing, "period-not-forcing", Severity::Info),
        (
            C::SceneNumberRejected,
            "scene-number-rejected",
            Severity::Info,
        ),
        (
            C::TransitionLookalike,
            "transition-lookalike",
            Severity::Info,
        ),
        (
            C::UnframedTransition,
            "unframed-transition",
            Severity::Warning,
        ),
        (C::ToTrailingSpace, "to-trailing-space", Severity::Warning),
        (C::LoneCaret, "lone-caret", Severity::Warning),
        (C::CaretNotLast, "caret-not-last", Severity::Info),
        (
            C::PartialParenthetical,
            "partial-parenthetical",
            Severity::Info,
        ),
        (C::EmphasisLineBreak, "emphasis-line-break", Severity::Info),
        (C::NestedOpener, "nested-opener", Severity::Info),
        (C::UnclosedNote, "unclosed-note", Severity::Warning),
        (C::UnclosedBoneyard, "unclosed-boneyard", Severity::Warning),
        (C::StrayCloser, "stray-closer", Severity::Info),
        (
            C::TitlePageStrayLine,
            "title-page-stray-line",
            Severity::Warning,
        ),
        (
            C::AdjacentOutlineLine,
            "adjacent-outline-line",
            Severity::Info,
        ),
        (C::AmbiguousEquals, "ambiguous-equals", Severity::Info),
        (C::SigilInDialogue, "sigil-in-dialogue", Severity::Info),
    ];
    assert_eq!(rows.len(), 22);
    for (code, name, severity) in rows {
        assert_eq!(code.as_str(), name);
        assert_eq!(code.severity(), severity, "{name}");
    }
}

#[test]
fn diag_mixed_line_endings() {
    fires("a\r\nb\nc", C::MixedLineEndings, "\n");
    assert_eq!(diags("a\r\nb\nc\rd").len(), 1);
    quiet("a\r\nb\r\nc", C::MixedLineEndings);
    quiet("a\rb\r", C::MixedLineEndings);
    quiet("abc", C::MixedLineEndings);
}

#[test]
fn diag_title_page_stray_line() {
    fires(
        "Title: X\n  stray\n\nINT. A",
        C::TitlePageStrayLine,
        "  stray",
    );
    quiet("Title: X\n   more\n\nINT. A", C::TitlePageStrayLine);
}

#[test]
fn diag_order_is_by_span_start() {
    let got = diags("Title: X\n  s\n\na\r\nb\nc");
    assert_eq!(
        got,
        [
            (C::TitlePageStrayLine, "  s".to_owned()),
            // The first break is `\n`, so the first differing one is the `\r\n`.
            (C::MixedLineEndings, "\r\n".to_owned()),
        ]
    );
}
