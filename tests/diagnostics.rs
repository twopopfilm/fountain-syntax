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
    for (index, (code, name, severity)) in rows.into_iter().enumerate() {
        assert_eq!(code as usize, index, "{name}: variant order is table order");
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

#[test]
fn diag_unframed_heading() {
    fires("Walk.\nINT. HOUSE\n\nb", C::UnframedHeading, "INT. HOUSE");
    fires("INT. HOUSE\nShe waits.", C::UnframedHeading, "INT. HOUSE");
    quiet("INT. HOUSE\n\nShe waits.", C::UnframedHeading);
    quiet("INTERIOR\nx", C::UnframedHeading);
    quiet(".INT. HOUSE\nx", C::UnframedHeading);
}

#[test]
fn diag_empty_heading() {
    fires("INT.\n\nx", C::EmptyHeading, "INT.");
    fires("int \n\nx", C::EmptyHeading, "int ");
    fires("INT.[[a\nb]]\n\nx", C::EmptyHeading, "INT.[[a\nb]]");
    quiet("INT. HOUSE\n\nx", C::EmptyHeading);
    quiet(".INT.\n\nx", C::EmptyHeading);
    quiet("INT. #1#\n\nx", C::EmptyHeading);
}

#[test]
fn diag_period_not_forcing() {
    fires(". SPACE AFTER", C::PeriodNotForcing, ". SPACE AFTER");
    fires(".>HEADING", C::PeriodNotForcing, ".>HEADING");
    fires("\u{FEFF}. x", C::PeriodNotForcing, ". x");
    quiet(".HOUSE", C::PeriodNotForcing);
    quiet("..DOUBLE", C::PeriodNotForcing);
    quiet("...trailing off", C::PeriodNotForcing);
    quiet(".", C::PeriodNotForcing);
}

#[test]
fn diag_scene_number_rejected() {
    fires("INT. A #1 A#\n\nx", C::SceneNumberRejected, "INT. A #1 A#");
    fires("INT. A #1_A#\n\nx", C::SceneNumberRejected, "INT. A #1_A#");
    fires(
        "INT. A #7# trailing words\n\nx",
        C::SceneNumberRejected,
        "INT. A #7# trailing words",
    );
    fires("INT. A ##\n\nx", C::SceneNumberRejected, "INT. A ##");
    fires(".TITLE #1 A#", C::SceneNumberRejected, ".TITLE #1 A#");
    quiet("INT. A #12A#\n\nx", C::SceneNumberRejected);
    quiet("INT. A\n\nx", C::SceneNumberRejected);
    quiet("She waits #1 A#.", C::SceneNumberRejected);
    // The locked form is a scene number with the feature and quiet without it.
    quiet("INT. A ##12A#\n\nx", C::SceneNumberRejected);
}

#[test]
fn diag_transition_lookalike() {
    fires("a\n\nFADE OUT.\n\nb", C::TransitionLookalike, "FADE OUT.");
    fires("FADE IN:\n\nb", C::TransitionLookalike, "FADE IN:");
    fires(
        "a\n\nFADE TO BLACK.\n\nb",
        C::TransitionLookalike,
        "FADE TO BLACK.",
    );
    fires(
        "a\n\nCUT TO BLACK.",
        C::TransitionLookalike,
        "CUT TO BLACK.",
    );
    quiet("FADE IN: on her face\n\nx", C::TransitionLookalike);
    quiet("Fade in:\n\nx", C::TransitionLookalike);
    quiet("a\n\nFADE OUT.\nb", C::TransitionLookalike);
    quiet("a\n\nCUT TO:\n\nb", C::TransitionLookalike);
}

#[test]
fn diag_unframed_transition() {
    fires("a\nCUT TO:\n\nb", C::UnframedTransition, "CUT TO:");
    fires("a\n\nCUT TO:\nINT. HOUSE", C::UnframedTransition, "CUT TO:");
    quiet("a\n\nCUT TO:\n\nb", C::UnframedTransition);
    quiet("a\nCut to:\nb", C::UnframedTransition);
    quiet("@CUT TO:\nhi", C::UnframedTransition);
}

#[test]
fn diag_to_trailing_space() {
    fires("a\n\nCUT TO: \n\nb", C::ToTrailingSpace, "CUT TO: ");
    fires("a\n\nCUT TO: \nb", C::ToTrailingSpace, "CUT TO: ");
    quiet("a\nCUT TO: \nb", C::ToTrailingSpace);
    quiet("a\n\nCUT TO:\n\nb", C::ToTrailingSpace);
    // A forced line or a title-page key is not the look-alike.
    quiet("a\n\n!CUT TO: \n\nb", C::ToTrailingSpace);
    quiet("CUT TO: \n   x", C::ToTrailingSpace);
}

#[test]
fn diag_cue_needs_force() {
    fires(
        "\u{674E}\u{660E}\nHello.",
        C::CueNeedsForce,
        "\u{674E}\u{660E}",
    );
    fires("a\n\n23\nHello.", C::CueNeedsForce, "23");
    quiet("MARA\nHello.", C::CueNeedsForce);
    quiet("\u{674E}\u{660E}", C::CueNeedsForce);
    quiet("...\nHello.", C::CueNeedsForce);
    quiet("\u{674E}\u{660E} Bob\nHi", C::CueNeedsForce);
    quiet("@\u{674E}\u{660E}\nHello.", C::CueNeedsForce);
}

#[test]
fn diag_sigil_in_dialogue() {
    fires("MARA\nhi\n!Action now", C::SigilInDialogue, "!Action now");
    fires("MARA\nhi\n@OTTO\nyo", C::SigilInDialogue, "@OTTO");
    fires("MARA\nhi\n.INT. X", C::SigilInDialogue, ".INT. X");
    quiet("MARA\nhi\n\n!Action", C::SigilInDialogue);
    quiet("MARA\nhi\n~la la", C::SigilInDialogue);
}

#[test]
fn diag_caret_not_last() {
    fires("NAME ^ (V.O.)\nhi", C::CaretNotLast, "NAME ^ (V.O.)");
    fires("@NAME ^ x\nhi", C::CaretNotLast, "@NAME ^ x");
    quiet("MARA (V.O.) ^\nhi", C::CaretNotLast);
    quiet("OTTO^\nhi", C::CaretNotLast);
}

#[test]
fn diag_partial_parenthetical() {
    fires(
        "MARA\n(whispering) and then speaks",
        C::PartialParenthetical,
        "(whispering) and then speaks",
    );
    fires("MARA\n(not closed", C::PartialParenthetical, "(not closed");
    fires(
        "MARA\nhi\nparagraph)",
        C::PartialParenthetical,
        "paragraph)",
    );
    quiet("MARA\n(softly)\nhi", C::PartialParenthetical);
    quiet("(not closed\nx", C::PartialParenthetical);
}

#[test]
fn diag_blank_lookalike() {
    fires("a\n \nb", C::BlankLookalike, " ");
    fires("a\n  \nb", C::BlankLookalike, "  ");
    fires("a\r\n \r\nb", C::BlankLookalike, " ");
    quiet("a\n\nb", C::BlankLookalike);
    quiet("MARA\nhi\n  \nyo", C::BlankLookalike);
    quiet("a\n \n\nb", C::BlankLookalike);
    quiet(" \na", C::BlankLookalike);
}

#[test]
fn diag_adjacent_outline_line() {
    fires("# Act\nShe waits.", C::AdjacentOutlineLine, "She waits.");
    fires("===\nx", C::AdjacentOutlineLine, "x");
    fires("= syn\nx", C::AdjacentOutlineLine, "x");
    fires("MARA\n# x\nhi", C::AdjacentOutlineLine, "hi");
    quiet("# Act\n\nShe waits.", C::AdjacentOutlineLine);
    quiet("# Act\n## Two", C::AdjacentOutlineLine);
    quiet("# Act\n[[n]]", C::AdjacentOutlineLine);
}

#[test]
fn diag_ambiguous_equals() {
    fires("==", C::AmbiguousEquals, "==");
    fires("=== text", C::AmbiguousEquals, "=== text");
    quiet("===", C::AmbiguousEquals);
    quiet("= syn", C::AmbiguousEquals);
    quiet("  ===  ", C::AmbiguousEquals);
}

#[test]
fn diag_lone_caret() {
    fires("OTTO ^\nyo", C::LoneCaret, "OTTO ^");
    fires(
        "MARA\nhi\n\nOTTO ^\nyo\n\nNELL ^\nhey",
        C::LoneCaret,
        "NELL ^",
    );
    quiet("MARA\nhi\n\nOTTO ^\nyo", C::LoneCaret);
    quiet("MARA\nhi\n\n# Act\n\nOTTO ^\nyo", C::LoneCaret);
    // An unpaired block can be the left half of the next pair (the #40 ruling).
    quiet(
        "MARA\nhi\n\nOTTO ^\nyo\n\nNELL ^\nhey\n\nIVAN ^\nyes",
        C::LoneCaret,
    );
}

/// The `(start, end)` of each diagnostic of `code` in `src`.
fn offsets(src: &str, code: C) -> Vec<(usize, usize)> {
    parse_checked(src)
        .diagnostics()
        .iter()
        .filter(|d| d.code == code)
        .map(|d| (d.span.start, d.span.end))
        .collect()
}

#[test]
fn diag_nested_opener() {
    fires("[[outer [[inner]] outer]]", C::NestedOpener, "[[");
    assert_eq!(
        offsets("[[outer [[inner]] outer]]", C::NestedOpener),
        [(8, 10)]
    );
    fires("/* a /* b */", C::NestedOpener, "/*");
    quiet("[[a]] [[b]]", C::NestedOpener);
    quiet("/* a */ /* b */", C::NestedOpener);
    // A Note's inner text is searched for `[[` only, and a Boneyard's for `/*` only.
    quiet("[[a /* b]]", C::NestedOpener);
    quiet("/* a [[ b */", C::NestedOpener);
}

#[test]
fn diag_unclosed_note() {
    fires("a [[b\n\nc", C::UnclosedNote, "[[");
    quiet("a [[b\nc]]", C::UnclosedNote);
    quiet("a [[b\n  \nc]]", C::UnclosedNote);
}

#[test]
fn diag_unclosed_boneyard() {
    fires("a /* b", C::UnclosedBoneyard, "/*");
    assert_eq!(offsets("/* a /* b", C::UnclosedBoneyard), [(0, 2), (5, 7)]);
    quiet("a /* b\n\nc */", C::UnclosedBoneyard);
}

#[test]
fn diag_stray_closer() {
    fires("a ]] b", C::StrayCloser, "]]");
    fires("text before */", C::StrayCloser, "*/");
    assert_eq!(
        offsets("[[outer [[inner]] outer]] world.", C::StrayCloser),
        [(23, 25)]
    );
    quiet("[[a]]", C::StrayCloser);
    quiet("/* a */", C::StrayCloser);
    quiet("\u{FEFF}[[a]]", C::StrayCloser);
    // Non-overlapping: `]]]` holds one pair.
    assert_eq!(offsets("x ]]]", C::StrayCloser), [(2, 4)]);
}
