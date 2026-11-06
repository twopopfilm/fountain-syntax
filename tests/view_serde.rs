//! `serde` support: a `Document` round-trips through its source text.
#![cfg(feature = "serde")]

mod common;

use common::dump;
use fountain_syntax::{Document, parse};

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
