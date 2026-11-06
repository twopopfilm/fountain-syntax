mod common;

use common::{dump, find, parse_checked, top};
use fountain_syntax::NodeKind;

/// Parses `src` and returns the dump of its root.
fn tree(src: &str) -> String {
    dump(parse_checked(src).root())
}

/// The dump of the first `SceneNumber` in `src`, if it has one.
fn number(src: &str) -> Option<String> {
    let doc = parse_checked(src);
    find(&doc, NodeKind::SceneNumber).first().map(|n| dump(*n))
}

const NUM_12A: &str = r##"SceneNumber(Marker"#" Text"12A" Marker"#")"##;

/// EL-heading-1
#[test]
fn el_heading_1_each_prefix() {
    for prefix in ["INT.", "EXT.", "EST.", "INT./EXT.", "INT/EXT.", "I/E."] {
        for p in [prefix.to_string(), prefix.to_lowercase()] {
            assert_eq!(
                tree(&format!("{p} HOUSE - DAY")),
                format!(r#"Document(SceneHeading(Text"{p} HOUSE - DAY"))"#)
            );
        }
    }
}

/// EL-heading-2
#[test]
fn el_heading_2_forced() {
    assert_eq!(
        tree(".SNIPPET"),
        r#"Document(SceneHeading(Sigil"." Text"SNIPPET"))"#
    );
}

/// EL-heading-3
#[test]
fn el_heading_3_scene_number() {
    assert_eq!(
        tree("INT. HOUSE - DAY #12A#"),
        r###"Document(SceneHeading(Text"INT. HOUSE - DAY" Whitespace" " SceneNumber(Marker"#" Text"12A" Marker"#")))"###
    );
    assert_eq!(
        tree("INT. HOUSE #1#  "),
        r###"Document(SceneHeading(Text"INT. HOUSE" Whitespace" " SceneNumber(Marker"#" Text"1" Marker"#") Whitespace"  "))"###
    );
}

/// EL-heading-4
#[test]
fn el_heading_4_needs_blank_lines() {
    assert_eq!(
        tree("a\nINT. X\n\nb"),
        r#"Document(Action(Text"a" LineBreak"\n" Text"INT. X") Gap(LineBreak"\n" LineBreak"\n") Action(Text"b"))"#
    );
    // Lowercase, so the cue rule cannot take it (an uppercase one is a cue: tree_dialogue.rs).
    assert_eq!(
        tree("int. x\nb"),
        r#"Document(Action(Text"int. x" LineBreak"\n" Text"b"))"#
    );
    // Framed: the end of the document counts as a blank line.
    let doc = parse_checked("a\n\nINT. X");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::SceneHeading]);
}

/// EL-synopsis-1
#[test]
fn el_synopsis_1_after_heading() {
    let doc = parse_checked("INT. X\n\n= what happens");
    assert_eq!(top(&doc), [NodeKind::SceneHeading, NodeKind::Synopsis]);
    // The Synopsis is transparent, so the heading is framed with it directly below.
    let doc = parse_checked("INT. X\n= what happens");
    assert_eq!(top(&doc), [NodeKind::SceneHeading, NodeKind::Synopsis]);
}

/// OFF-6a
#[test]
fn off_6a_framing_skips_invisible_and_transparent_lines() {
    let doc = parse_checked("text\n\n[[note]]\nINT. X\n# section\n\nmore");
    assert_eq!(
        top(&doc),
        [
            NodeKind::Action,
            NodeKind::Note,
            NodeKind::SceneHeading,
            NodeKind::Section,
            NodeKind::Action
        ]
    );
    // A Note line above, with text above it: not framed.
    assert_eq!(
        tree("text\n[[n]]\nINT. X\n\n"),
        r#"Document(Action(Text"text" LineBreak"\n" Note(Marker"[[" Text"n" Marker"]]") LineBreak"\n" Text"INT. X") Gap(LineBreak"\n" LineBreak"\n"))"#
    );
    // A Note line below, with text below it: not framed.
    assert_eq!(
        tree("int. x\n[[n]]\nmore"),
        r#"Document(Action(Text"int. x" LineBreak"\n" Note(Marker"[[" Text"n" Marker"]]") LineBreak"\n" Text"more"))"#
    );
}

/// OFF-6b
#[test]
fn off_6b_period_or_space_only() {
    for src in ["INT HOUSE", "INT.HOUSE", "ext. house", "I/E HOUSE"] {
        assert_eq!(
            top(&parse_checked(src)),
            [NodeKind::SceneHeading],
            "{src:?}"
        );
    }
    for src in [
        "INT\tHOUSE",
        "INT-HOUSE",
        "INTERIOR HOUSE",
        "I/EXT. HOUSE",
        "INT",
    ] {
        assert_eq!(top(&parse_checked(src)), [NodeKind::Action], "{src:?}");
    }
}

/// OFF-6c
#[test]
fn off_6c_prose_with_a_prefix() {
    assert_eq!(
        tree("Ext. 12 is busy."),
        r#"Document(SceneHeading(Text"Ext. 12 is busy."))"#
    );
    assert_eq!(
        tree("!Ext. 12 is busy."),
        r#"Document(Action(Sigil"!" Text"Ext. 12 is busy."))"#
    );
}

/// OFF-6d
#[test]
fn off_6d_bare_prefix() {
    assert_eq!(tree("INT."), r#"Document(SceneHeading(Text"INT."))"#);
    assert_eq!(
        tree("INT "),
        r#"Document(SceneHeading(Text"INT" Whitespace" "))"#
    );
}

/// OFF-6e
#[test]
fn off_6e_document_edges_count_as_blank() {
    assert_eq!(top(&parse_checked("INT. X")), [NodeKind::SceneHeading]);
    assert_eq!(
        top(&parse_checked("\u{FEFF}INT. X\n\nmore")),
        [NodeKind::SceneHeading, NodeKind::Action]
    );
    assert_eq!(
        top(&parse_checked("a\n\nINT. X\n")),
        [NodeKind::Action, NodeKind::SceneHeading]
    );
}

/// OFF-21c
#[test]
fn off_21c_transparent_for_framing() {
    assert_eq!(
        tree("text\n# s\nINT. X\n\n"),
        r###"Document(Action(Text"text" LineBreak"\n" Section(Marker"#" Whitespace" " Text"s") LineBreak"\n" Text"INT. X") Gap(LineBreak"\n" LineBreak"\n"))"###
    );
}

/// OFF-7a
#[test]
fn off_7a_unicode_alphanumeric() {
    for text in ["LAMP ROOM", "5 HOURS LATER", "ÉPILOGUE"] {
        assert_eq!(
            tree(&format!(".{text}")),
            format!(r#"Document(SceneHeading(Sigil"." Text"{text}"))"#)
        );
    }
}

/// OFF-7b
#[test]
fn off_7b_period_then_space_or_symbol() {
    assert_eq!(
        tree(". SPACE AFTER"),
        r#"Document(Action(Text". SPACE AFTER"))"#
    );
    assert_eq!(tree(".>HEADING"), r#"Document(Action(Text".>HEADING"))"#);
}

/// OFF-7c
#[test]
fn off_7c_two_or_more_periods() {
    assert_eq!(tree("..DOUBLE"), r#"Document(Action(Text"..DOUBLE"))"#);
    assert_eq!(
        tree("...trailing off"),
        r#"Document(Action(Text"...trailing off"))"#
    );
}

/// OFF-7d
#[test]
fn off_7d_forced_needs_no_blank_lines() {
    let doc = parse_checked("a\n.LAMP ROOM\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::SceneHeading, NodeKind::Action]
    );
    assert_eq!(find(&doc, NodeKind::SceneHeading)[0].text(), ".LAMP ROOM");
}

/// OFF-8a
#[test]
fn off_8a_characters() {
    for n in ["12", "4B", "É1", "2-A.1"] {
        assert_eq!(
            number(&format!("INT. X #{n}#")).as_deref(),
            Some(format!(r##"SceneNumber(Marker"#" Text"{n}" Marker"#")"##).as_str()),
            "{n}"
        );
    }
    assert_eq!(number("INT. X ##"), None);
}

/// OFF-8b
#[test]
fn off_8b_space_or_underscore_inside() {
    for n in ["1 A", "1_A"] {
        let src = format!("INT. X #{n}#");
        assert_eq!(number(&src), None, "{n}");
        assert_eq!(
            tree(&src),
            format!(r##"Document(SceneHeading(Text"{src}"))"##)
        );
    }
}

/// OFF-8c
#[test]
fn off_8c_end_of_line_after_a_space() {
    assert_eq!(number("INT. X #7# trailing words"), None);
    assert_eq!(number("INT. X#1#"), None);
    assert!(number("INT. X #7#\t ").is_some());
    assert!(number("INT. X\t#7#").is_some());
}

/// OFF-8d
#[test]
fn off_8d_forced_heading() {
    assert_eq!(
        tree(".TITLE CARD #3#"),
        r###"Document(SceneHeading(Sigil"." Text"TITLE CARD" Whitespace" " SceneNumber(Marker"#" Text"3" Marker"#")))"###
    );
}

/// OFF-8e
#[test]
fn off_8e_several_pairs_and_section() {
    assert_eq!(
        tree("INT. X #1# #2#"),
        r###"Document(SceneHeading(Text"INT. X #1#" Whitespace" " SceneNumber(Marker"#" Text"2" Marker"#")))"###
    );
    assert_eq!(
        tree("# 12 #"),
        r###"Document(Section(Marker"#" Whitespace" " Text"12 #"))"###
    );
}

/// OFF-8f
#[test]
fn off_8f_number_is_apart_from_the_text() {
    let doc = parse_checked("INT. X #12A#");
    let heading = find(&doc, NodeKind::SceneHeading)[0];
    let numbers = find(&doc, NodeKind::SceneNumber);
    assert_eq!(numbers.len(), 1);
    assert_eq!(numbers[0].parent(), Some(heading));
    assert_eq!(dump(numbers[0]), NUM_12A);
    assert_eq!(numbers[0].text(), "#12A#");
}

/// OFF-8g
#[test]
#[cfg(not(feature = "locked-scene-numbers"))]
fn off_8g_locked_without_flag() {
    assert_eq!(
        tree("INT. X ##12A#"),
        r###"Document(SceneHeading(Text"INT. X ##12A#"))"###
    );
}

/// OFF-20d
#[test]
fn off_20d_section_vs_numbered_heading() {
    assert_eq!(
        tree("INT. HOUSE - DAY #5#"),
        r###"Document(SceneHeading(Text"INT. HOUSE - DAY" Whitespace" " SceneNumber(Marker"#" Text"5" Marker"#")))"###
    );
    let doc = parse_checked("# 12 #\n\n#5# INT. X");
    assert_eq!(top(&doc), [NodeKind::Section, NodeKind::Section]);
}

/// OFF-22e
#[test]
fn off_22e_dot_arrow() {
    let doc = parse_checked(".>HEADING");
    assert_eq!(top(&doc), [NodeKind::Action]);
    assert_eq!(dump(doc.root()), r#"Document(Action(Text".>HEADING"))"#);
}

/// OFF-16e
#[test]
fn off_16e_boneyard_then_heading() {
    assert_eq!(
        tree("/* x */INT. A"),
        r#"Document(Boneyard(Marker"/*" Text" x " Marker"*/") SceneHeading(Text"INT. A"))"#
    );
    // A whole-line Boneyard is invisible, so the lines around it are adjacent.
    let doc = parse_checked("text\n\n/* c */\nINT. A\n\nmore");
    assert_eq!(
        top(&doc),
        [
            NodeKind::Action,
            NodeKind::Boneyard,
            NodeKind::SceneHeading,
            NodeKind::Action
        ]
    );
    let doc = parse_checked("text\n/* c */\nINT. A\n\nmore");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::Action]);
}

/// OFF-18a
#[test]
fn off_18a_indented_heading_first_line() {
    assert_eq!(
        tree("\tINT. HOUSE - DAY"),
        r#"Document(SceneHeading(Whitespace"\t" Text"INT. HOUSE - DAY"))"#
    );
    assert_eq!(
        tree("a\n\n  INT. X"),
        r#"Document(Action(Text"a") Gap(LineBreak"\n" LineBreak"\n") SceneHeading(Whitespace"  " Text"INT. X"))"#
    );
}

/// Hidden-node placement: a hidden span at the edge of a SceneNumber goes beside it, inside the
/// heading; one strictly inside the number splits its token.
#[test]
fn hidden_placement_before_scene_number() {
    assert_eq!(
        tree("INT. X /*n*/#1#"),
        r###"Document(SceneHeading(Text"INT. X" Whitespace" " Boneyard(Marker"/*" Text"n" Marker"*/") SceneNumber(Marker"#" Text"1" Marker"#")))"###
    );
    assert_eq!(
        tree("INT. X #1/*n*/2#"),
        r###"Document(SceneHeading(Text"INT. X" Whitespace" " SceneNumber(Marker"#" Text"1" Boneyard(Marker"/*" Text"n" Marker"*/") Text"2" Marker"#")))"###
    );
}

/// Hidden-node placement: a hidden span inside a heading prefix splits the heading Text, and one
/// right after the forced sigil sits between the Sigil and the Text, inside the heading.
#[test]
fn hidden_placement_in_prefix_and_after_sigil() {
    assert_eq!(
        tree("IN/*x*/T. A"),
        r#"Document(SceneHeading(Text"IN" Boneyard(Marker"/*" Text"x" Marker"*/") Text"T. A"))"#
    );
    assert_eq!(
        tree("./*x*/LAMP"),
        r#"Document(SceneHeading(Sigil"." Boneyard(Marker"/*" Text"x" Marker"*/") Text"LAMP"))"#
    );
}

/// OFF-21b
#[test]
fn off_21b_outline_line_before_forced_heading_is_top_level() {
    // Decision 2: no Action member follows the Section, so it is top-level.
    assert_eq!(
        tree("a\n# s\n.HEAD"),
        r###"Document(Action(Text"a") Gap(LineBreak"\n") Section(Marker"#" Whitespace" " Text"s") Gap(LineBreak"\n") SceneHeading(Sigil"." Text"HEAD"))"###
    );
}

/// EXT-locked-scene-numbers-1
#[test]
#[cfg(feature = "locked-scene-numbers")]
fn ext_locked_scene_numbers_1() {
    assert_eq!(
        tree("INT. X ##12A#"),
        r###"Document(SceneHeading(Text"INT. X" Whitespace" " SceneNumber(Marker"##" Text"12A" Marker"#")))"###
    );
    assert_eq!(
        tree(".FORCED ##3#"),
        r###"Document(SceneHeading(Sigil"." Text"FORCED" Whitespace" " SceneNumber(Marker"##" Text"3" Marker"#")))"###
    );
    // An ordinary number still works.
    assert_eq!(number("INT. X #12A#").as_deref(), Some(NUM_12A));
}

/// EXT-locked-scene-numbers-2
#[test]
#[cfg(not(feature = "locked-scene-numbers"))]
fn ext_locked_scene_numbers_2() {
    assert_eq!(
        tree("INT. X ##12A#"),
        r###"Document(SceneHeading(Text"INT. X ##12A#"))"###
    );
}

/// EXT-locked-scene-numbers-3
#[test]
#[cfg(feature = "locked-scene-numbers")]
fn ext_locked_scene_numbers_3() {
    for src in [
        "INT. X ##",
        "INT. X ###x#",
        "INT. X##12A#",
        "INT. X #",
        "INT. X ## #",
    ] {
        assert_eq!(number(src), None, "{src:?}");
        assert_eq!(tree(src), format!(r#"Document(SceneHeading(Text"{src}"))"#));
    }
}
