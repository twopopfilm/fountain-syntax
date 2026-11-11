//! Tests for the targeted edit API (`SPEC.md` 10.2, issue #13).

mod common;

use common::{find, parse_checked};
use fountain_syntax::{Child, Document, EditError, NodeKind, Span, parse};

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
        // A node that ends where an insertion happens grows, so it is not held fixed.
        if old.end < span.start || (old.end == span.start && !span.is_empty()) {
            assert!(
                after_nodes.contains(&(kind, old)),
                "{kind:?} {old:?} moved or vanished"
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
