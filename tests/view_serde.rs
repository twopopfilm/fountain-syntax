#![cfg(feature = "serde")]

mod common;

use common::dump;
use fountain_syntax::{DialogueItem, Document, Element, Run, Screenplay, parse};

const SOURCE: &str = "Title: Big Fish\n\nINT. HOUSE - DAY\n\nMARA\nHello.\n\n/* gone */\n";

#[test]
fn a_document_round_trips_through_json_with_the_same_source_and_tree() {
    let doc = parse(SOURCE);
    let json = serde_json::to_string(&doc).unwrap();
    assert_eq!(json, serde_json::json!({ "source": SOURCE }).to_string());
    let back: Document = serde_json::from_str(&json).unwrap();
    assert_eq!(back.source(), doc.source());
    assert_eq!(dump(back.root()), dump(doc.root()));
}

#[test]
fn an_empty_document_round_trips() {
    let doc = parse("");
    let back: Document = serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();
    assert_eq!(back.source(), "");
    assert_eq!(dump(back.root()), dump(doc.root()));
}

#[test]
fn malformed_json_is_an_error_not_a_broken_tree() {
    for bad in ["{", "null", "[]", r#"{"source": 5}"#, r#"{"nodes": []}"#] {
        assert!(
            serde_json::from_str::<Document>(bad).is_err(),
            "{bad:?} should not deserialize"
        );
    }
}

#[test]
fn spans_and_kinds_round_trip() {
    use fountain_syntax::{NodeKind, Span, TokenKind};
    let span = Span::new(3, 9);
    let json = serde_json::to_string(&span).unwrap();
    assert_eq!(serde_json::from_str::<Span>(&json).unwrap(), span);
    let json = serde_json::to_string(&NodeKind::SceneHeading).unwrap();
    assert_eq!(
        serde_json::from_str::<NodeKind>(&json).unwrap(),
        NodeKind::SceneHeading
    );
    let json = serde_json::to_string(&TokenKind::Colon).unwrap();
    assert_eq!(
        serde_json::from_str::<TokenKind>(&json).unwrap(),
        TokenKind::Colon
    );
}

/// Every element kind, in original text. The `!!` line is a Shot with the `shot` feature and a
/// forced Action without it; the scene number is locked with `locked-scene-numbers`.
fn script() -> String {
    let number = if cfg!(feature = "locked-scene-numbers") {
        "##2A#"
    } else {
        "#2A#"
    };
    format!(
        "Title: Big Fish\nAuthor: A. Writer\nNotes: first line\n    second line\n\n\
         Quiet at first, then **loud** and _soft_ and ***both***.\n[[a note]]\n\n\
         /* a boneyard */\n\n\
         # Act One\n\n= The set-up\n\n\
         INT. HOUSE - DAY #1#\n\n\
         He waits.\n# inside\n= inside too\n===\nThen he leaves. [[near]] /* far */\n\n\
         @ivan (CONT'D)\n(softly)\nhello \\*there\\*\n  \nagain\n~la la\n# in a block\nbye\n\n\
         MARA (V.O.)\nI see.\n\n\
         = Between the halves\n\n\
         OTTO ^\nSo do I.\n\n\
         NELL ^\nNo partner.\n\n\
         CUT TO:\n\n\
         > THE END <\n\n\
         ~sing\n~along\n\n\
         ===\n\n\
         .FORCED YARD {number}\n\n\
         !!CLOSE ON the door\n\n\
         !a forced action\n\n\
         >FADE OUT.\n"
    )
}

fn kinds(view: &Screenplay) -> Vec<&'static str> {
    let all = view
        .preamble
        .elements
        .iter()
        .chain(view.scenes.iter().flat_map(|s| s.elements.iter()));
    all.map(|e| match e {
        Element::Action(_) => "action",
        Element::Dialogue(_) => "dialogue",
        Element::DualDialogue(_) => "dual",
        Element::Transition(_) => "transition",
        Element::Lyrics(_) => "lyrics",
        Element::Section(_) => "section",
        Element::Synopsis(_) => "synopsis",
        Element::PageBreak(_) => "page break",
        Element::Note(_) => "note",
        Element::Boneyard(_) => "boneyard",
        #[cfg(feature = "shot")]
        Element::Shot(_) => "shot",
        _ => "other",
    })
    .collect()
}

#[test]
fn serde_round_trip_of_the_typed_view() {
    let view = parse(&script()).screenplay();

    // The script reaches every kind, so the round trip is not vacuous.
    assert!(view.title_page.is_some());
    assert_eq!(view.scenes.len(), 2);
    let kinds = kinds(&view);
    let wanted = [
        "action",
        "dialogue",
        "dual",
        "transition",
        "lyrics",
        "page break",
        "note",
        "boneyard",
    ]
    .into_iter()
    .chain(cfg!(feature = "shot").then_some("shot"));
    for kind in wanted {
        assert!(kinds.contains(&kind), "no {kind} in {kinds:?}");
    }
    // The Section and Synopsis before the first heading belong to scene 1.
    assert_eq!(view.scenes[0].sections.len(), 1);
    assert_eq!(view.scenes[0].synopses.len(), 1);
    assert_eq!(view.preamble.elements.len(), 3);
    let blocks: Vec<_> = view.scenes[0]
        .elements
        .iter()
        .filter_map(|e| match e {
            Element::Dialogue(b) => Some(b),
            _ => None,
        })
        .collect();
    assert!(blocks[0].cue.forced && blocks[0].cue.extensions[0].is_contd());
    assert!(
        blocks[0]
            .items
            .iter()
            .any(|i| matches!(i, DialogueItem::Interjection(_)))
    );
    assert!(blocks.iter().any(|b| b.unpaired_caret));
    let has_emphasis = view.preamble.elements.iter().any(|e| {
        matches!(e, Element::Action(a) if a.text.runs.iter().any(|r| matches!(r, Run::Emphasis { .. })))
    });
    assert!(has_emphasis);
    #[cfg(feature = "locked-scene-numbers")]
    assert!(view.scenes[1].number.as_ref().is_some_and(|n| n.locked));

    let json = serde_json::to_string(&view).unwrap();
    let back: Screenplay = serde_json::from_str(&json).unwrap();
    assert_eq!(back, view);
}
