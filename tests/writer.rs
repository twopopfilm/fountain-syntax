//! The writer (SPEC 9): `write` and `Display` give the source back byte for byte.

mod common;

use common::parse_checked;

fn assert_round_trips(src: &str) {
    let doc = parse_checked(src);
    assert_eq!(doc.write(), src, "write changed {src:?}");
    assert_eq!(doc.to_string(), src, "Display changed {src:?}");
}

#[test]
fn wr_1_corpus_scripts_round_trip() {
    for src in [
        include_str!("corpus/twopop/blue_rev2.fountain"),
        include_str!("corpus/twopop/edge_cases.fountain"),
        include_str!("corpus/twopop/two_scene.fountain"),
    ] {
        assert_round_trips(src);
    }
}

#[test]
fn wr_2_line_endings_round_trip() {
    for src in [
        "a\r\nb\r\n",
        "a\rb",
        "a\r\nb\nc\r",
        "\u{FEFF}INT. A\n",
        "INT. A",
        "\n\n\n",
        "",
    ] {
        assert_round_trips(src);
    }
}

#[test]
fn wr_4_reference_writer_breakers_round_trip() {
    for src in [
        ".INT. X\n",
        ". SPACE AFTER\n",
        ">FIN<\n",
        "\n>CUT TO:\n\n",
        "\nMARA ^\nhi\n",
        "INT. X   #1#\n",
    ] {
        assert_round_trips(src);
    }
}
