mod common;

use common::{dump, find, parse_checked, top};
use fountain_syntax::NodeKind;

/// OFF-2a
#[test]
fn off_2a_line_endings() {
    let doc = parse_checked("a\nb\r\nc\rd");
    assert_eq!(top(&doc), [NodeKind::Action]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"a" LineBreak"\n" Text"b" LineBreak"\r\n" Text"c" LineBreak"\r" Text"d"))"#
    );
}

/// OFF-2b
#[test]
fn off_2b_final_newline() {
    let with = parse_checked("a\n");
    let without = parse_checked("a");
    assert_eq!(top(&with), [NodeKind::Action]);
    assert_eq!(top(&without), [NodeKind::Action]);
    assert_eq!(
        dump(with.root()),
        r#"Document(Action(Text"a") Gap(LineBreak"\n"))"#
    );
    assert_eq!(dump(without.root()), r#"Document(Action(Text"a"))"#);
}

/// OFF-2c
#[test]
fn off_2c_bom() {
    let lead = parse_checked("\u{FEFF}a");
    assert_eq!(
        dump(lead.root()),
        r#"Document(Bom"\u{feff}" Action(Text"a"))"#
    );
    let inside = parse_checked("a\u{FEFF}");
    assert_eq!(dump(inside.root()), r#"Document(Action(Text"a\u{feff}"))"#);
    // A source that is only a BOM has no Gap: the BOM is the root's only child.
    let only = parse_checked("\u{FEFF}");
    assert_eq!(dump(only.root()), r#"Document(Bom"\u{feff}")"#);
}

/// OFF-2d
#[test]
fn off_2d_other_separators() {
    let doc = parse_checked("a\u{2028}b\u{0085}c\x0Bd\x0Ce");
    assert_eq!(top(&doc), [NodeKind::Action]);
    assert_eq!(find(&doc, NodeKind::Action).len(), 1);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"a\u{2028}b\u{85}c\u{b}d\u{c}e"))"#
    );
}

/// OFF-1a
#[test]
fn off_1a_whitespace_only_line_is_blank() {
    let doc = parse_checked("a\n \t\nb");
    assert_eq!(top(&doc), [NodeKind::Action, NodeKind::Action]);
    assert_eq!(
        dump(find(&doc, NodeKind::Gap)[0]),
        r#"Gap(LineBreak"\n" Whitespace" \t" LineBreak"\n")"#
    );
}

/// OFF-1d
#[test]
fn off_1d_leading_blank_lines_kept() {
    let doc = parse_checked("\n\n  x");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Gap(LineBreak"\n" LineBreak"\n") Action(Whitespace"  " Text"x"))"#
    );
}

/// EL-action-1
#[test]
fn el_action_1_paragraph_keeps_lines() {
    let doc = parse_checked("one\n  two \nthree");
    assert_eq!(top(&doc), [NodeKind::Action]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"one" LineBreak"\n" Whitespace"  " Text"two" Whitespace" " LineBreak"\n" Text"three"))"#
    );
}

/// EL-linebreak-1
#[test]
fn el_linebreak_1_breaks_kept() {
    let doc = parse_checked("x\ny\r\nz\r");
    assert_eq!(
        dump(doc.root()),
        r#"Document(Action(Text"x" LineBreak"\n" Text"y" LineBreak"\r\n" Text"z") Gap(LineBreak"\r"))"#
    );
}

/// An empty source is one empty Gap; a blank-only source is Gap tokens.
#[test]
fn empty_document_is_one_gap() {
    let empty = parse_checked("");
    assert_eq!(dump(empty.root()), "Document(Gap())");
    let gap = find(&empty, NodeKind::Gap)[0];
    assert_eq!((gap.span().start, gap.span().end), (0, 0));
    let blank = parse_checked("\n\n");
    assert_eq!(
        dump(blank.root()),
        r#"Document(Gap(LineBreak"\n" LineBreak"\n"))"#
    );
}

/// Every string up to length 4 over a small alphabet satisfies the tree invariants (SPEC 8.3).
#[test]
fn tree_invariants_exhaustive() {
    const ALPHABET: [char; 28] = [
        '\n', '\r', ' ', '\t', 'a', 'A', 'é', '.', '!', '@', '[', ']', '/', '*', '~', '>', '<',
        '#', '=', '^', '(', ')', ':', 'I', 'N', 'T', 'O', '1',
    ];
    let mut layer = vec![String::new()];
    parse_checked("");
    for _ in 0..4 {
        layer = layer
            .iter()
            .flat_map(|s| ALPHABET.iter().map(move |c| format!("{s}{c}")))
            .collect();
        for s in &layer {
            parse_checked(s);
        }
    }
    // Hidden spans that fuse lines need five bytes or more, past the sweep.
    for s in [
        "/*\r\n*/",
        "[[\r\n]]",
        "a/*\r\n\r\n*/b",
        "[[a\r\n  \r\nb]]",
        "x[[a\rb]]y",
        "a\r[[n]]\r\nb",
        "[[\r\r]]",
        // Title pages: continuation lines, stray lines, hidden spans and CRLF breaks.
        "a:\r\n\tb\r\n  c\r\n   d: e\r\n\r\nINT. X",
        "[[n]]\n\nA:\n[[n]]\n\tv /* x\n */ w\n/* y */\n\nb",
        "a: b [[c\n\nd]]\n  \n\ta: b\n \n",
    ] {
        parse_checked(s);
    }
}
