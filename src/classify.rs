//! Line classification: pure predicates over a line's text. [`shape`] applies the first step of
//! SPEC 4.0 (a sigil or structural marker as the first non-whitespace character).

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
        _ => LineShape::Plain,
    }
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
