mod common;

use common::{dump, find, parse_checked, top};
use fountain_syntax::NodeKind;

/// Parses `src` and returns the dump of its root.
fn tree(src: &str) -> String {
    dump(parse_checked(src).root())
}

/// Whether the first top-level node of `src` is a title page.
fn is_title_page(src: &str) -> bool {
    top(&parse_checked(src)).first() == Some(&NodeKind::TitlePage)
}

/// The text of every `TitleKey` in `src`, in order.
fn keys(src: &str) -> Vec<String> {
    let doc = parse_checked(src);
    find(&doc, NodeKind::TitleKey)
        .iter()
        .map(|n| n.text().to_string())
        .collect()
}

/// EL-title-page-1
#[test]
fn el_title_page_1_inline_values() {
    assert_eq!(
        tree("Title: Big Fish\nAuthor: John August"),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"Big Fish")) LineBreak"\n" TitleEntry(TitleKey(Text"Author") Colon":" Whitespace" " TitleValue(Text"John August"))))"#
    );
}

/// EL-title-page-2
#[test]
fn el_title_page_2_indented_values() {
    assert_eq!(
        tree("Title:\n\tBig Fish\n   The Movie\nAuthor: me"),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" LineBreak"\n" Whitespace"\t" TitleValue(Text"Big Fish") LineBreak"\n" Whitespace"   " TitleValue(Text"The Movie")) LineBreak"\n" TitleEntry(TitleKey(Text"Author") Colon":" Whitespace" " TitleValue(Text"me"))))"#
    );
    // An inline value and indented lines together; trailing whitespace is kept outside the value.
    assert_eq!(
        tree("Contact: a \n \t b  "),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Contact") Colon":" Whitespace" " TitleValue(Text"a") Whitespace" " LineBreak"\n" Whitespace" \t " TitleValue(Text"b") Whitespace"  ")))"#
    );
}

/// EL-title-page-4
#[test]
fn el_title_page_4_body_after_blank() {
    assert_eq!(
        tree("Title: x\n\nINT. HOUSE - DAY\n\nMara waits."),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x"))) Gap(LineBreak"\n" LineBreak"\n") SceneHeading(Text"INT. HOUSE - DAY") Gap(LineBreak"\n" LineBreak"\n") Action(Text"Mara waits."))"#
    );
    // A whitespace-only line ends the page too, and a two-space line is not special here.
    for src in ["Title: x\n   \nb", "Title: x\n  \nb"] {
        let doc = parse_checked(src);
        assert_eq!(
            top(&doc),
            [NodeKind::TitlePage, NodeKind::Action],
            "{src:?}"
        );
    }
}

/// OFF-6e
#[test]
fn off_6e_heading_after_title_page() {
    for src in ["Title: x\n\nINT. X\n\nb", "Title: x\n\nINT. X"] {
        let doc = parse_checked(src);
        assert_eq!(
            top(&doc)[..2],
            [NodeKind::TitlePage, NodeKind::SceneHeading],
            "{src:?}"
        );
    }
    // The page end counts as a blank line before, not after: the heading still needs one after.
    let doc = parse_checked("Title: x\n\nINT. X\nb");
    assert_eq!(top(&doc), [NodeKind::TitlePage, NodeKind::Action]);
    // A cue is framed by the page end the same way.
    let doc = parse_checked("Title: x\n\nMARA\nhi");
    assert_eq!(top(&doc), [NodeKind::TitlePage, NodeKind::DialogueBlock]);
}

/// OFF-17a
#[test]
fn off_17a_what_starts_a_title_page() {
    // Any key will do, even in the first block of ordinary prose.
    assert_eq!(
        tree("Note: this is a first line of action."),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Note") Colon":" Whitespace" " TitleValue(Text"this is a first line of action."))))"#
    );
    // A line that is a heading by prefix is never a key line.
    let doc = parse_checked("INT. HOUSE: DAY");
    assert_eq!(top(&doc), [NodeKind::SceneHeading]);
    let doc = parse_checked("INT. HOUSE: DAY\n\nb");
    assert_eq!(top(&doc), [NodeKind::SceneHeading, NodeKind::Action]);
    assert!(!is_title_page("int x: y"));
    // The first visible line decides: a key line later in the block does not start a page.
    assert!(!is_title_page("Hello there\nTitle: x"));
    // A bare key needs an indented line under it: a tab or three spaces, not none, one or two.
    assert!(!is_title_page("Title:\nBig Fish"));
    assert!(!is_title_page("Title:"));
    assert!(!is_title_page("FADE IN:\n\nINT. X"));
    assert!(!is_title_page("Title:\n x"));
    assert!(!is_title_page("Title:\n  x"));
    assert!(is_title_page("Title:\n   x"));
    assert!(is_title_page("Title:\n\tx"));
    // The indented line must be under the key, not after a blank line.
    assert!(!is_title_page("Title:\n\n\tx"));
    // Text that is not a key.
    for src in [
        "Mara, hi: x",
        "-: x",
        "'&: x",
        "# Title: x",
        "~ la: x",
        ".x: y",
        ": x",
    ] {
        assert!(!is_title_page(src), "{src:?}");
    }
    // A key keeps its inner spaces and punctuation of the allowed kinds, and may be indented.
    assert_eq!(keys("  Mr. O: x"), Vec::<String>::new());
    assert_eq!(keys("  Draft-date_1 & o'k: x"), ["Draft-date_1 & o'k"]);
}

/// OFF-17a
#[test]
fn off_17a_outline_lines_before_the_page() {
    // The page starts at the first visible non-blank line, so Sections, Synopses and Page Breaks
    // before the first key line are skipped like invisible lines, and stay top-level nodes.
    assert_eq!(
        tree("# Act\nTitle: Big\nAuthor: X\n\nINT. A\n\nhi"),
        r##"Document(Section(Marker"#" Whitespace" " Text"Act") Gap(LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"Big")) LineBreak"\n" TitleEntry(TitleKey(Text"Author") Colon":" Whitespace" " TitleValue(Text"X"))) Gap(LineBreak"\n" LineBreak"\n") SceneHeading(Text"INT. A") Gap(LineBreak"\n" LineBreak"\n") Action(Text"hi"))"##
    );
    assert_eq!(
        tree("= syn\nTitle: x"),
        r#"Document(Synopsis(Marker"=" Whitespace" " Text"syn") Gap(LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x"))))"#
    );
    assert_eq!(
        tree("===\nTitle: x"),
        r#"Document(PageBreak(Marker"===") Gap(LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x"))))"#
    );
    // Invisible and outline lines together keep their source order.
    assert_eq!(
        tree("[[n]]\n# Act\nTitle: x"),
        r##"Document(Note(Marker"[[" Text"n" Marker"]]") Gap(LineBreak"\n") Section(Marker"#" Whitespace" " Text"Act") Gap(LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x"))))"##
    );
    assert_eq!(
        tree("# Act\n[[n]]\nTitle: x"),
        r##"Document(Section(Marker"#" Whitespace" " Text"Act") Gap(LineBreak"\n") Note(Marker"[[" Text"n" Marker"]]") Gap(LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x"))))"##
    );
    assert_eq!(
        top(&parse_checked("# Act\n= s\n===\nTitle: x\n\nhi")),
        [
            NodeKind::Section,
            NodeKind::Synopsis,
            NodeKind::PageBreak,
            NodeKind::TitlePage,
            NodeKind::Action
        ]
    );
    // A Section, a blank line, then the key line: the blank line was always skipped.
    assert_eq!(
        tree("# Act\n\nTitle: x"),
        r##"Document(Section(Marker"#" Whitespace" " Text"Act") Gap(LineBreak"\n" LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x"))))"##
    );
    // A bare key still needs an indented line under it.
    assert!(is_title_page_after_outline("# Act\nTitle:\n\tx"));
    // `#Title: x` is a Section, not a key line: it is skipped and the next line is tested.
    assert_eq!(
        tree("#Title: x\nAuthor: y"),
        r##"Document(Section(Marker"#" Text"Title: x") Gap(LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"Author") Colon":" Whitespace" " TitleValue(Text"y"))))"##
    );
    // A first block of ordinary prose with a colon is a page, after a Section and a blank line too.
    assert_eq!(
        tree("# Act\n\nplain text: with colon"),
        r##"Document(Section(Marker"#" Whitespace" " Text"Act") Gap(LineBreak"\n" LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"plain text") Colon":" Whitespace" " TitleValue(Text"with colon"))))"##
    );
}

/// Whether any top-level node of `src` is a title page.
fn is_title_page_after_outline(src: &str) -> bool {
    top(&parse_checked(src)).contains(&NodeKind::TitlePage)
}

/// OFF-17a
#[test]
fn off_17a_outline_before_a_non_page() {
    // A heading-prefix line is never a key line, whatever precedes it.
    assert_eq!(
        tree("# Act\nINT. HOUSE: DAY\n\nx"),
        r##"Document(Section(Marker"#" Whitespace" " Text"Act") Gap(LineBreak"\n") SceneHeading(Text"INT. HOUSE: DAY") Gap(LineBreak"\n" LineBreak"\n") Action(Text"x"))"##
    );
    // A bare key with no indented value under it.
    assert_eq!(
        tree("# Act\nTitle:\nplain\n\nx"),
        r##"Document(Section(Marker"#" Whitespace" " Text"Act") Gap(LineBreak"\n") Action(Text"Title:" LineBreak"\n" Text"plain") Gap(LineBreak"\n" LineBreak"\n") Action(Text"x"))"##
    );
    // An outline line after the first key line is still a stray line inside the page.
    assert_eq!(
        tree("Title: x\n# Act\nAuthor: y"),
        r##"Document(TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x")) LineBreak"\n" Text"# Act" LineBreak"\n" TitleEntry(TitleKey(Text"Author") Colon":" Whitespace" " TitleValue(Text"y"))))"##
    );
    // The first visible non-blank line decides: ordinary text before the key line blocks the page.
    assert!(!is_title_page_after_outline("# Act\nHello there\nTitle: x"));
}

/// OFF-6e
#[test]
fn off_6e_heading_after_page_with_leading_outline() {
    let doc = parse_checked("# Act\nTitle: x\n\nINT. A\n\nb");
    assert_eq!(
        top(&doc),
        [
            NodeKind::Section,
            NodeKind::TitlePage,
            NodeKind::SceneHeading,
            NodeKind::Action
        ]
    );
    // The page end still counts as the blank line before the heading, which needs one after it.
    let doc = parse_checked("# Act\nTitle: x\n\nINT. A\nb");
    assert_eq!(
        top(&doc),
        [NodeKind::Section, NodeKind::TitlePage, NodeKind::Action]
    );
}

/// OFF-17b
#[test]
fn off_17b_ends_at_the_first_blank_line() {
    let doc = parse_checked("Title: a\n\nAuthor: b");
    assert_eq!(top(&doc), [NodeKind::TitlePage, NodeKind::Action]);
    assert_eq!(find(&doc, NodeKind::TitleEntry).len(), 1);
    // The page runs to the end of the document when there is no blank line.
    let doc = parse_checked("Title: a\nAuthor: b\n");
    assert_eq!(top(&doc), [NodeKind::TitlePage]);
    assert_eq!(find(&doc, NodeKind::TitleEntry).len(), 2);
}

/// OFF-17c
#[test]
fn off_17c_keys_as_written() {
    assert_eq!(
        keys("title: a\nTitle: b\nTITLE: c\nauthors: d"),
        ["title", "Title", "TITLE", "authors"]
    );
}

/// OFF-17d
#[test]
fn off_17d_stray_lines() {
    // A one- or two-space line that is not a key line sits in the page, attached to no entry.
    assert_eq!(
        tree("Title: x\n Stray\nAuthor: y"),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x")) LineBreak"\n" Whitespace" " Text"Stray" LineBreak"\n" TitleEntry(TitleKey(Text"Author") Colon":" Whitespace" " TitleValue(Text"y"))))"#
    );
    assert_eq!(
        find(&parse_checked("Title: x\n  S"), NodeKind::TitleValue).len(),
        1
    );
    // An indented line that is a key line is a key line.
    assert_eq!(keys("Title: x\n  Author: y"), ["Title", "Author"]);
    // A stray line ends the current entry, so a tab-indented line after it is judged alone:
    // a stray line again without a colon, a key line with one.
    let doc = parse_checked("Title: x\n  S\n\tcont");
    assert_eq!(find(&doc, NodeKind::TitleValue).len(), 1);
    assert_eq!(keys("Title: x\n  S\n\tK: v"), ["Title", "K"]);
    // With a current entry, a deeper-indented line is a continuation even when it holds a colon.
    let doc = parse_checked("Title: x\n\tSub: title");
    assert_eq!(find(&doc, NodeKind::TitleEntry).len(), 1);
    assert_eq!(find(&doc, NodeKind::TitleValue).len(), 2);
    assert_eq!(keys("Title: x\n   Sub: title"), ["Title"]);
    // Without one, the key-line test applies: a three-space key line starts an entry.
    assert_eq!(keys("A: x\n  stray\n   B: y"), ["A", "B"]);
}

/// OFF-17e
#[test]
fn off_17e_values_and_colons() {
    assert_eq!(
        tree("Draft date: 5:30 pm"),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Draft date") Colon":" Whitespace" " TitleValue(Text"5:30 pm"))))"#
    );
    // The key ends at the first colon, and space before it is not part of the key.
    assert_eq!(
        tree("Title : a:b"),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Title") Whitespace" " Colon":" Whitespace" " TitleValue(Text"a:b"))))"#
    );
    // An empty value is allowed after the first line; it has no TitleValue node.
    assert_eq!(
        tree("Author: x\nTitle:\nSource:  \nDate: y"),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Author") Colon":" Whitespace" " TitleValue(Text"x")) LineBreak"\n" TitleEntry(TitleKey(Text"Title") Colon":") LineBreak"\n" TitleEntry(TitleKey(Text"Source") Colon":" Whitespace"  ") LineBreak"\n" TitleEntry(TitleKey(Text"Date") Colon":" Whitespace" " TitleValue(Text"y"))))"#
    );
    // A bare key with an empty value, then a key line, is fine in the middle but not at the start.
    assert!(!is_title_page("Title:\nAuthor: x"));
}

/// OFF-17f
#[test]
fn off_17f_leading_blank_lines() {
    let doc = parse_checked("\n \n\t\nTitle: x\n\nb");
    assert_eq!(top(&doc), [NodeKind::TitlePage, NodeKind::Action]);
    assert_eq!(
        dump(doc.root()),
        r#"Document(Gap(LineBreak"\n" Whitespace" " LineBreak"\n" Whitespace"\t" LineBreak"\n") TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x"))) Gap(LineBreak"\n" LineBreak"\n") Action(Text"b"))"#
    );
    // Invisible lines before the first key line are top-level nodes before the page.
    let doc = parse_checked("[[n]]\n\n/* b */\nTitle: x\n\nb");
    assert_eq!(
        top(&doc),
        [
            NodeKind::Note,
            NodeKind::Boneyard,
            NodeKind::TitlePage,
            NodeKind::Action
        ]
    );
    // An indented first key line, and a BOM before it.
    assert!(is_title_page("\tTitle: x"));
    assert!(is_title_page("\u{FEFF}Title: x"));
}

/// OFF-15f
#[test]
fn off_15f_note_in_title_page() {
    // A Note on its own line does not end the page; it sits in the page between entries.
    assert_eq!(
        tree("Title: x\n[[n]]\nAuthor: y"),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" Whitespace" " TitleValue(Text"x")) LineBreak"\n" Note(Marker"[[" Text"n" Marker"]]") LineBreak"\n" TitleEntry(TitleKey(Text"Author") Colon":" Whitespace" " TitleValue(Text"y"))))"#
    );
    // Before a continuation line it is inside the entry, which the bare key still counts as indented.
    assert_eq!(
        tree("Title:\n[[n]]\n\tx"),
        r#"Document(TitlePage(TitleEntry(TitleKey(Text"Title") Colon":" LineBreak"\n" Note(Marker"[[" Text"n" Marker"]]") LineBreak"\n" Whitespace"\t" TitleValue(Text"x"))))"#
    );
    // After the last entry it is still in the page, which ends at the blank line.
    let doc = parse_checked("Title: x\n/* b */\n\nb");
    assert_eq!(top(&doc), [NodeKind::TitlePage, NodeKind::Action]);
    assert_eq!(find(&doc, NodeKind::Boneyard).len(), 1);
    // A Note inside a key line splits the text; the visible text is what is read.
    let doc = parse_checked("Ti[[n]]tle: x");
    assert_eq!(top(&doc), [NodeKind::TitlePage]);
    // A Note that fuses two lines makes one key line.
    let doc = parse_checked("Title: a [[n\nm]] b\nAuthor: c");
    assert_eq!(find(&doc, NodeKind::TitleEntry).len(), 2);
}

/// EXT-open-keys-1
#[test]
fn ext_open_keys_1_kept_in_order() {
    assert_eq!(
        keys("Title: a\nSome Unknown Key: b\ntitle: c\nTITLE: d\nTitle: e\nBC: f"),
        ["Title", "Some Unknown Key", "title", "TITLE", "Title", "BC"]
    );
}
