//! Syntax tree tests: dialogue blocks.

mod common;

use common::{dump, find, parse_checked, top};
use fountain_syntax::NodeKind;

/// Parses `src` and returns the dump of its root.
fn tree(src: &str) -> String {
    dump(parse_checked(src).root())
}

/// The dump of the first `CharacterCue` in `src`, if it has one.
fn cue(src: &str) -> Option<String> {
    let doc = parse_checked(src);
    find(&doc, NodeKind::CharacterCue).first().map(|n| dump(*n))
}

/// Whether `src` starts with a Dialogue block.
fn is_cue(src: &str) -> bool {
    top(&parse_checked(src)).first() == Some(&NodeKind::DialogueBlock)
}

/// EL-character-1
#[test]
fn el_character_1() {
    assert_eq!(
        tree("NELL (V.O.)\nHello."),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"NELL") Whitespace" " Extension(Marker"(" Text"V.O." Marker")")) LineBreak"\n" Dialogue(Text"Hello.")))"#
    );
}

/// EL-character-2
#[test]
fn el_character_2_forced_keeps_case() {
    assert_eq!(
        tree("@McCLANE\nYippee."),
        r#"Document(DialogueBlock(CharacterCue(Sigil"@" CharacterName(Text"McCLANE")) LineBreak"\n" Dialogue(Text"Yippee.")))"#
    );
    // Decision 4: a forced cue is a cue even with nothing after it.
    assert_eq!(
        tree("@MARA\n"),
        r#"Document(DialogueBlock(CharacterCue(Sigil"@" CharacterName(Text"MARA"))) Gap(LineBreak"\n"))"#
    );
    assert_eq!(
        tree("a\n@MARA\n\nb"),
        r#"Document(Action(Text"a") Gap(LineBreak"\n") DialogueBlock(CharacterCue(Sigil"@" CharacterName(Text"MARA"))) Gap(LineBreak"\n" LineBreak"\n") Action(Text"b"))"#
    );
}

/// EL-character-3
#[test]
fn el_character_3_last_line_is_action() {
    let doc = parse_checked("a\n\nMARA");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::Action]);
    assert!(find(&doc, NodeKind::CharacterCue).is_empty());
}

/// EL-dialogue-1
#[test]
fn el_dialogue_1() {
    assert_eq!(
        tree("MARA\nline one\n  line two "),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Dialogue(Text"line one") LineBreak"\n" Dialogue(Whitespace"  " Text"line two" Whitespace" ")))"#
    );
}

/// EL-dialogue-2
#[test]
fn el_dialogue_2_two_space_line() {
    assert_eq!(
        tree("MARA\none\n  \ntwo"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Dialogue(Text"one") LineBreak"\n" Dialogue(Whitespace"  ") LineBreak"\n" Dialogue(Text"two")))"#
    );
    // Inside a block a two-space line is not blank: what follows it is still Dialogue.
    let doc = parse_checked("MARA\nhi\n  \nBOB\nyo");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock]);
    assert_eq!(find(&doc, NodeKind::CharacterCue).len(), 1);
    assert_eq!(find(&doc, NodeKind::Dialogue).len(), 4);
    let doc = parse_checked("MARA\nhi\n  \nINT. X\n\nb");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock, NodeKind::Action]);
    assert!(find(&doc, NodeKind::SceneHeading).is_empty());
    assert_eq!(
        dump(find(&doc, NodeKind::Dialogue)[2]),
        r#"Dialogue(Text"INT. X")"#
    );
    // Outside a block it is blank, and frames what follows.
    let doc = parse_checked("Mara\nhi\n  \nBOB\nyo");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::DialogueBlock]);
    let doc = parse_checked("a\n\n  \nINT. X");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::SceneHeading]);
    let doc = parse_checked("a\n  \nINT. X\n  \nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::SceneHeading, NodeKind::Action]
    );
}

/// EL-dialogue-3
#[test]
fn el_dialogue_3_blank_ends_block() {
    assert_eq!(
        tree("MARA\nhi\n\nAfter."),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Dialogue(Text"hi")) Gap(LineBreak"\n" LineBreak"\n") Action(Text"After."))"#
    );
}

/// EL-parenthetical-1
#[test]
fn el_parenthetical_1() {
    assert_eq!(
        tree("MARA\n(quietly)\nhi\n(beat)\nbye"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Parenthetical(Text"(quietly)") LineBreak"\n" Dialogue(Text"hi") LineBreak"\n" Parenthetical(Text"(beat)") LineBreak"\n" Dialogue(Text"bye")))"#
    );
}

/// EL-transition-1
#[test]
fn el_transition_1() {
    let doc = parse_checked("a\n\nCUT TO:\n\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::Transition, NodeKind::Action]
    );
    assert_eq!(
        dump(find(&doc, NodeKind::Transition)[0]),
        r#"Transition(Text"CUT TO:")"#
    );
    assert_eq!(
        tree("  SMASH CUT TO:"),
        r#"Document(Transition(Whitespace"  " Text"SMASH CUT TO:"))"#
    );
}

/// EL-dual-2
#[test]
fn el_dual_2_caret_kept() {
    let doc = parse_checked("MARA\nhi\n\nOTTO ^\nyo");
    assert_eq!(
        top(&doc),
        [NodeKind::DialogueBlock, NodeKind::DialogueBlock]
    );
    let cues = find(&doc, NodeKind::CharacterCue);
    assert_eq!(
        dump(cues[1]),
        r#"CharacterCue(CharacterName(Text"OTTO") Whitespace" " Caret(Marker"^"))"#
    );
    assert_eq!(find(&doc, NodeKind::CharacterName)[1].text(), "OTTO");
}

/// EL-lyrics-2
#[test]
fn el_lyrics_2_lyric_in_dialogue() {
    assert_eq!(
        tree("MARA\nhi\n~la la\n~la\nbye"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Dialogue(Text"hi") LineBreak"\n" Lyrics(Sigil"~" Text"la la" LineBreak"\n" Sigil"~" Text"la") LineBreak"\n" Dialogue(Text"bye")))"#
    );
    // A lyric right after the cue is a text line, so the cue stands.
    assert_eq!(
        tree("MARA\n~la"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Lyrics(Sigil"~" Text"la")))"#
    );
}

/// EL-indent-1
#[test]
fn el_indent_1() {
    for ind in ["\t", "            ", "        ", "    ", ""] {
        let doc = parse_checked(&format!("{ind}MARA\n{ind}(beat)\n{ind}Hello."));
        assert_eq!(top(&doc), [NodeKind::DialogueBlock], "{ind:?}");
        assert_eq!(find(&doc, NodeKind::CharacterName)[0].text(), "MARA");
        assert_eq!(
            find(&doc, NodeKind::Parenthetical)[0].text(),
            format!("{ind}(beat)")
        );
        assert_eq!(
            find(&doc, NodeKind::Dialogue)[0].text(),
            format!("{ind}Hello.")
        );
    }
}

/// OFF-1b
#[test]
fn off_1b_two_spaces_continue() {
    let doc = parse_checked("MARA\nhi\n  \nyo");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock]);
    for blank in [" ", "   ", "\t\t", " \t"] {
        let doc = parse_checked(&format!("MARA\nhi\n{blank}\nyo"));
        assert_eq!(
            top(&doc),
            [NodeKind::DialogueBlock, NodeKind::Action],
            "{blank:?}"
        );
    }
}

/// OFF-3a
#[test]
fn off_3a_name_case_and_extension_case() {
    for src in [
        "NELL (V.O.)",
        "MARA (cont'd)",
        "MARA (on the radio, softly)",
    ] {
        assert!(is_cue(&format!("{src}\nhi")), "{src:?}");
    }
    assert_eq!(
        tree("Mara (V.O.)\nhi"),
        r#"Document(Action(Text"Mara (V.O.)" LineBreak"\n" Text"hi"))"#
    );
}

/// OFF-3b
#[test]
fn off_3b_several_extensions() {
    assert_eq!(
        cue("OTTO (V.O.) (CONT'D)\nhi").as_deref(),
        Some(
            r#"CharacterCue(CharacterName(Text"OTTO") Whitespace" " Extension(Marker"(" Text"V.O." Marker")") Whitespace" " Extension(Marker"(" Text"CONT'D" Marker")"))"#
        )
    );
    assert_eq!(
        cue("OTTO (A)(B)\nhi").as_deref(),
        Some(
            r#"CharacterCue(CharacterName(Text"OTTO") Whitespace" " Extension(Marker"(" Text"A" Marker")") Extension(Marker"(" Text"B" Marker")"))"#
        )
    );
    assert_eq!(
        cue("OTTO ()\nhi").as_deref(),
        Some(
            r#"CharacterCue(CharacterName(Text"OTTO") Whitespace" " Extension(Marker"(" Marker")"))"#
        )
    );
}

/// OFF-3c
#[test]
fn off_3c_text_after_or_bad_group() {
    for src in [
        "MARA (HI) THERE",
        "MARA (HI",
        "MARA ((HI))",
        "MARA (A (B) C)",
    ] {
        let doc = parse_checked(&format!("{src}\nhi"));
        assert_eq!(top(&doc), [NodeKind::Action], "{src:?}");
    }
}

/// OFF-3d
#[test]
fn off_3d_forced_with_extension() {
    assert_eq!(
        cue("@deLaCruz (V.O.)\nhi").as_deref(),
        Some(
            r#"CharacterCue(Sigil"@" CharacterName(Text"deLaCruz") Whitespace" " Extension(Marker"(" Text"V.O." Marker")"))"#
        )
    );
    // Decision 4: malformed groups after a forced name make the whole remainder the name.
    assert_eq!(
        cue("@deLaCruz (V.O. ok\nhi").as_deref(),
        Some(r#"CharacterCue(Sigil"@" CharacterName(Text"deLaCruz (V.O. ok"))"#)
    );
}

/// OFF-3e
#[test]
fn off_3e_contd_kept_as_written() {
    for contd in ["CONT'D", "cont'd", "CONT\u{2019}D"] {
        assert_eq!(
            cue(&format!("MARA ({contd})\nhi")).as_deref(),
            Some(
                format!(
                    r#"CharacterCue(CharacterName(Text"MARA") Whitespace" " Extension(Marker"(" Text"{contd}" Marker")"))"#
                )
                .as_str()
            ),
            "{contd}"
        );
    }
    let doc = parse_checked("MARA\nhi");
    assert!(find(&doc, NodeKind::Extension).is_empty());
}

/// OFF-4a
#[test]
fn off_4a_unicode_uppercase() {
    for name in ["JOSÉ", "ЕЛЕНА", "ΣΩΚΡΑΤΗΣ"] {
        assert!(is_cue(&format!("{name}\nhi")), "{name}");
    }
    assert!(!is_cue("елена\nhi"));
}

/// OFF-4b
#[test]
fn off_4b_caseless_name() {
    assert!(!is_cue("李明\nhi"));
    assert!(is_cue("@李明\nhi"));
}

/// OFF-4c
#[test]
fn off_4c_no_letter() {
    assert!(!is_cue("23\nhi"));
    assert!(!is_cue("(V.O.)\nhi"));
}

/// OFF-4d
#[test]
fn off_4d_digits_and_punctuation() {
    for name in ["R2D2", "MR. O'BRIEN"] {
        assert_eq!(
            cue(&format!("{name}\nhi")).as_deref(),
            Some(format!(r#"CharacterCue(CharacterName(Text"{name}"))"#).as_str())
        );
    }
}

/// OFF-5a
#[test]
fn off_5a_last_line_is_action() {
    assert_eq!(tree("MARA"), r#"Document(Action(Text"MARA"))"#);
    assert_eq!(top(&parse_checked("a\n\nMARA\n")), [NodeKind::Action; 2]);
    // Only invisible and transparent lines after it: still the end.
    assert_eq!(
        top(&parse_checked("MARA\n[[n]]\n# s")),
        [NodeKind::Action, NodeKind::Note, NodeKind::Section]
    );
}

/// OFF-5b
#[test]
fn off_5b_blank_after_is_action() {
    for blank in ["", " ", "\t", "  "] {
        let doc = parse_checked(&format!("MARA\n{blank}\nhi"));
        assert_eq!(top(&doc), [NodeKind::Action; 2], "{blank:?}");
    }
}

/// OFF-9a
#[test]
fn off_9a_only_to_makes_a_transition() {
    for line in ["FADE OUT.", "FADE IN:", "CUT TO BLACK."] {
        let doc = parse_checked(&format!("a\n\n{line}\n\nb"));
        assert_eq!(top(&doc), [NodeKind::Action; 3], "{line}");
    }
}

/// OFF-9b
#[test]
fn off_9b_between_text_lines() {
    assert_eq!(
        tree("a\nBACK TO:\nb"),
        r#"Document(Action(Text"a" LineBreak"\n" Text"BACK TO:" LineBreak"\n" Text"b"))"#
    );
}

/// OFF-9c
#[test]
fn off_9c_text_under_is_a_cue() {
    assert_eq!(
        tree("\nCUT TO:\nINT. X"),
        r#"Document(Gap(LineBreak"\n") DialogueBlock(CharacterCue(CharacterName(Text"CUT TO:")) LineBreak"\n" Dialogue(Text"INT. X")))"#
    );
}

/// OFF-9d
#[test]
fn off_9d_uppercase_and_exact() {
    for line in ["Cut to:", "CUT TO:.", "CUT TO"] {
        let doc = parse_checked(&format!("a\n\n{line}\n\nb"));
        assert_eq!(top(&doc), [NodeKind::Action; 3], "{line}");
    }
}

/// OFF-9e
#[test]
fn off_9e_forced() {
    let doc = parse_checked("a\n>cut to:\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::Transition, NodeKind::Action]
    );
}

/// OFF-11b
#[test]
fn off_11b_no_space_caret() {
    assert_eq!(
        cue("OTTO^\nhi").as_deref(),
        Some(r#"CharacterCue(CharacterName(Text"OTTO") Caret(Marker"^"))"#)
    );
}

/// OFF-11c
#[test]
fn off_11c_caret_and_extension() {
    assert_eq!(
        cue("MARA (V.O.) ^\nhi").as_deref(),
        Some(
            r#"CharacterCue(CharacterName(Text"MARA") Whitespace" " Extension(Marker"(" Text"V.O." Marker")") Whitespace" " Caret(Marker"^"))"#
        )
    );
    assert_eq!(
        cue("NAME ^ (V.O.)\nhi").as_deref(),
        Some(
            r#"CharacterCue(CharacterName(Text"NAME ^") Whitespace" " Extension(Marker"(" Text"V.O." Marker")"))"#
        )
    );
    assert_eq!(
        cue("@NAME ^\nhi").as_deref(),
        Some(r#"CharacterCue(Sigil"@" CharacterName(Text"NAME") Whitespace" " Caret(Marker"^"))"#)
    );
    // A forced cue whose name would be empty keeps the whole remainder as its name.
    assert_eq!(
        cue("@^").as_deref(),
        Some(r#"CharacterCue(Sigil"@" CharacterName(Text"^"))"#)
    );
    assert_eq!(
        cue("@(V.O.)").as_deref(),
        Some(r#"CharacterCue(Sigil"@" CharacterName(Text"(V.O.)"))"#)
    );
}

/// OFF-12a
#[test]
fn off_12a_parenthetical_shape() {
    assert_eq!(
        tree("MARA\n  (beat) \nhi"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Parenthetical(Whitespace"  " Text"(beat)" Whitespace" ") LineBreak"\n" Dialogue(Text"hi")))"#
    );
    let doc = parse_checked("MARA\n()\nhi");
    assert_eq!(find(&doc, NodeKind::Parenthetical).len(), 1);
}

/// OFF-12b
#[test]
fn off_12b_partial_parentheses_are_dialogue() {
    let doc = parse_checked("MARA\n(whispering) and then speaks\n(not closed\n(two\nlines)");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock]);
    assert!(find(&doc, NodeKind::Parenthetical).is_empty());
    assert_eq!(find(&doc, NodeKind::Dialogue).len(), 4);
}

/// OFF-12c
#[test]
fn off_12c_no_cue_above() {
    assert_eq!(
        tree("a\n\n(beat)\nhi"),
        r#"Document(Action(Text"a") Gap(LineBreak"\n" LineBreak"\n") Action(Text"(beat)" LineBreak"\n" Text"hi"))"#
    );
}

/// OFF-13a
#[test]
fn off_13a_lyric_in_block() {
    let doc = parse_checked("MARA\nhi\n~la\nyo\n\nb");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock, NodeKind::Action]);
    let block = find(&doc, NodeKind::DialogueBlock)[0];
    assert_eq!(find(&doc, NodeKind::Lyrics)[0].parent(), Some(block));
    assert_eq!(find(&doc, NodeKind::Dialogue)[1].text(), "yo");
}

/// OFF-18d
#[test]
fn off_18d_indent_levels() {
    let doc =
        parse_checked("\t\t\t\tMARA\n\t\t\t(beat)\n\t\tHello.\n\n\t\t\t\t\t\tCUT TO:\n\n\tAfter.");
    assert_eq!(
        top(&doc),
        [
            NodeKind::DialogueBlock,
            NodeKind::Transition,
            NodeKind::Action
        ]
    );
    assert_eq!(find(&doc, NodeKind::Parenthetical).len(), 1);
    assert_eq!(find(&doc, NodeKind::Dialogue).len(), 1);
}

/// OFF-21a
#[test]
fn off_21a_outline_in_dialogue() {
    assert_eq!(
        tree("MARA\nhi\n# s\n= syn\nyo"),
        r###"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Dialogue(Text"hi") LineBreak"\n" Section(Marker"#" Whitespace" " Text"s") LineBreak"\n" Synopsis(Marker"=" Whitespace" " Text"syn") LineBreak"\n" Dialogue(Text"yo")))"###
    );
    // Decision 2: an outline line after the block's last member is top-level.
    let doc = parse_checked("MARA\nhi\n# s\n\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::DialogueBlock, NodeKind::Section, NodeKind::Action]
    );
    // Transparent: the cue looks past it for its text line.
    let doc = parse_checked("MARA\n# s\nhi");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock]);
}

/// OFF-23a
#[test]
fn off_23a_to_with_trailing_space_alone() {
    for src in ["CUT TO: ", "a\n\nCUT TO:\t\n\nb"] {
        let doc = parse_checked(src);
        assert!(find(&doc, NodeKind::Transition).is_empty(), "{src:?}");
    }
    assert_eq!(
        tree("CUT TO: "),
        r#"Document(Action(Text"CUT TO:" Whitespace" "))"#
    );
}

/// OFF-23b
#[test]
fn off_23b_to_trailing_space_with_text_after() {
    assert_eq!(
        tree("\nCUT TO: \nnext"),
        r#"Document(Gap(LineBreak"\n") Action(Text"CUT TO:" Whitespace" " LineBreak"\n" Text"next"))"#
    );
}

/// OFF-23c
#[test]
fn off_23c_caret_then_trailing_space() {
    assert_eq!(
        cue("MARA ^ \t\nhi").as_deref(),
        Some(
            r#"CharacterCue(CharacterName(Text"MARA") Whitespace" " Caret(Marker"^") Whitespace" \t")"#
        )
    );
}

/// OFF-23d
#[test]
fn off_23d_two_trailing_spaces_mean_nothing() {
    assert_eq!(
        cue("MARA  \nhi").as_deref(),
        Some(r#"CharacterCue(CharacterName(Text"MARA") Whitespace"  ")"#)
    );
    assert_eq!(
        tree("a  \nb"),
        r#"Document(Action(Text"a" Whitespace"  " LineBreak"\n" Text"b"))"#
    );
}

/// OFF-23e
#[test]
fn off_23e_carriage_return_is_not_whitespace() {
    let doc = parse_checked("a\r\n\r\nCUT TO:\r\n\r\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::Transition, NodeKind::Action]
    );
    assert_eq!(
        tree("MARA\rhi"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\r" Dialogue(Text"hi")))"#
    );
}

/// OFF-24a
#[test]
fn off_24a_default_content() {
    let doc = parse_checked("MARA\nhi\n(beat)\nINT. X\nCUT TO:\nBOB\n\nb");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock, NodeKind::Action]);
    assert_eq!(find(&doc, NodeKind::Dialogue).len(), 4);
    assert_eq!(find(&doc, NodeKind::Parenthetical).len(), 1);
}

/// OFF-24b
#[test]
fn off_24b_forced_lines_end_the_block() {
    for (line, kind) in [
        (".HEAD", NodeKind::SceneHeading),
        ("!act", NodeKind::Action),
        (">CUT", NodeKind::Transition),
        (">MID<", NodeKind::Centered),
    ] {
        let doc = parse_checked(&format!("MARA\nhi\n{line}\nmore\nlines"));
        let kinds = top(&doc);
        assert_eq!(kinds[..2], [NodeKind::DialogueBlock, kind], "{line}");
        assert_eq!(find(&doc, NodeKind::Dialogue).len(), 1, "{line}");
    }
    // A forced cue ends the block and opens its own.
    let doc = parse_checked("MARA\nhi\n@BOB\nyo");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock; 2]);
    assert_eq!(
        dump(find(&doc, NodeKind::DialogueBlock)[1]),
        r#"DialogueBlock(CharacterCue(Sigil"@" CharacterName(Text"BOB")) LineBreak"\n" Dialogue(Text"yo"))"#
    );
    // A cue whose next line ends a block is not a cue.
    let doc = parse_checked("MARA\n!act");
    assert_eq!(top(&doc), [NodeKind::Action]);
    let doc = parse_checked("MARA\n.HEAD");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::SceneHeading]);
}

/// OFF-24c
#[test]
fn off_24c_interruptions_keep_the_block() {
    let doc = parse_checked("MARA\nhi\n~la\n# s\n= syn\n===\nyo");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock]);
    assert_eq!(find(&doc, NodeKind::Dialogue).len(), 2);
    assert_eq!(find(&doc, NodeKind::PageBreak).len(), 1);
}

/// OFF-24d
#[test]
fn off_24d_hidden_in_dialogue() {
    assert_eq!(
        tree("MARA\nhi\n[[n]]\nyo"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Dialogue(Text"hi") LineBreak"\n" Note(Marker"[[" Text"n" Marker"]]") LineBreak"\n" Dialogue(Text"yo")))"#
    );
    assert_eq!(
        tree("MARA\nhi /* x */ there"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Dialogue(Text"hi " Boneyard(Marker"/*" Text" x " Marker"*/") Text" there")))"#
    );
    // A blank line before the Note still ends the block.
    let doc = parse_checked("MARA\nhi\n\n[[n]]\nyo");
    assert_eq!(
        top(&doc),
        [NodeKind::DialogueBlock, NodeKind::Note, NodeKind::Action]
    );
    // A Note after the last line is top-level (decision 2).
    let doc = parse_checked("MARA\nhi\n/* b */");
    assert_eq!(top(&doc), [NodeKind::DialogueBlock, NodeKind::Boneyard]);
}

/// EXT-shot-3
#[test]
#[cfg(feature = "shot")]
fn ext_shot_3_ends_block() {
    let doc = parse_checked("MARA\nhi\n!!CLOSE ON\nmore");
    assert_eq!(
        top(&doc),
        [NodeKind::DialogueBlock, NodeKind::Shot, NodeKind::Action]
    );
    assert_eq!(find(&doc, NodeKind::Dialogue).len(), 1);
}

/// Hidden-node placement: a hidden span inside a cue name splits the name's Text.
#[test]
fn hidden_placement_inside_name() {
    assert_eq!(
        tree("MA/*x*/RA\nhi"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MA" Boneyard(Marker"/*" Text"x" Marker"*/") Text"RA")) LineBreak"\n" Dialogue(Text"hi")))"#
    );
}

/// OFF-6a
///
/// An unframed uppercase heading-by-prefix line is Action, never a cue, even with a blank line
/// before it and a text line under it.
#[test]
fn off_6a_unframed_uppercase_heading_is_action() {
    assert_eq!(
        tree("\nINT. X\nb"),
        r#"Document(Gap(LineBreak"\n") Action(Text"INT. X" LineBreak"\n" Text"b"))"#
    );
    let doc = parse_checked("a\n\nINT. X\n[[n]]\nmore");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::Action]);
    assert!(find(&doc, NodeKind::CharacterCue).is_empty());
}

/// OFF-23a
#[test]
fn off_23a_heading_ending_in_to_stays_heading() {
    assert_eq!(
        tree("a\n\nINT. WAY TO: \n\nb"),
        r#"Document(Action(Text"a") Gap(LineBreak"\n" LineBreak"\n") SceneHeading(Text"INT. WAY TO:" Whitespace" ") Gap(LineBreak"\n" LineBreak"\n") Action(Text"b"))"#
    );
}

/// A trailing Note on a cue line is a child of the CharacterCue, after the name.
#[test]
fn hidden_placement_trailing_note_in_cue() {
    assert_eq!(
        tree("MARA [[n]]\nhi"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA") Whitespace" " Note(Marker"[[" Text"n" Marker"]]")) LineBreak"\n" Dialogue(Text"hi")))"#
    );
}

/// A trailing Note on a dialogue line is a child of the Dialogue.
#[test]
fn hidden_placement_trailing_note_in_dialogue() {
    assert_eq!(
        tree("MARA\nhi [[m]]"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Dialogue(Text"hi" Whitespace" " Note(Marker"[[" Text"m" Marker"]]"))))"#
    );
}

/// A trailing Note on a parenthetical line is a child of the Parenthetical.
#[test]
fn hidden_placement_trailing_note_in_parenthetical() {
    assert_eq!(
        tree("MARA\n(soft) [[n]]"),
        r#"Document(DialogueBlock(CharacterCue(CharacterName(Text"MARA")) LineBreak"\n" Parenthetical(Text"(soft)" Whitespace" " Note(Marker"[[" Text"n" Marker"]]"))))"#
    );
}
