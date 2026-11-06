//! Syntax tree tests: notes and boneyard.

mod common;

use common::{dump, find, parse_checked, top};
use fountain_syntax::{Child, NodeKind};

/// EL-note-1
#[test]
fn el_note_1_inline_and_own_line() {
    let doc = parse_checked("Mara [[aside]] waits.\n\n[[own line]]\n");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::Note]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"Mara " Note(Marker"[[" Text"aside" Marker"]]") Text" waits.") Gap(LineBreak"\n" LineBreak"\n") Note(Marker"[[" Text"own line" Marker"]]") Gap(LineBreak"\n"))"#
    );
}

/// EL-boneyard-1
#[test]
fn el_boneyard_1_hides_scene_in_tree() {
    let doc = parse_checked("/*\nINT. HOUSE - DAY\n\nMara waits.\n*/\n");
    assert_eq!(top(&doc), [NodeKind::Boneyard]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Boneyard(Marker"/*" LineBreak"\n" Text"INT. HOUSE - DAY" LineBreak"\n" LineBreak"\n" Text"Mara waits." LineBreak"\n" Marker"*/") Gap(LineBreak"\n"))"#
    );
}

/// OFF-15a
#[test]
fn off_15a_no_nested_notes() {
    let doc = parse_checked("[[outer [[inner]] outer]]");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Note(Marker"[[" Text"outer [[inner" Marker"]]") Whitespace" " Text"outer]]"))"#
    );
}

/// OFF-15b
#[test]
fn off_15b_unclosed_note_is_text() {
    let doc = parse_checked("[[a\n\nb]]");
    assert!(find(&doc, NodeKind::Note).is_empty());
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"[[a") Gap(LineBreak"\n" LineBreak"\n") Action(Text"b]]"))"#
    );
}

/// OFF-15c
#[test]
fn off_15c_stray_closer_is_text() {
    let doc = parse_checked("a ]] b\n]]");
    assert!(find(&doc, NodeKind::Note).is_empty());
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"a ]] b" LineBreak"\n" Text"]]"))"#
    );
}

/// OFF-15d
#[test]
fn off_15d_note_over_two_lines() {
    let doc = parse_checked("Mara [[a\nb]] waits.");
    assert_eq!(find(&doc, NodeKind::Note).len(), 1);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"Mara " Note(Marker"[[" Text"a" LineBreak"\n" Text"b" Marker"]]") Text" waits."))"#
    );
}

/// OFF-15e
#[test]
fn off_15e_own_line_note_inside_action() {
    let doc = parse_checked("a\n[[n]]\nb");
    assert_eq!(top(&doc), [NodeKind::Action]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"a" LineBreak"\n" Note(Marker"[[" Text"n" Marker"]]") LineBreak"\n" Text"b"))"#
    );
    // Decision 2: an invisible line at either end of a paragraph is a top-level node.
    let edges = parse_checked("[[n]]\na\n  [[m]]");
    assert_eq!(
        dump(edges.root()),
        r#"Document(Note(Marker"[[" Text"n" Marker"]]") Gap(LineBreak"\n") Action(Text"a") Gap(LineBreak"\n" Whitespace"  ") Note(Marker"[[" Text"m" Marker"]]"))"#
    );
}

/// OFF-15f
#[test]
fn off_15f_first_opener_wins() {
    let boneyard = parse_checked("/* [[ */ ]]");
    assert_eq!(
        dump(boneyard.root()),
        r#"Document(Action(Boneyard(Marker"/*" Text" [[ " Marker"*/") Whitespace" " Text"]]"))"#
    );
    let note = parse_checked("[[ /* ]] */");
    assert_eq!(
        dump(note.root()),
        r#"Document(Action(Note(Marker"[[" Text" /* " Marker"]]") Whitespace" " Text"*/"))"#
    );
}

/// OFF-15g
#[test]
fn off_15g_note_search_stops_at_blank() {
    for blank in [" ", "\t", "   "] {
        let src = format!("[[a\n{blank}\nb]]");
        let doc = parse_checked(&src);
        assert!(find(&doc, NodeKind::Note).is_empty(), "{src:?}");
        assert_eq!(top(&doc), [NodeKind::Action, NodeKind::Action], "{src:?}");
    }
    // A Boneyard is exempt: it crosses the same blank line.
    let doc = parse_checked("/*a\n \nb*/");
    assert_eq!(top(&doc), [NodeKind::Boneyard]);
}

/// OFF-1c
#[test]
fn off_1c_two_spaces_inside_note() {
    let doc = parse_checked("[[a\n  \nb]]");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Note(Marker"[[" Text"a" LineBreak"\n" Text"  " LineBreak"\n" Text"b" Marker"]]"))"#
    );
}

/// OFF-16a
#[test]
fn off_16a_inline_boneyard() {
    let doc = parse_checked("Mara /* aside */ waits.");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"Mara " Boneyard(Marker"/*" Text" aside " Marker"*/") Text" waits."))"#
    );
}

/// OFF-16a
#[test]
fn off_16a_inline_boneyard_across_lines() {
    let doc = parse_checked("Mara /* x\n\ny */ waits.");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"Mara " Boneyard(Marker"/*" Text" x" LineBreak"\n" LineBreak"\n" Text"y " Marker"*/") Text" waits."))"#
    );
}

/// OFF-16b
#[test]
fn off_16b_no_nested_boneyard() {
    let doc = parse_checked("/* outer /* inner */ still outer? */");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Boneyard(Marker"/*" Text" outer /* inner " Marker"*/") Whitespace" " Text"still outer? */"))"#
    );
}

/// OFF-16c
#[test]
fn off_16c_unclosed_boneyard_is_text() {
    let doc = parse_checked("a /* b\n\nc");
    assert!(find(&doc, NodeKind::Boneyard).is_empty());
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"a /* b") Gap(LineBreak"\n" LineBreak"\n") Action(Text"c"))"#
    );
}

/// OFF-16d
#[test]
fn off_16d_stray_closer_with_text() {
    let doc = parse_checked("text */\n*/");
    assert!(find(&doc, NodeKind::Boneyard).is_empty());
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"text */" LineBreak"\n" Text"*/"))"#
    );
}

/// EXT-revision-1
#[test]
fn ext_revision_1_beat_block_is_boneyard() {
    let block = "/* revision data\n{\"revisions\": {\"blue\": [[0, 4], [9, 2]]}}\nend of data */";
    let doc = parse_checked(&format!("Mara waits.\n\n{block}\n"));
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::Boneyard]);
    let boneyards = find(&doc, NodeKind::Boneyard);
    assert_eq!(boneyards.len(), 1);
    assert_eq!(boneyards[0].text(), block);
    assert_eq!(boneyards[0].parent(), Some(doc.root()));
    assert!(find(&doc, NodeKind::Note).is_empty());
}

/// Hidden-node placement: an invisible top-level line puts its hidden nodes at the top level and its
/// whitespace in Gaps.
#[test]
fn hidden_placement_top_level_invisible_line() {
    let doc = parse_checked("  [[a]] [[b]]");
    assert_eq!(top(&doc), [NodeKind::Note, NodeKind::Note]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Gap(Whitespace"  ") Note(Marker"[[" Text"a" Marker"]]") Gap(Whitespace" ") Note(Marker"[[" Text"b" Marker"]]"))"#
    );
    assert!(doc.root().children().all(|c| matches!(c, Child::Node(_))));
}

/// Hidden-node placement: a hidden span strictly inside a token splits it.
#[test]
fn hidden_placement_splits_token() {
    let doc = parse_checked("Ma[[n]]ra");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"Ma" Note(Marker"[[" Text"n" Marker"]]") Text"ra"))"#
    );
}
