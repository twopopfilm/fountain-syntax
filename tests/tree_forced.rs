//! Syntax tree tests: forced elements.

mod common;

use common::{dump, find, parse_checked, top};
use fountain_syntax::NodeKind;

/// Parses `src` and returns the dump of its root.
fn tree(src: &str) -> String {
    dump(parse_checked(src).root())
}

/// EL-section-1
#[test]
fn el_section_1_depths() {
    let doc = parse_checked("# One\n## Two\n### Three\n#### Four");
    assert_eq!(top(&doc), [NodeKind::Section; 4]);
    assert_eq!(
        dump(doc.root()),
        r#####"Document(Section(Marker"#" Whitespace" " Text"One") Gap(LineBreak"\n") Section(Marker"##" Whitespace" " Text"Two") Gap(LineBreak"\n") Section(Marker"###" Whitespace" " Text"Three") Gap(LineBreak"\n") Section(Marker"####" Whitespace" " Text"Four"))"#####
    );
}

/// EXT-sections-1
#[test]
fn ext_sections_1_depth_1_to_8() {
    for depth in 1..=8 {
        let marker = "#".repeat(depth);
        assert_eq!(
            tree(&format!("{marker} T")),
            format!(r#"Document(Section(Marker"{marker}" Whitespace" " Text"T"))"#)
        );
    }
}

/// EL-page-break-1
#[test]
fn el_page_break_1() {
    let doc = parse_checked("===\n=====\n==========");
    assert_eq!(top(&doc), [NodeKind::PageBreak; 3]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(PageBreak(Marker"===") Gap(LineBreak"\n") PageBreak(Marker"=====") Gap(LineBreak"\n") PageBreak(Marker"=========="))"#
    );
}

/// EL-lyrics-1
#[test]
fn el_lyrics_1_run() {
    let doc = parse_checked("~one\n~two");
    assert_eq!(top(&doc), [NodeKind::Lyrics]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Lyrics(Sigil"~" Text"one" LineBreak"\n" Sigil"~" Text"two"))"#
    );
}

/// OFF-13b
#[test]
fn off_13b_lone_lyric() {
    let doc = parse_checked("a\n~la la\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::Lyrics, NodeKind::Action]
    );
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"a") Gap(LineBreak"\n") Lyrics(Sigil"~" Text"la la") Gap(LineBreak"\n") Action(Text"b"))"#
    );
}

/// OFF-13c
#[test]
fn off_13c_indented_lyric() {
    assert_eq!(
        tree("  ~ indented lyric"),
        r#"Document(Lyrics(Whitespace"  " Sigil"~" Text" indented lyric"))"#
    );
}

/// OFF-13d
#[test]
fn off_13d_runs_split_by_blank() {
    let doc = parse_checked("~a\n~b\n\n~c");
    assert_eq!(top(&doc), [NodeKind::Lyrics, NodeKind::Lyrics]);
    assert_eq!(find(&doc, NodeKind::Lyrics).len(), 2);
}

/// A Lyrics run continues across an invisible line, whose Note becomes a child of the run, and a
/// transparent line ends it (OFF-13d, OFF-21).
#[test]
fn lyrics_run_across_invisible_and_ended_by_transparent() {
    assert_eq!(
        tree("~a\n[[n]]\n~b"),
        r#"Document(Lyrics(Sigil"~" Text"a" LineBreak"\n" Note(Marker"[[" Text"n" Marker"]]") LineBreak"\n" Sigil"~" Text"b"))"#
    );
    let doc = parse_checked("~a\n# s\n~b");
    assert_eq!(
        top(&doc),
        [NodeKind::Lyrics, NodeKind::Section, NodeKind::Lyrics]
    );
    // A trailing invisible line is not part of the run.
    let doc = parse_checked("~a\n[[n]]");
    assert_eq!(top(&doc), [NodeKind::Lyrics, NodeKind::Note]);
}

/// EL-centered-1
#[test]
fn el_centered_1() {
    assert_eq!(
        tree(">THE END<"),
        r#"Document(Centered(Marker">" Text"THE END" Marker"<"))"#
    );
}

/// EL-centered-2
#[test]
fn el_centered_2() {
    assert_eq!(
        tree("  >  THE END  <  "),
        r#"Document(Centered(Whitespace"  " Marker">" Whitespace"  " Text"THE END" Whitespace"  " Marker"<" Whitespace"  "))"#
    );
}

/// EL-transition-2
#[test]
fn el_transition_2_forced() {
    let doc = parse_checked("a\n> fade out.\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::Transition, NodeKind::Action]
    );
    assert_eq!(
        find(&doc, NodeKind::Transition).len(),
        1,
        "one Transition node"
    );
    assert_eq!(
        tree("> fade out."),
        r#"Document(Transition(Sigil">" Text" fade out."))"#
    );
}

/// EL-action-3
#[test]
fn el_action_3_forced_and_joined() {
    assert_eq!(
        tree("!MARA\nnext line\n!ANOTHER"),
        r#"Document(Action(Sigil"!" Text"MARA" LineBreak"\n" Text"next line" LineBreak"\n" Sigil"!" Text"ANOTHER"))"#
    );
}

/// OFF-10a
#[test]
fn off_10a_centered_needs_both_marks() {
    assert_eq!(top(&parse_checked(">FIN<")), [NodeKind::Centered]);
    assert_eq!(top(&parse_checked(">FIN")), [NodeKind::Transition]);
    assert_eq!(top(&parse_checked("FIN<")), [NodeKind::Action]);
}

/// OFF-10b
#[test]
fn off_10b_trailing_whitespace_after_the_close() {
    assert_eq!(
        tree(">FIN< "),
        r#"Document(Centered(Marker">" Text"FIN" Marker"<" Whitespace" "))"#
    );
}

/// OFF-10c
#[test]
fn off_10c_nested_marks() {
    assert_eq!(
        tree(">> FIN <<"),
        r#"Document(Centered(Marker">" Text"> FIN <" Marker"<"))"#
    );
}

/// OFF-10d
#[test]
fn off_10d_not_centered() {
    assert_eq!(
        tree("> FIN"),
        r#"Document(Transition(Sigil">" Text" FIN"))"#
    );
    assert_eq!(
        tree(">FIN< and more"),
        r#"Document(Transition(Sigil">" Text"FIN< and more"))"#
    );
}

/// OFF-10e
#[test]
fn off_10e_marks_kept_as_tokens() {
    let doc = parse_checked(">FIN<");
    let centered = find(&doc, NodeKind::Centered)[0];
    assert_eq!(centered.text(), ">FIN<");
    assert_eq!(dump(centered), r#"Centered(Marker">" Text"FIN" Marker"<")"#);
    assert_eq!(tree("><"), r#"Document(Centered(Marker">" Marker"<"))"#);
}

/// OFF-19a
#[test]
fn off_19a_three_or_more_equals() {
    for run in ["===", "====", "==============="] {
        assert_eq!(tree(run), format!(r#"Document(PageBreak(Marker"{run}"))"#));
    }
}

/// OFF-19b
#[test]
fn off_19b_two_equals_and_equals_with_text_are_action() {
    assert_eq!(tree("=="), r#"Document(Action(Text"=="))"#);
    assert_eq!(tree("=== text"), r#"Document(Action(Text"=== text"))"#);
}

/// OFF-19c
#[test]
fn off_19c_synopses() {
    assert_eq!(tree("="), r#"Document(Synopsis(Marker"="))"#);
    assert_eq!(
        tree("= text"),
        r#"Document(Synopsis(Marker"=" Whitespace" " Text"text"))"#
    );
    assert_eq!(
        tree("= = ="),
        r#"Document(Synopsis(Marker"=" Whitespace" " Text"= ="))"#
    );
}

/// OFF-19d
#[test]
fn off_19d_whitespace_around_a_run() {
    assert_eq!(
        tree("  ==="),
        r#"Document(PageBreak(Whitespace"  " Marker"==="))"#
    );
    assert_eq!(
        tree("===  "),
        r#"Document(PageBreak(Marker"===" Whitespace"  "))"#
    );
}

/// OFF-19e
#[test]
fn off_19e_text_under_a_page_break_is_its_own_action() {
    let doc = parse_checked("===\nafter");
    assert_eq!(top(&doc), [NodeKind::PageBreak, NodeKind::Action]);
}

/// OFF-20a
#[test]
fn off_20a_no_space_needed() {
    assert_eq!(
        tree("#Section"),
        r###"Document(Section(Marker"#" Text"Section"))"###
    );
    assert_eq!(
        tree("#1 priority item"),
        r###"Document(Section(Marker"#" Text"1 priority item"))"###
    );
}

/// OFF-20b
#[test]
fn off_20b_empty_section() {
    assert_eq!(tree("#"), r###"Document(Section(Marker"#"))"###);
    assert_eq!(
        tree("# "),
        r###"Document(Section(Marker"#" Whitespace" "))"###
    );
}

/// OFF-20c
#[test]
fn off_20c_no_max_depth() {
    let marker = "#".repeat(30);
    assert_eq!(
        tree(&format!("{marker} deep")),
        format!(r#"Document(Section(Marker"{marker}" Whitespace" " Text"deep"))"#)
    );
}

/// OFF-20e
#[test]
fn off_20e_no_blank_lines_needed() {
    let doc = parse_checked("text\n# s");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::Section]);
    let doc = parse_checked("# s\ntext");
    assert_eq!(top(&doc), [NodeKind::Section, NodeKind::Action]);
    // Transparent for framing: a cue right after a Section still has the document start before it.
    let doc = parse_checked("# s\nMARA\nhi");
    assert_eq!(top(&doc), [NodeKind::Section, NodeKind::DialogueBlock]);
}

/// OFF-21b
#[test]
fn off_21b_outline_inside_action() {
    assert_eq!(
        tree("a\n# s\nb"),
        r###"Document(Action(Text"a" LineBreak"\n" Section(Marker"#" Whitespace" " Text"s") LineBreak"\n" Text"b"))"###
    );
    assert_eq!(
        tree("a\n= s\n===\nb"),
        r#"Document(Action(Text"a" LineBreak"\n" Synopsis(Marker"=" Whitespace" " Text"s") LineBreak"\n" PageBreak(Marker"===") LineBreak"\n" Text"b"))"#
    );
    assert_eq!(
        tree("a\n[[n]]\nb"),
        r#"Document(Action(Text"a" LineBreak"\n" Note(Marker"[[" Text"n" Marker"]]") LineBreak"\n" Text"b"))"#
    );
}

/// OFF-21b: an outline line at the end of a paragraph, or before it, is top-level.
#[test]
fn off_21b_trailing_outline_is_top_level() {
    let doc = parse_checked("a\n# s\n\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::Section, NodeKind::Action]
    );
    let doc = parse_checked("# s\na");
    assert_eq!(top(&doc), [NodeKind::Section, NodeKind::Action]);
    let doc = parse_checked("a\n[[n]]\n\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::Note, NodeKind::Action]
    );
}

/// OFF-22a
#[test]
fn off_22a_indented_sigil() {
    assert_eq!(
        tree("  !indented action"),
        r#"Document(Action(Whitespace"  " Sigil"!" Text"indented action"))"#
    );
}

/// OFF-22b
#[test]
fn off_22b_stacked_sigils() {
    assert_eq!(tree("!.dot"), r#"Document(Action(Sigil"!" Text".dot"))"#);
    assert_eq!(tree("!>x"), r#"Document(Action(Sigil"!" Text">x"))"#);
    assert_eq!(tree(">!x"), r#"Document(Transition(Sigil">" Text"!x"))"#);
    assert_eq!(tree("~!x"), r#"Document(Lyrics(Sigil"~" Text"!x"))"#);
}

/// OFF-22c
#[test]
fn off_22c_arrows() {
    assert_eq!(
        tree(">>> a line of arrows"),
        r#"Document(Transition(Sigil">" Text">> a line of arrows"))"#
    );
}

/// OFF-22d
#[test]
fn off_22d_sigils_kept_in_tree() {
    let doc = parse_checked("!a\n\n~b\n\n>c");
    assert_eq!(doc.source(), "!a\n\n~b\n\n>c");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Sigil"!" Text"a") Gap(LineBreak"\n" LineBreak"\n") Lyrics(Sigil"~" Text"b") Gap(LineBreak"\n" LineBreak"\n") Transition(Sigil">" Text"c"))"#
    );
}

/// OFF-22f
#[test]
fn off_22f_bare_sigils() {
    assert_eq!(tree("!"), r#"Document(Action(Sigil"!"))"#);
    assert_eq!(tree("!  "), r#"Document(Action(Sigil"!" Whitespace"  "))"#);
    assert_eq!(tree("~"), r#"Document(Lyrics(Sigil"~"))"#);
    assert_eq!(tree("~ "), r#"Document(Lyrics(Sigil"~" Whitespace" "))"#);
    assert_eq!(tree("@"), r#"Document(Action(Text"@"))"#);
    assert_eq!(tree("@  "), r#"Document(Action(Text"@" Whitespace"  "))"#);
}

/// OFF-22g
#[test]
#[cfg(not(feature = "shot"))]
fn off_22g_double_bang() {
    assert_eq!(tree("!!text"), r#"Document(Action(Sigil"!" Text"!text"))"#);
}

/// OFF-22g
#[test]
#[cfg(feature = "shot")]
fn off_22g_double_bang() {
    assert_eq!(tree("!!text"), r#"Document(Shot(Sigil"!!" Text"text"))"#);
}

/// Hidden-node placement: a hidden span after a single-line node's text, at its end, is the node's
/// last child.
#[test]
fn hidden_placement_at_the_end_of_a_node() {
    assert_eq!(
        tree("# s [[n]]"),
        r###"Document(Section(Marker"#" Whitespace" " Text"s" Whitespace" " Note(Marker"[[" Text"n" Marker"]]")))"###
    );
    assert_eq!(
        tree("=== [[n]]"),
        r#"Document(PageBreak(Marker"===" Whitespace" " Note(Marker"[[" Text"n" Marker"]]")))"#
    );
    assert_eq!(
        tree(">FIN<[[n]]"),
        r#"Document(Centered(Marker">" Text"FIN" Marker"<" Note(Marker"[[" Text"n" Marker"]]")))"#
    );
}

/// Hidden-node placement: a trailing Note on a shot line is a child of the Shot.
#[test]
#[cfg(feature = "shot")]
fn hidden_placement_trailing_note_in_shot() {
    assert_eq!(
        tree("!!shot [[n]]"),
        r#"Document(Shot(Sigil"!!" Text"shot" Whitespace" " Note(Marker"[[" Text"n" Marker"]]")))"#
    );
}

/// Hidden-node placement: a hidden span strictly inside a single-line node is its child.
#[test]
fn hidden_placement_inside_a_node() {
    assert_eq!(
        tree(">[[n]]FIN<"),
        r#"Document(Centered(Marker">" Note(Marker"[[" Text"n" Marker"]]") Text"FIN" Marker"<"))"#
    );
}

/// EXT-shot-1
#[test]
#[cfg(feature = "shot")]
fn ext_shot_1() {
    let doc = parse_checked("a\n!!CLOSE ON THE DOOR  \nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Action, NodeKind::Shot, NodeKind::Action]
    );
    assert_eq!(
        dump(find(&doc, NodeKind::Shot)[0]),
        r#"Shot(Sigil"!!" Text"CLOSE ON THE DOOR" Whitespace"  ")"#
    );
    assert_eq!(tree("!!"), r#"Document(Shot(Sigil"!!"))"#);
}

/// EXT-shot-3
#[test]
#[cfg(feature = "shot")]
fn ext_shot_3_triple_bang() {
    assert_eq!(tree("!!!text"), r#"Document(Shot(Sigil"!!" Text"!text"))"#);
}

/// EXT-shot-2
#[test]
#[cfg(not(feature = "shot"))]
fn ext_shot_2_without_flag() {
    let doc = parse_checked("!!text");
    assert_eq!(top(&doc), [NodeKind::Action]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Sigil"!" Text"!text"))"#
    );
}
