//! Tests for the targeted edit API (`SPEC.md` 10.2, issue #13).

mod common;

use common::{find, parse_checked};
use fountain_syntax::{Child, Document, EditError, Element, NodeKind, Span, parse};

/// The top-level nodes (Gaps included) as `(kind, span)`.
fn top_level(doc: &Document) -> Vec<(NodeKind, Span)> {
    doc.root()
        .children()
        .filter_map(|c| match c {
            Child::Node(n) => Some((n.kind(), n.span())),
            Child::Token(_) => None,
        })
        .collect()
}

/// Asserts that `after` is `before` with only `span` replaced by `text`, and that the nodes
/// outside the span kept their kinds and moved only by the length difference.
fn assert_only_span_changed(before: &str, after: &Document, span: Span, text: &str) {
    let expected = format!("{}{}{}", &before[..span.start], text, &before[span.end..]);
    assert_eq!(after.source(), expected);
    parse_checked(after.source());
    let after_nodes = top_level(after);
    for (kind, old) in top_level(&parse(before)) {
        if old.end < span.start || (old.end == span.start && !span.is_empty()) {
            assert!(
                after_nodes.contains(&(kind, old)),
                "{kind:?} {old:?} moved or vanished"
            );
        } else if old.end == span.start {
            // A node that ends where text is inserted stays put or grows by the insertion.
            let grown = Span::new(old.start, old.end + text.len());
            assert!(
                after_nodes.contains(&(kind, old)) || after_nodes.contains(&(kind, grown)),
                "{kind:?} {old:?} neither kept nor grown to {grown:?}"
            );
        } else if old.start >= span.end {
            let moved = Span::new(
                old.start + text.len() - span.len(),
                old.end + text.len() - span.len(),
            );
            assert!(
                after_nodes.contains(&(kind, moved)),
                "{kind:?} {old:?} not shifted to {moved:?}"
            );
        }
    }
}

const SCRIPT: &str = "INT. A #1#\n\nHello.\n\nINT. B\n";

#[test]
fn wr_7_with_replaced_changes_only_the_span() {
    let doc = parse_checked(SCRIPT);
    let number = find(&doc, NodeKind::SceneNumber)[0].span();

    let longer = doc.with_replaced(number, "#12A#");
    assert_eq!(longer.source(), "INT. A #12A#\n\nHello.\n\nINT. B\n");
    assert_only_span_changed(SCRIPT, &longer, number, "#12A#");

    let shorter = doc.with_replaced(number, "");
    assert_only_span_changed(SCRIPT, &shorter, number, "");

    // An empty span inserts. It sits inside the heading's Text token, not on a boundary
    // between two top-level nodes, which has no single owner.
    let at = Span::new(5, 5);
    let inserted = doc.with_replaced(at, "B");
    assert_eq!(inserted.source(), "INT. BA #1#\n\nHello.\n\nINT. B\n");
    assert_only_span_changed(SCRIPT, &inserted, at, "B");
}

#[test]
fn wr_7_identity_replacement_on_the_corpus() {
    for src in [
        include_str!("corpus/twopop/blue_rev2.fountain"),
        include_str!("corpus/twopop/edge_cases.fountain"),
        include_str!("corpus/twopop/two_scene.fountain"),
    ] {
        let doc = parse_checked(src);
        for child in doc.root().children() {
            let Child::Node(node) = child else { continue };
            if node.kind() == NodeKind::Gap {
                continue;
            }
            assert_eq!(
                doc.with_replaced(node.span(), node.text()).source(),
                src,
                "{:?} at {:?}",
                node.kind(),
                node.span()
            );
        }
    }
}

#[test]
#[should_panic]
fn wr_7_bad_span_panics() {
    parse("abc").with_replaced(Span::new(2, 9), "x");
}

fn heading_span(doc: &Document, scene: usize) -> Span {
    doc.screenplay().scenes[scene].heading.span.unwrap()
}

#[test]
fn wr_9_sets_a_number_on_a_heading_without_one() {
    let src = "INT. A\n\nHi.\n";
    let doc = parse_checked(src);
    let edited = doc.with_scene_number(heading_span(&doc, 0), "12A").unwrap();
    assert_eq!(edited.source(), "INT. A #12A#\n\nHi.\n");
    assert_only_span_changed(src, &edited, Span::new(6, 6), " #12A#");
    assert_eq!(
        edited.screenplay().scenes[0].number.clone().unwrap().text,
        "12A"
    );
}

#[test]
fn wr_9_inserts_before_trailing_whitespace() {
    let doc = parse_checked("INT. A  \n");
    let edited = doc.with_scene_number(heading_span(&doc, 0), "7").unwrap();
    assert_eq!(edited.source(), "INT. A #7#  \n");
}

#[test]
fn wr_9_replaces_an_existing_number_only() {
    for (src, number, expected) in [
        ("INT. A  #1#\n", "2", "INT. A  #2#\n"),
        (".ROOF #1#\n", "2", ".ROOF #2#\n"),
        ("\u{FEFF}INT. A\n", "2", "\u{FEFF}INT. A #2#\n"),
    ] {
        let doc = parse_checked(src);
        let edited = doc
            .with_scene_number(heading_span(&doc, 0), number)
            .unwrap();
        assert_eq!(edited.source(), expected);
        assert_eq!(
            edited.screenplay().scenes[0].number.clone().unwrap().text,
            number
        );
    }
}

#[cfg(feature = "locked-scene-numbers")]
#[test]
fn wr_9_locked_number_is_replaced_by_a_plain_one() {
    let doc = parse_checked("INT. A ##1#\n");
    let edited = doc.with_scene_number(heading_span(&doc, 0), "2").unwrap();
    assert_eq!(edited.source(), "INT. A #2#\n");
    assert!(!edited.screenplay().scenes[0].number.clone().unwrap().locked);
}

#[cfg(feature = "locked-scene-numbers")]
#[test]
fn wr_9_with_locked_scene_number_writes_the_locked_form() {
    let doc = parse_checked("INT. A\n");
    let edited = doc
        .with_locked_scene_number(heading_span(&doc, 0), "3")
        .unwrap();
    assert_eq!(edited.source(), "INT. A ##3#\n");
    assert!(edited.screenplay().scenes[0].number.clone().unwrap().locked);
}

#[test]
fn wr_13_scene_number_refusals() {
    let doc = parse_checked("INT. A\n\nHi.\n");
    let action = find(&doc, NodeKind::Action)[0].span();
    for span in [action, Span::new(0, 1)] {
        assert_eq!(
            doc.with_scene_number(span, "1").err(),
            Some(EditError::NotFound)
        );
    }
    let heading = heading_span(&doc, 0);
    for number in ["1 2", "a#b", ""] {
        assert_eq!(
            doc.with_scene_number(heading, number).err(),
            Some(EditError::Invalid),
            "{number:?}"
        );
    }
    assert_eq!(doc.source(), "INT. A\n\nHi.\n");
}

#[test]
fn wr_10_inserts_a_synopsis_after_the_heading() {
    let src = "INT. A\n\nHi.\n";
    let doc = parse_checked(src);
    let edited = doc
        .with_synopsis(heading_span(&doc, 0), "Mara arrives.")
        .unwrap();
    assert_eq!(edited.source(), "INT. A\n= Mara arrives.\n\nHi.\n");
    assert_only_span_changed(src, &edited, Span::new(6, 6), "\n= Mara arrives.");
    assert_eq!(
        edited.screenplay().scenes[0].synopses[0].text.plain(),
        "Mara arrives."
    );

    for (src, expected) in [
        ("INT. A\r\n\r\nHi.\r\n", "INT. A\r\n= x\r\n\r\nHi.\r\n"),
        ("INT. A\r\rHi.\r", "INT. A\r= x\r\rHi.\r"),
        ("INT. A", "INT. A\n= x"),
    ] {
        let doc = parse_checked(src);
        let edited = doc.with_synopsis(heading_span(&doc, 0), "x").unwrap();
        assert_eq!(edited.source(), expected);
        assert_eq!(edited.screenplay().scenes[0].synopses[0].text.plain(), "x");
    }
}

#[test]
fn wr_10_replaces_the_synopsis_directly_after_the_heading() {
    let src = "INT. A\n= old\n\nHi.\n";
    let doc = parse_checked(src);
    let edited = doc.with_synopsis(heading_span(&doc, 0), "new").unwrap();
    assert_eq!(edited.source(), "INT. A\n= new\n\nHi.\n");
    assert_only_span_changed(src, &edited, Span::new(7, 12), "= new");

    // Only the first Synopsis line directly after the heading is replaced.
    let doc = parse_checked("INT. A\n= one\n= two\n\nHi.\n");
    let edited = doc.with_synopsis(heading_span(&doc, 0), "new").unwrap();
    assert_eq!(edited.source(), "INT. A\n= new\n= two\n\nHi.\n");

    // A Synopsis after a blank line is not directly after the heading: insert instead.
    let doc = parse_checked("INT. A\n\n= old\n\nHi.\n");
    let edited = doc.with_synopsis(heading_span(&doc, 0), "new").unwrap();
    assert_eq!(edited.source(), "INT. A\n= new\n\n= old\n\nHi.\n");

    // Leading Synopses before the heading are never touched.
    let doc = parse_checked("= lead\n\nINT. A\n\nHi.\n");
    let edited = doc.with_synopsis(heading_span(&doc, 0), "new").unwrap();
    assert_eq!(edited.source(), "= lead\n\nINT. A\n= new\n\nHi.\n");
}

#[test]
fn wr_11_adds_a_note_block_after_an_element() {
    let src = "INT. A\n\nHi.\n\nBYE.\n";
    let doc = parse_checked(src);
    let hi = find(&doc, NodeKind::Action)[0].span();
    let edited = doc.with_note(hi, "check this").unwrap();
    assert_eq!(edited.source(), "INT. A\n\nHi.\n\n[[check this]]\n\nBYE.\n");
    assert_only_span_changed(src, &edited, Span::new(11, 11), "\n\n[[check this]]");
    let view = edited.screenplay();
    let Element::Note(note) = &view.scenes[0].elements[1] else {
        panic!("{:?}", view.scenes[0].elements)
    };
    assert_eq!(note.text, "check this");

    for (src, expected, at_heading) in [
        ("INT. A\n\nHi.", "INT. A\n\nHi.\n\n[[n]]", false),
        (
            "INT. A\r\n\r\nHi.\r\n",
            "INT. A\r\n\r\nHi.\r\n\r\n[[n]]\r\n",
            false,
        ),
        // One line break before the next element: the note keeps a blank line after it.
        (
            "INT. A\n= x\n\nHi.\n",
            "INT. A\n\n[[n]]\n\n= x\n\nHi.\n",
            true,
        ),
    ] {
        let doc = parse_checked(src);
        let target = if at_heading {
            heading_span(&doc, 0)
        } else {
            find(&doc, NodeKind::Action)[0].span()
        };
        let edited = doc.with_note(target, "n").unwrap();
        assert_eq!(edited.source(), expected);
        parse_checked(edited.source());
    }
}

#[test]
fn wr_13_synopsis_and_note_refusals() {
    let doc = parse_checked("INT. A\n\nMARA\nHi.\n");
    let block = find(&doc, NodeKind::DialogueBlock)[0].span();
    let cue = find(&doc, NodeKind::CharacterCue)[0].span();
    let gap = find(&doc, NodeKind::Gap)[0].span();
    let heading = heading_span(&doc, 0);

    for span in [block, cue, Span::new(0, 1)] {
        assert_eq!(
            doc.with_synopsis(span, "x").err(),
            Some(EditError::NotFound)
        );
    }
    for span in [cue, gap, Span::new(0, 1), Span::new(0, doc.source().len())] {
        assert_eq!(doc.with_note(span, "x").err(), Some(EditError::NotFound));
    }

    for text in ["a\nb", "a\rb", "a\r\nb"] {
        assert_eq!(
            doc.with_synopsis(heading, text).err(),
            Some(EditError::Invalid),
            "{text:?}"
        );
    }
    for text in ["a]]b", "a\n\nb", "a\r\n\r\nb", "a\r\rb"] {
        assert_eq!(
            doc.with_note(heading, text).err(),
            Some(EditError::Invalid),
            "{text:?}"
        );
    }
    assert_eq!(doc.source(), "INT. A\n\nMARA\nHi.\n");
}
