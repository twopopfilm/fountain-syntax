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
    /// blank lines; the block pass demotes it to [`LineShape::Plain`] otherwise.
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
    let bytes = trimmed_start.as_bytes();
    ["INT./EXT", "INT/EXT", "I/E", "INT", "EXT", "EST"]
        .iter()
        .any(|p| {
            bytes.len() > p.len()
                && bytes[..p.len()].eq_ignore_ascii_case(p.as_bytes())
                && matches!(bytes[p.len()], b'.' | b' ')
        })
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
    }

    #[cfg(not(feature = "shot"))]
    #[test]
    fn double_bang_without_shot() {
        assert_eq!(shape("!!x"), LineShape::ForcedAction);
    }
}
