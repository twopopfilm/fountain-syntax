//! Line classification: pure predicates over a line's text. [`shape`] applies the first step of
//! SPEC 4.0 (a sigil or structural marker as the first non-whitespace character).

use std::ops::Range;

/// Alphanumeric for `OFF-7` and `OFF-8`.
// shortcut: `char::is_alphanumeric` also takes Nl/No characters such as `½`, a little wider than
// SPEC 3.5's letter or decimal digit; upgrade when the standard library exposes General_Category.
pub(crate) fn is_alnum(c: char) -> bool {
    c.is_alphanumeric()
}

/// What a visible line is by its first non-whitespace character.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LineShape {
    /// `~`.
    Lyrics,
    /// `!`.
    ForcedAction,
    /// `!!` with the `shot` feature.
    #[cfg(feature = "shot")]
    Shot,
    /// `@` followed by a non-whitespace character.
    ForcedCue,
    /// `.` followed by an alphanumeric character.
    ForcedHeading,
    /// A heading prefix and a period or space (SPEC 4.2). It is a heading only when framed by
    /// blank lines; the framing pass (`frame` in parse.rs) demotes it to [`LineShape::Plain`]
    /// otherwise.
    PrefixHeading,
    /// `>` first and `<` last.
    Centered,
    /// Any other line that starts with `>`.
    ForcedTransition,
    /// `#`s; the count is the depth.
    Section(usize),
    /// A single `=` that does not make a Page Break.
    Synopsis,
    /// Three or more `=` and nothing else.
    PageBreak,
    /// No sigil or marker applies.
    Plain,
}

/// Classifies a line whose leading whitespace is already removed (SPEC 4.0, step 1).
pub(crate) fn shape(trimmed_start: &str) -> LineShape {
    let mut chars = trimmed_start.chars();
    match chars.next() {
        Some('~') => LineShape::Lyrics,
        #[cfg(feature = "shot")]
        Some('!') if trimmed_start.starts_with("!!") => LineShape::Shot,
        Some('!') => LineShape::ForcedAction,
        Some('@') if chars.next().is_some_and(|c| !c.is_whitespace()) => LineShape::ForcedCue,
        Some('.') if chars.next().is_some_and(is_alnum) => LineShape::ForcedHeading,
        Some('>') if trimmed_start.trim_end_matches([' ', '\t']).ends_with('<') => {
            LineShape::Centered
        }
        Some('>') => LineShape::ForcedTransition,
        Some('#') => LineShape::Section(trimmed_start.chars().take_while(|&c| c == '#').count()),
        Some('=') => {
            let run = trimmed_start.chars().take_while(|&c| c == '=').count();
            let rest = &trimmed_start[run..];
            if run >= 3 && rest.trim_matches([' ', '\t']).is_empty() {
                LineShape::PageBreak
            } else if run == 1 {
                LineShape::Synopsis
            } else {
                LineShape::Plain
            }
        }
        _ if heading_prefix(trimmed_start) => LineShape::PrefixHeading,
        _ => LineShape::Plain,
    }
}

/// Whether a line, with its leading whitespace removed and its trailing whitespace kept, starts with
/// a heading prefix in any case and a period or space (`OFF-6b`, `OFF-6d`).
pub(crate) fn heading_prefix(trimmed_start: &str) -> bool {
    heading_prefix_len(trimmed_start).is_some()
}

/// The byte length of the heading prefix and its period or space, when the line starts with one.
pub(crate) fn heading_prefix_len(trimmed_start: &str) -> Option<usize> {
    let bytes = trimmed_start.as_bytes();
    ["INT./EXT", "INT/EXT", "I/E", "INT", "EXT", "EST"]
        .iter()
        .find(|p| {
            bytes.len() > p.len()
                && bytes[..p.len()].eq_ignore_ascii_case(p.as_bytes())
                && matches!(bytes[p.len()], b'.' | b' ')
        })
        .map(|p| p.len() + 1)
}

/// Uppercase text (SPEC 3.5): at least one uppercase letter and no lowercase one.
pub(crate) fn is_uppercase_text(text: &str) -> bool {
    text.chars().any(char::is_uppercase) && !text.chars().any(char::is_lowercase)
}

/// The forced lines that end a Dialogue block (`OFF-24b`). Such a line is also never the text line
/// a cue needs after it.
pub(crate) fn ends_block(shape: LineShape) -> bool {
    match shape {
        LineShape::ForcedAction
        | LineShape::ForcedCue
        | LineShape::ForcedHeading
        | LineShape::Centered
        | LineShape::ForcedTransition => true,
        #[cfg(feature = "shot")]
        LineShape::Shot => true,
        _ => false,
    }
}

/// An automatic Transition, read with the leading whitespace removed: uppercase and ending in
/// exactly `TO:` (`OFF-9d`).
pub(crate) fn transition(trimmed_start: &str) -> bool {
    is_uppercase_text(trimmed_start) && trimmed_start.ends_with("TO:")
}

/// An uppercase line ending in `TO:` and then spaces or tabs: never a Transition or a cue
/// (`OFF-23a`, `OFF-23b`).
pub(crate) fn to_trailing_space(text: &str) -> bool {
    let body = text.trim_end_matches([' ', '\t']);
    body.len() < text.len() && transition(body)
}

/// The parts of a Character cue, as ranges into the text given to [`cue`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CueParts {
    /// The name, without the whitespace after it. A forced cue with malformed groups keeps them as
    /// name text, so the range runs to the end of the text before any caret. A forced cue with an
    /// empty name takes the whole text, caret included, and has no caret.
    pub(crate) name: Range<usize>,
    /// Each extension group, `(` to `)`.
    pub(crate) extensions: Vec<Range<usize>>,
    /// The `^` of a dual-dialogue cue.
    pub(crate) caret: Option<usize>,
}

/// Reads a cue from a line trimmed at both ends, after the `@` when `forced` (SPEC 4.4).
///
/// A caret counts only as the last character (`OFF-11c`). The name runs to the first `(`; after it
/// come groups `(…)` with no `(` inside, separated by whitespace only (`OFF-3b`, `OFF-3c`). An
/// automatic cue needs an uppercase name (`OFF-3a`, `OFF-4`). A forced cue always reads: with
/// malformed groups the remainder is the name, and an empty name takes the whole text.
pub(crate) fn cue(trimmed: &str, forced: bool) -> Option<CueParts> {
    let caret = trimmed.ends_with('^').then(|| trimmed.len() - 1);
    let body = trimmed[..caret.unwrap_or(trimmed.len())].trim_end_matches([' ', '\t']);
    let open = body.find('(').unwrap_or(body.len());
    let name = 0..body[..open].trim_end_matches([' ', '\t']).len();
    let parts = |name, extensions| {
        Some(CueParts {
            name,
            extensions,
            caret,
        })
    };
    match groups(body, open) {
        _ if forced && name.is_empty() => Some(CueParts {
            name: 0..trimmed.len(),
            extensions: Vec::new(),
            caret: None,
        }),
        Some(ext) if forced || is_uppercase_text(&body[name.clone()]) => parts(name, ext),
        None if forced => parts(0..body.len(), Vec::new()),
        _ => None,
    }
}

/// The extension groups of `body` from `at` on, or `None` when anything else is there.
fn groups(body: &str, mut at: usize) -> Option<Vec<Range<usize>>> {
    let mut out = Vec::new();
    while at < body.len() {
        at = body.len() - body[at..].trim_start_matches([' ', '\t']).len();
        let inner = body[at..].strip_prefix('(')?;
        let close = inner.find(['(', ')'])?;
        if inner.as_bytes()[close] != b')' {
            return None;
        }
        out.push(at..at + close + 2);
        at += close + 2;
    }
    Some(out)
}

/// The parts of a title-page key line, as offsets into the text given to [`key_line`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct KeyParts {
    /// The key, without the spaces before the colon.
    pub(crate) key: Range<usize>,
    /// The first colon.
    pub(crate) colon: usize,
}

/// Reads a title-page key line, with its leading whitespace removed (SPEC 4.1, `OFF-17a`).
///
/// The key runs to the first colon and holds alphanumerics, spaces, `-`, `_`, `'` and `&`, at least
/// one of them alphanumeric. A line the heading-prefix test matches is never a key line.
pub(crate) fn key_line(trimmed_start: &str) -> Option<KeyParts> {
    let colon = trimmed_start
        .find(':')
        .filter(|_| !heading_prefix(trimmed_start))?;
    let key = &trimmed_start[..colon];
    let allowed = |c: char| is_alnum(c) || matches!(c, ' ' | '-' | '_' | '\'' | '&');
    (key.chars().all(allowed) && key.chars().any(is_alnum)).then(|| KeyParts {
        key: 0..key.trim_end_matches(' ').len(),
        colon,
    })
}

/// Whether a line is indented as a title-page continuation: a tab, or three or more spaces
/// (`OFF-17d`).
pub(crate) fn continuation_indent(line: &str) -> bool {
    let lead = &line[..line.len() - line.trim_start_matches([' ', '\t']).len()];
    lead.contains('\t') || lead.len() >= 3
}

/// The scene number that ends a heading line, as the range of the whole `#…#` (`##…#` when locked)
/// and whether it is locked (`OFF-8`). It follows a space or tab and holds alphanumerics, `-` and
/// `.`; only trailing whitespace may follow. A locked number needs the `locked-scene-numbers` feature.
pub(crate) fn scene_number(text: &str) -> Option<(Range<usize>, bool)> {
    let body = text.trim_end_matches([' ', '\t']);
    let inner = body.strip_suffix('#')?;
    let run = inner
        .chars()
        .rev()
        .take_while(|&c| is_alnum(c) || c == '-' || c == '.')
        .map(char::len_utf8)
        .sum::<usize>();
    let open = inner.len().checked_sub(run + 1).filter(|_| run > 0)?;
    let spaced = |at: usize| {
        matches!(
            body.as_bytes().get(..at).and_then(<[u8]>::last),
            Some(b' ' | b'\t')
        )
    };
    if inner.as_bytes().get(open) != Some(&b'#') {
        return None;
    }
    if cfg!(feature = "locked-scene-numbers")
        && open > 0
        && inner.as_bytes()[open - 1] == b'#'
        && spaced(open - 1)
    {
        return Some((open - 1..body.len(), true));
    }
    spaced(open).then_some((open..body.len(), false))
}

/// Whether a heading line ends with something shaped like a scene number that `scene_number` does
/// not take (`OFF-8b`, `OFF-8c`, `OFF-8e`): a `#` after an earlier `#` at the end, or a `#…#` after a
/// space or tab with text following. The locked form `##<run>#` after a space or tab is not one
/// (`OFF-8g`); it is a number or plain text, as the feature decides.
pub(crate) fn rejected_scene_number(text: &str) -> bool {
    if scene_number(text).is_some() {
        return false;
    }
    let body = text.trim_end_matches([' ', '\t']);
    let number_char = |c: &char| is_alnum(*c) || *c == '-' || *c == '.';
    let locked = body.strip_suffix('#').is_some_and(|inner| {
        let run: usize = inner
            .chars()
            .rev()
            .take_while(number_char)
            .map(char::len_utf8)
            .sum();
        inner[..inner.len() - run]
            .strip_suffix("##")
            .is_some_and(|b| run > 0 && b.ends_with([' ', '\t']))
    });
    if !locked && body.ends_with('#') && body[..body.len() - 1].contains('#') {
        return true;
    }
    body.match_indices('#').any(|(at, _)| {
        let rest = &body[at + 1..];
        let run: usize = rest
            .chars()
            .take_while(number_char)
            .map(char::len_utf8)
            .sum();
        body[..at].ends_with([' ', '\t'])
            && run > 0
            && rest[run..]
                .strip_prefix('#')
                .is_some_and(|after| !after.is_empty())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes() {
        assert_eq!(shape("~ la"), LineShape::Lyrics);
        assert_eq!(shape("!x"), LineShape::ForcedAction);
        assert_eq!(shape("@MARA"), LineShape::ForcedCue);
        assert_eq!(shape("@"), LineShape::Plain);
        assert_eq!(shape("@ x"), LineShape::Plain);
        assert_eq!(shape(".INT"), LineShape::ForcedHeading);
        assert_eq!(shape(". INT"), LineShape::Plain);
        assert_eq!(shape("..x"), LineShape::Plain);
        assert_eq!(shape(".>HEADING"), LineShape::Plain);
        assert_eq!(shape(">x<"), LineShape::Centered);
        assert_eq!(shape(">x< \t"), LineShape::Centered);
        assert_eq!(shape(">x< y"), LineShape::ForcedTransition);
        assert_eq!(shape(">"), LineShape::ForcedTransition);
        assert_eq!(shape("##x"), LineShape::Section(2));
        assert_eq!(shape("="), LineShape::Synopsis);
        assert_eq!(shape("= = ="), LineShape::Synopsis);
        assert_eq!(shape("==="), LineShape::PageBreak);
        assert_eq!(shape("=== "), LineShape::PageBreak);
        assert_eq!(shape("=="), LineShape::Plain);
        assert_eq!(shape("=== x"), LineShape::Plain);
        assert_eq!(shape("x"), LineShape::Plain);
        assert_eq!(shape(""), LineShape::Plain);
    }

    #[test]
    fn heading_prefixes() {
        for yes in [
            "INT. X",
            "int x",
            "Ext.",
            "EST ",
            "INT./EXT x",
            "int/ext. x",
            "I/E.",
            "i/e x",
        ] {
            assert!(heading_prefix(yes), "{yes:?}");
        }
        for no in [
            "INT", "INT\tX", "INT-X", "INTERIOR", "I/EXT. X", "IN. X", "", "ÉINT. X",
        ] {
            assert!(!heading_prefix(no), "{no:?}");
        }
        assert_eq!(shape("INT. X"), LineShape::PrefixHeading);
        assert_eq!(shape("INTERIOR"), LineShape::Plain);
    }

    #[test]
    fn scene_numbers() {
        assert_eq!(scene_number("INT. X #12A#"), Some((7..12, false)));
        assert_eq!(scene_number("INT. X\t#1#  "), Some((7..10, false)));
        assert_eq!(scene_number("INT. #É1#"), Some((5..10, false)));
        assert_eq!(scene_number("INT. #2-A.1#"), Some((5..12, false)));
        assert_eq!(scene_number("X #1# #2#"), Some((6..9, false)));
        assert_eq!(scene_number("X #1#"), Some((2..5, false)));
        assert_eq!(scene_number("a #12#"), Some((2..6, false)));
        for none in [
            "X #1 A#", "X #1_A#", "X #7# y", "X#1#", "X ##", "X #", "X #1", "#1#", "",
        ] {
            assert_eq!(scene_number(none), None, "{none:?}");
        }
    }

    #[test]
    fn rejected_scene_numbers() {
        for yes in [
            "X #1 A#", "X #1_A#", "X #7# y", "X ##", "X #1# #", ".T #1 A#",
        ] {
            assert!(rejected_scene_number(yes), "{yes:?}");
        }
        for no in [
            "X #12A#",
            "X #1#  ",
            "X",
            "X #",
            "X #1",
            "X#1# y",
            "Fine #1 A",
            "",
            "X #a",
        ] {
            assert!(!rejected_scene_number(no), "{no:?}");
        }
        // The locked form is not rejected, with or without the feature.
        assert!(!rejected_scene_number("X ##12A#"));
    }

    #[test]
    fn heading_prefix_lens() {
        assert_eq!(heading_prefix_len("INT. X"), Some(4));
        assert_eq!(heading_prefix_len("int/ext x"), Some(8));
        assert_eq!(heading_prefix_len("INT./EXT. X"), Some(9));
        assert_eq!(heading_prefix_len("INTERIOR"), None);
    }

    #[test]
    fn uppercase_text() {
        for yes in ["MARA", "R2D2", "JOSÉ", "ΣΩΚΡΑΤΗΣ", "MR. O'BRIEN", "A 李"] {
            assert!(is_uppercase_text(yes), "{yes:?}");
        }
        for no in ["Mara", "23", "李明", "", "елена", "(...)"] {
            assert!(!is_uppercase_text(no), "{no:?}");
        }
    }

    #[test]
    fn transitions() {
        assert!(transition("CUT TO:"));
        assert!(transition("SMASH CUT TO:"));
        for no in ["Cut to:", "CUT TO: ", "CUT TO:.", "TO", "FADE OUT."] {
            assert!(!transition(no), "{no:?}");
        }
        assert!(to_trailing_space("CUT TO: "));
        assert!(to_trailing_space("CUT TO:\t "));
        for no in ["CUT TO:", "Cut to: ", "CUT TO:. ", "CUT  "] {
            assert!(!to_trailing_space(no), "{no:?}");
        }
    }

    fn cue_of(text: &str, forced: bool) -> Option<(&str, Vec<&str>, Option<usize>)> {
        cue(text, forced).map(|c| {
            let ext = c.extensions.iter().map(|r| &text[r.clone()]).collect();
            (&text[c.name], ext, c.caret)
        })
    }

    #[test]
    fn cues() {
        assert_eq!(cue_of("MARA", false), Some(("MARA", vec![], None)));
        assert_eq!(
            cue_of("OTTO (V.O.) (CONT'D)", false),
            Some(("OTTO", vec!["(V.O.)", "(CONT'D)"], None))
        );
        assert_eq!(
            cue_of("MARA (on the radio)", false),
            Some(("MARA", vec!["(on the radio)"], None))
        );
        assert_eq!(cue_of("OTTO^", false), Some(("OTTO", vec![], Some(4))));
        assert_eq!(
            cue_of("MARA (V.O.) ^", false),
            Some(("MARA", vec!["(V.O.)"], Some(12)))
        );
        assert_eq!(
            cue_of("NAME ^ (V.O.)", false),
            Some(("NAME ^", vec!["(V.O.)"], None))
        );
        assert_eq!(
            cue_of("A ()(B)", false),
            Some(("A", vec!["()", "(B)"], None))
        );
        for no in [
            "Mara (V.O.)",
            "MARA (HI) THERE",
            "MARA (HI",
            "MARA ((HI))",
            "MARA (A (B))",
            "23",
            "李明",
            "(V.O.)",
            "^",
            "",
        ] {
            assert_eq!(cue(no, false), None, "{no:?}");
        }
    }

    #[test]
    fn forced_cues() {
        assert_eq!(
            cue_of("deLaCruz (V.O.)", true),
            Some(("deLaCruz", vec!["(V.O.)"], None))
        );
        assert_eq!(cue_of("NAME ^", true), Some(("NAME", vec![], Some(5))));
        assert_eq!(cue_of("a (b", true), Some(("a (b", vec![], None)));
        assert_eq!(cue_of("a (b ^", true), Some(("a (b", vec![], Some(5))));
        assert_eq!(cue_of("^", true), Some(("^", vec![], None)));
        assert_eq!(cue_of("(V.O.)", true), Some(("(V.O.)", vec![], None)));
        assert_eq!(cue_of("李明", true), Some(("李明", vec![], None)));
    }

    #[test]
    fn block_enders() {
        for s in ["!x", "@X", ".X", ">x<", ">x"] {
            assert!(ends_block(shape(s)), "{s:?}");
        }
        for s in ["~x", "# x", "= x", "===", "x", "INT. X"] {
            assert!(!ends_block(shape(s)), "{s:?}");
        }
    }

    #[test]
    fn key_lines() {
        fn key(t: &str) -> Option<(&str, usize)> {
            key_line(t).map(|k| (&t[k.key], k.colon))
        }
        assert_eq!(key("Title: x"), Some(("Title", 5)));
        assert_eq!(key("Draft date: 5:30"), Some(("Draft date", 10)));
        assert_eq!(key("Title :"), Some(("Title", 6)));
        assert_eq!(key("A-b_c's & d:"), Some(("A-b_c's & d", 11)));
        assert_eq!(key("1:"), Some(("1", 1)));
        assert_eq!(key("É:"), Some(("É", 2)));
        // `EXT:` has no period or space after the prefix, so it is not a heading prefix.
        assert_eq!(key("EXT: x"), Some(("EXT", 3)));
        for no in [
            "",
            ":",
            " :",
            "-: x",
            "'&:",
            "Title",
            "Mr. O: x",
            "a,b: c",
            "a\tb: c",
            "#a: b",
            "INT. A: B",
            "int x: y",
        ] {
            assert_eq!(key_line(no), None, "{no:?}");
        }
    }

    #[test]
    fn continuation_indents() {
        for yes in ["\tx", "   x", "    x", " \tx", "  \tx", "\t"] {
            assert!(continuation_indent(yes), "{yes:?}");
        }
        for no in ["x", " x", "  x", "", "  ", " x\t"] {
            assert!(!continuation_indent(no), "{no:?}");
        }
    }

    #[cfg(feature = "locked-scene-numbers")]
    #[test]
    fn locked_scene_numbers() {
        assert_eq!(scene_number("X ##12A#"), Some((2..8, true)));
        assert_eq!(scene_number("X ##"), None);
        assert_eq!(scene_number("X ###x#"), None);
        assert_eq!(scene_number("X##1#"), None);
    }

    #[cfg(not(feature = "locked-scene-numbers"))]
    #[test]
    fn locked_scene_number_without_flag() {
        assert_eq!(scene_number("X ##12A#"), None);
    }

    #[cfg(feature = "shot")]
    #[test]
    fn shot_shape() {
        assert_eq!(shape("!!x"), LineShape::Shot);
        assert_eq!(shape("!!!x"), LineShape::Shot);
        assert_eq!(shape("!x"), LineShape::ForcedAction);
        assert!(ends_block(LineShape::Shot));
    }

    #[cfg(not(feature = "shot"))]
    #[test]
    fn double_bang_without_shot() {
        assert_eq!(shape("!!x"), LineShape::ForcedAction);
    }
}
