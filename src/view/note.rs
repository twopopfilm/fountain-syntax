//! Note conventions (SPEC 5.1): colour, marker and tag notes, read on demand.

use super::{Note, NoteConvention};

impl Note {
    /// The convention this note follows, if any (SPEC 5.1, `EXT-notes-view-1`).
    ///
    /// Computed on demand from the trimmed text; nothing is stored and nothing changes
    /// (`EXT-notes-view-2`). The first rule that fits wins:
    ///
    /// 1. **Marker** (as Beat reads it): the word `marker` (any ASCII case), then an optional
    ///    colour word directly followed by `:`, then the text. `marker amber: fix pacing` is
    ///    amber with the text `fix pacing`; `marker: fix` has no colour; `marker fix pacing` has
    ///    no colon, so no colour and the whole rest is the text; a bare `marker` has empty text.
    ///    The colour word is whatever non-space characters sit before the first colon, so
    ///    `marker call at 12:30` has no colour. `markers are fun` is not a marker.
    /// 2. **Colour**: `color` or `colour` (any ASCII case) and one word, or `#` and exactly six
    ///    hex digits, or a single word made of letters only. The word comes back as typed.
    /// 3. **Tag** (as Slugline reads it): one word with no whitespace, then `:`, then the text
    ///    (trimmed). `Check: confirm tides` is the tag `Check`. The tag is one word and its
    ///    length is not limited.
    /// 4. Otherwise `None`, and so for an empty note.
    ///
    /// The colour names and the tag length are not determined by the sources (SPEC 13.5), so the
    /// words are returned as typed and their meaning is the caller's: `[[Tomorrow]]` is a
    /// [`NoteConvention::Colour`] here because it is one word of letters. Never panics.
    pub fn conventions(&self) -> Option<NoteConvention> {
        let t = self.text.trim();
        marker(t).or_else(|| colour(t)).or_else(|| tag(t))
    }
}

/// `head: tail` where `head` is a non-empty word before the first colon. `tail` is trimmed.
fn split_tag(s: &str) -> Option<(&str, &str)> {
    let (head, tail) = s.split_once(':')?;
    let one_word = !head.is_empty() && !head.contains(char::is_whitespace);
    one_word.then(|| (head, tail.trim()))
}

fn marker(t: &str) -> Option<NoteConvention> {
    const WORD: &str = "marker";
    // `get` is None when byte 6 is inside a multi-byte character, so no slicing can panic.
    if !t.get(..WORD.len())?.eq_ignore_ascii_case(WORD) {
        return None;
    }
    let after = &t[WORD.len()..];
    if let Some(text) = after.strip_prefix(':') {
        return Some(NoteConvention::Marker {
            colour: None,
            text: text.trim().to_owned(),
        });
    }
    if !after.is_empty() && !after.starts_with(char::is_whitespace) {
        return None;
    }
    let rest = after.trim_start();
    Some(match split_tag(rest) {
        Some((colour, text)) => NoteConvention::Marker {
            colour: Some(colour.to_owned()),
            text: text.to_owned(),
        },
        None => NoteConvention::Marker {
            colour: None,
            text: rest.to_owned(),
        },
    })
}

fn colour(t: &str) -> Option<NoteConvention> {
    let word = |w: &str| Some(NoteConvention::Colour(w.to_owned()));
    if let Some((first, rest)) = t.split_once(char::is_whitespace) {
        let rest = rest.trim_start();
        let one_word = !rest.is_empty() && !rest.contains(char::is_whitespace);
        let keyword = first.eq_ignore_ascii_case("color") || first.eq_ignore_ascii_case("colour");
        return if keyword && one_word {
            word(rest)
        } else {
            None
        };
    }
    let hex = t
        .strip_prefix('#')
        .is_some_and(|h| h.len() == 6 && h.bytes().all(|b| b.is_ascii_hexdigit()));
    let letters = !t.is_empty() && t.chars().all(char::is_alphabetic);
    (hex || letters).then(|| NoteConvention::Colour(t.to_owned()))
}

fn tag(t: &str) -> Option<NoteConvention> {
    let (tag, text) = split_tag(t)?;
    Some(NoteConvention::Tag {
        tag: tag.to_owned(),
        text: text.to_owned(),
    })
}
