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

/// DIAG-mixed-line-endings
#[test]
fn diag_mixed_line_endings() {
    fires("a\r\nb\nc", C::MixedLineEndings, "\n");
    assert_eq!(diags("a\r\nb\nc\rd").len(), 1);
    quiet("a\r\nb\r\nc", C::MixedLineEndings);
    quiet("a\rb\r", C::MixedLineEndings);
    quiet("abc", C::MixedLineEndings);
}

/// DIAG-title-page-stray-line
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

/// DIAG-unframed-heading
#[test]
fn diag_unframed_heading() {
    fires("Walk.\nINT. HOUSE\n\nb", C::UnframedHeading, "INT. HOUSE");
    fires("INT. HOUSE\nShe waits.", C::UnframedHeading, "INT. HOUSE");
    fires("MARA\nExt. 4 please.", C::UnframedHeading, "Ext. 4 please.");
    quiet("INT. HOUSE\n\nShe waits.", C::UnframedHeading);
    quiet("INTERIOR\nx", C::UnframedHeading);
    quiet(".INT. HOUSE\nx", C::UnframedHeading);
}

/// DIAG-empty-heading
#[test]
fn diag_empty_heading() {
    fires("INT.\n\nx", C::EmptyHeading, "INT.");
    fires("int \n\nx", C::EmptyHeading, "int ");
    fires("INT.[[a\nb]]\n\nx", C::EmptyHeading, "INT.[[a\nb]]");
    quiet("INT. HOUSE\n\nx", C::EmptyHeading);
    quiet(".INT.\n\nx", C::EmptyHeading);
    quiet("INT. #1#\n\nx", C::EmptyHeading);
}

/// DIAG-period-not-forcing
#[test]
fn diag_period_not_forcing() {
    fires(". SPACE AFTER", C::PeriodNotForcing, ". SPACE AFTER");
    fires(".>HEADING", C::PeriodNotForcing, ".>HEADING");
    fires("\u{FEFF}. x", C::PeriodNotForcing, ". x");
    quiet(".HOUSE", C::PeriodNotForcing);
    quiet("..DOUBLE", C::PeriodNotForcing);
    quiet("...trailing off", C::PeriodNotForcing);
    quiet(".", C::PeriodNotForcing);
    // Inside the title page a leading period is a value or a stray line, not a forced heading.
    quiet(
        "Title: X\n    . a dotted value\n\nINT. A\n\nx",
        C::PeriodNotForcing,
    );
    let stray = "Title: X\n. stray\n\nINT. A\n\nx";
    fires(stray, C::TitlePageStrayLine, ". stray");
    quiet(stray, C::PeriodNotForcing);
    fires(
        "Title: X\n\n. SPACE AFTER",
        C::PeriodNotForcing,
        ". SPACE AFTER",
    );
}

/// DIAG-scene-number-rejected
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

/// DIAG-transition-lookalike
#[test]
fn diag_transition_lookalike() {
    fires("a\n\nFADE OUT.\n\nb", C::TransitionLookalike, "FADE OUT.");
    quiet("FADE IN:\n\nb", C::TransitionLookalike);
    quiet("a\n\nFADE IN:\n\nb", C::TransitionLookalike);
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

/// DIAG-unframed-transition
#[test]
fn diag_unframed_transition() {
    fires("a\nCUT TO:\n\nb", C::UnframedTransition, "CUT TO:");
    fires("a\n\nCUT TO:\nINT. HOUSE", C::UnframedTransition, "CUT TO:");
    fires("MARA\nWELCOME TO:", C::UnframedTransition, "WELCOME TO:");
    quiet("a\n\nCUT TO:\n\nb", C::UnframedTransition);
    quiet("a\nCut to:\nb", C::UnframedTransition);
    quiet("@CUT TO:\nhi", C::UnframedTransition);
}

/// DIAG-to-trailing-space
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

/// DIAG-cue-needs-force
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

/// DIAG-sigil-in-dialogue
#[test]
fn diag_sigil_in_dialogue() {
    fires("MARA\nhi\n!Action now", C::SigilInDialogue, "!Action now");
    fires("MARA\nhi\n@OTTO\nyo", C::SigilInDialogue, "@OTTO");
    fires("MARA\nhi\n.INT. X", C::SigilInDialogue, ".INT. X");
    quiet("MARA\nhi\n\n!Action", C::SigilInDialogue);
    quiet("MARA\nhi\n~la la", C::SigilInDialogue);
}

/// DIAG-caret-not-last
#[test]
fn diag_caret_not_last() {
    fires("NAME ^ (V.O.)\nhi", C::CaretNotLast, "NAME ^ (V.O.)");
    fires("@NAME ^ x\nhi", C::CaretNotLast, "@NAME ^ x");
    quiet("MARA (V.O.) ^\nhi", C::CaretNotLast);
    quiet("OTTO^\nhi", C::CaretNotLast);
}

/// DIAG-partial-parenthetical
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
    // The reverse half needs a line that ends with `)` and has no `(` at all (#49).
    quiet("MARA\nCall me (later)", C::PartialParenthetical);
    quiet("MARA\nIt's 5 (or 6).", C::PartialParenthetical); // ends with `.`, not `)`
    quiet("MARA\nCall me (later)  ", C::PartialParenthetical);
    quiet("MARA\nIl dit (non)", C::PartialParenthetical);
    quiet("MARA\nWait... (what?)", C::PartialParenthetical);
    fires(
        "MARA\n(looking at the\ndoor)",
        C::PartialParenthetical,
        "(looking at the",
    );
    fires(
        "MARA\n(looking at the\ndoor)",
        C::PartialParenthetical,
        "door)",
    );
    fires("MARA\nhi\n)", C::PartialParenthetical, ")");
    fires(
        "MARA\nhi\ndoor) [[ (note ]]",
        C::PartialParenthetical,
        "door) [[ (note ]]",
    );
}

/// DIAG-blank-lookalike
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

/// DIAG-adjacent-outline-line
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

/// DIAG-ambiguous-equals
#[test]
fn diag_ambiguous_equals() {
    fires("==", C::AmbiguousEquals, "==");
    fires("=== text", C::AmbiguousEquals, "=== text");
    quiet("===", C::AmbiguousEquals);
    quiet("= syn", C::AmbiguousEquals);
    quiet("  ===  ", C::AmbiguousEquals);
}

/// DIAG-lone-caret
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

/// DIAG-nested-opener
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

/// DIAG-unclosed-note
#[test]
fn diag_unclosed_note() {
    fires("a [[b\n\nc", C::UnclosedNote, "[[");
    quiet("a [[b\nc]]", C::UnclosedNote);
    quiet("a [[b\n  \nc]]", C::UnclosedNote);
}

/// DIAG-unclosed-boneyard
#[test]
fn diag_unclosed_boneyard() {
    fires("a /* b", C::UnclosedBoneyard, "/*");
    assert_eq!(offsets("/* a /* b", C::UnclosedBoneyard), [(0, 2), (5, 7)]);
    quiet("a /* b\n\nc */", C::UnclosedBoneyard);
}

/// DIAG-stray-closer
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

/// DIAG-emphasis-line-break
#[test]
fn diag_emphasis_line_break() {
    fires(
        "She *walks\nslowly* away.",
        C::EmphasisLineBreak,
        "She *walks",
    );
    fires(
        "MARA\nI *never\nsaid* that.",
        C::EmphasisLineBreak,
        "I *never",
    );
    quiet("She *walks* slowly.", C::EmphasisLineBreak);
    quiet("She *walks\n\nslowly* away.", C::EmphasisLineBreak);
    quiet("She *walks\nslowly away.", C::EmphasisLineBreak);
    quiet("She *walks\nslowly _away_.", C::EmphasisLineBreak);
    quiet("MARA\n*a\n(beat)\nb*", C::EmphasisLineBreak);
    quiet("She *walks\n!*slowly away", C::EmphasisLineBreak);
}

#[test]
fn diag_remembered_failures() {
    assert_eq!(offsets("[[a [[b\n\nc", C::UnclosedNote), [(0, 2), (4, 6)]);
    // The BOM is three bytes, so `]]` starts at byte 5.
    assert_eq!(offsets("\u{FEFF}a ]]", C::StrayCloser), [(5, 7)]);
    assert_eq!(offsets("/* a /* b */", C::NestedOpener), [(5, 7)]);
    // OFF-16b: the tail of `/* outer /* inner */ still outer? */` is outside the span.
    let src = "/* outer /* inner */ still outer? */";
    assert_eq!(offsets(src, C::NestedOpener), [(9, 11)]);
    assert_eq!(offsets(src, C::StrayCloser), [(34, 36)]);
}

/// One `none` row of SPEC 6.2: its ID, an input built from the row, and the codes expected on it
/// (empty unless the input is also another row's example).
type QuietRow = (&'static str, &'static str, &'static [C]);

const QUIET_ROWS: &[QuietRow] = &[
    ("OFF-1b", "MARA\nhi\n  \nyo", &[]),
    ("OFF-1c", "a [[note\n  \nstill]] b", &[]),
    ("OFF-1d", "\n  INT. HOUSE\n\nx", &[]),
    ("OFF-2b", "a\n\nb\n", &[]),
    ("OFF-2c", "\u{FEFF}Title: X\n\nx", &[]),
    ("OFF-2d", "a\u{2028}b\u{85}c\u{B}d\u{C}e\n\nx", &[]),
    (
        "OFF-3a",
        "NELL (V.O.)\nhi\n\nMARA (cont'd)\nhi\n\nMARA (on the radio, softly)\nhi",
        &[],
    ),
    ("OFF-3b", "OTTO (V.O.) (CONT'D)\nhi", &[]),
    (
        "OFF-3c",
        "OTTO (V.O.) hi\nyo\n\nOTTO (V.O.\nyo\n\nOTTO ((x))\nyo",
        &[],
    ),
    ("OFF-3d", "@deLaCruz (V.O.)\nhi", &[]),
    ("OFF-3e", "MARA (CONT'D)\nhi", &[]),
    (
        "OFF-4a",
        "JOS\u{C9}\nhi\n\n\u{415}\u{41B}\u{415}\u{41D}\u{410}\nhi\n\nelena\nhi",
        &[],
    ),
    ("OFF-4d", "R2D2\nhi\n\nMR. O'BRIEN\nhi", &[]),
    ("OFF-5a", "a\n\nMARA", &[]),
    ("OFF-5b", "MARA\n\nhi", &[]),
    ("OFF-5b-whitespace", "MARA\n  \nhi", &[C::BlankLookalike]),
    (
        "OFF-6b",
        "INT HOUSE\n\nx\n\nINT\tHOUSE\n\nx\n\nINT-HOUSE\n\nx\n\nINTERIOR\n\nx",
        &[],
    ),
    ("OFF-6c", "Ext. 12 is busy.\n\nx", &[]),
    ("OFF-6e", "INT. A\n\nx", &[]),
    ("OFF-6e-title", "Title: X\n\nINT. A\n\nx", &[]),
    (
        "OFF-7a",
        ".LAMP ROOM\n\nx\n\n.5 HOURS LATER\n\nx\n\n.\u{C9}PILOGUE",
        &[],
    ),
    ("OFF-7c", "..DOUBLE\n\n...trailing off", &[]),
    ("OFF-7d", ".OLD HOUSE\nShe waits.", &[]),
    (
        "OFF-8a",
        "INT. A #12#\n\nx\n\nINT. B #4B#\n\nx\n\nINT. C #\u{C9}1#\n\nx\n\nINT. D #2-A.1#\n\nx",
        &[],
    ),
    ("OFF-8g", "INT. A ##12A#\n\nx", &[]),
    ("OFF-8d", ".TITLE CARD #3#\n\nx", &[]),
    ("OFF-8f", "INT. A #5#\n\nx", &[]),
    ("OFF-9d", "a\n\nCUT TO:\n\nb\n\nCut to:\n\nx", &[]),
    ("OFF-9e", ">Cut to\n\nx\n\n> any thing\nx", &[]),
    ("OFF-10a", ">FIN<\n\n> FIN\n\nx", &[]),
    ("OFF-10b", ">FIN< \n\nx", &[]),
    ("OFF-10c", ">> FIN <<\n\nx", &[]),
    ("OFF-10d", "> FIN\n\n>FIN< and more\n\nx", &[]),
    ("OFF-10e", "> FIN <\n\nx", &[]),
    ("OFF-11b", "MARA\nhi\n\nOTTO^\nyo", &[]),
    ("OFF-11e", "MARA\nhi\n\n[[note]]\n\nOTTO ^\nyo", &[]),
    (
        "OFF-11e-broken",
        "MARA\nhi\n\nShe waits.\n\nOTTO ^\nyo",
        &[C::LoneCaret],
    ),
    ("OFF-12a", "MARA\n(softly)\nhi", &[]),
    ("OFF-12c", "(softly)\nhi", &[]),
    ("OFF-13a", "MARA\nhi\n~la la\nmore", &[]),
    ("OFF-13b", "~la la", &[]),
    ("OFF-13c", " ~ indented lyric", &[]),
    ("OFF-13d", "~a\n~b\n\n~c", &[]),
    (
        "OFF-14a",
        "She *walks* slowly, **bold**, ***both*** and _under_.",
        &[],
    ),
    ("OFF-14c", "snake_case_name", &[]),
    (
        "OFF-14d",
        "*start _nest* end_\n\n***triple*\n\n*bold **inside** italic*",
        &[],
    ),
    ("OFF-14e", "\\_literal\\_ \\\\ \\# trailing\\", &[]),
    (
        "OFF-14f",
        "Mara [[*note*]] and /* _ */ **INT. X**\n\nx",
        &[],
    ),
    ("OFF-15d", "a [[two\nlines]] b", &[]),
    ("OFF-15e", "MARA\nhi\n[[note]]\nmore", &[]),
    ("OFF-15f", "a [[x /* y]] z", &[]),
    ("OFF-15f-title", "Title: X\n[[n]]\n   v\n\nx", &[]),
    (
        "OFF-15g",
        "a [[b\n\nc]]",
        &[C::UnclosedNote, C::StrayCloser],
    ),
    ("OFF-16a", "Mara /* aside */ waits.", &[]),
    ("OFF-16e", "/* x */INT. A\n\nx", &[]),
    (
        "OFF-16e-layout",
        "a\n\nCUT TO:\n/* x */\nINT. A\n\nb",
        &[C::UnframedTransition, C::UnframedHeading],
    ),
    ("OFF-17a", "Title: X\n\nINT. A\n\nx", &[]),
    ("OFF-17a-prose", "Some prose: with a colon\n\nx", &[]),
    ("OFF-17b", "Title: X\nAuthor: Y\n\nx", &[]),
    ("OFF-17c", "TITLE: A\ntitle: B\nCustom Key: C\n\nx", &[]),
    (
        "OFF-17e",
        "Title: X\n   more\nAuthor:\n   Me\nDraft date: 5:30 pm\nEmpty:\n\nx",
        &[],
    ),
    ("OFF-17f", "\n\nTitle: X\n\nINT. A\n\nx", &[]),
    (
        "OFF-18a",
        "   INT. HOUSE\n\n   MARA\n   hi\n\n  CUT TO:\n\nx",
        &[],
    ),
    ("OFF-18b", "  indented action\n\nx", &[]),
    ("OFF-18c", "\ttab action\n\nx", &[]),
    ("OFF-18d", "    MARA\n\t\thi\n\n          CUT TO:\n\nx", &[]),
    ("OFF-19a", "===\n\n=====\n\nx", &[]),
    ("OFF-19c", "=\n\n= text\n\n= = =\n\nx", &[]),
    ("OFF-19d", "  ===\n\n===  \n\nx", &[]),
    ("OFF-20a", "#Section\n\n#1 priority item\n\nx", &[]),
    ("OFF-20b", "#\n\n# \n\nx", &[]),
    ("OFF-20c", "# a\n\n#### b\n\n############ c\n\nx", &[]),
    ("OFF-20d", "INT. HOUSE - DAY #5#\n\n# 12 #\n\nx", &[]),
    ("OFF-21a", "MARA\nhi\n# x\nyo", &[C::AdjacentOutlineLine]),
    (
        "OFF-21b",
        "She walks\n# x\nthen runs",
        &[C::AdjacentOutlineLine],
    ),
    ("OFF-22a", "  !indented action\n\nx", &[]),
    ("OFF-22b", "!.dot\n\n@!after at\n\n!>x\n\nx", &[]),
    ("OFF-22c", ">>> a line of arrows\n\nx", &[]),
    ("OFF-22d", ".h\n\n!a\n\n@NAME\nhi\n\n~l\n\n>t\n\n>c<", &[]),
    ("OFF-22f", "!\n\n~\n\n@\n\n!   \n\nx", &[]),
    ("OFF-22g", "!!x\n\nx", &[]),
    ("OFF-23c", "MARA\nhi\n\nOTTO ^ \nyo", &[]),
    (
        "OFF-23d",
        "MARA\nhi\n\nOTTO ^  \nyo\n\nShe waits.  \nthen goes.",
        &[],
    ),
    ("OFF-23e", "MARA\r\nhi\r\n\r\nOTTO ^\r\nyo", &[]),
    ("OFF-24a", "MARA\n(softly)\nhi\nmore\n\nx", &[]),
    // The text line right after the Page Break is `adjacent-outline-line` (OFF-21c).
    (
        "OFF-24c",
        "MARA\nhi\n~la\n# s\n= syn\n===\nmore",
        &[C::AdjacentOutlineLine],
    ),
    ("OFF-24d", "MARA\nhi\n[[n]] /* b */\nmore", &[]),
];

#[test]
fn diag_quiet_rows() {
    let mut failures = Vec::new();
    for (id, src, expected) in QUIET_ROWS {
        let got: Vec<C> = parse_checked(src)
            .diagnostics()
            .iter()
            .map(|d| d.code)
            .collect();
        if got != *expected {
            failures.push(format!("{id} {src:?}: expected {expected:?}, got {got:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn diag_clean_screenplay_is_quiet() {
    let src = "\
Title: The Long Wait
Author: A. Writer
Notes: First line of the notes
    and a second line.

# Act One

= The boat is finished.

INT. WORKSHOP - NIGHT #1#

Mara sands a *small* boat. The **old** radio hums.
She waits [[check the tide]] by the door.

[[Whole-line note.]]

/* A cut whole-line boneyard. */

MARA (V.O.)
(softly)
I never told him.
Call me (later)

OTTO ^
Nor did I.

~Row, row, row the boat.

CUT TO:

EXT. HARBOR - DAY #2#

>THE END<
";
    assert_eq!(diags(src), []);
}
