mod common;

use common::parse_checked;
use fountain_syntax::{DiagnosticCode, Element, parse};

const FRAGMENTS: &[&str] = &[
    "\n", "\r\n", "\r", " ", "  ", "\t", "\u{FEFF}", "INT. ", "EXT ", ".", "!", "!!", "@", "~",
    ">", "<", "#", "=", "==", "===", "^", "(", ")", "[[", "]]", "/*", "*/", "*", "**", "_", "\\",
    ":", "TO:", "CUT TO:", "FADE IN:", "Title: ", "MARA", "Mara", "李明", "é", "23", "#1#", "x",
    " text ",
];

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn never_fail_random_inputs() {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut lone_seen = 0;
    for _ in 0..5_000 {
        let count = xorshift(&mut state) % 49;
        let src: String = (0..count)
            .map(|_| FRAGMENTS[(xorshift(&mut state) % FRAGMENTS.len() as u64) as usize])
            .collect();
        let doc = parse_checked(&src);
        let play = doc.screenplay();
        let diags = doc.diagnostics();
        assert_eq!(diags, parse(&src).diagnostics(), "unstable for {src:?}");
        let lone = diags
            .iter()
            .filter(|d| d.code == DiagnosticCode::LoneCaret)
            .count();
        let unpaired = play
            .preamble
            .elements
            .iter()
            .chain(play.scenes.iter().flat_map(|s| &s.elements))
            .filter(|e| matches!(e, Element::Dialogue(b) if b.unpaired_caret))
            .count();
        assert_eq!(lone, unpaired, "lone-caret count for {src:?}");
        lone_seen += lone;
        let mut last = (0, 0, 0);
        for d in diags {
            let key = (d.span.start, d.span.end, d.code as usize);
            assert!(last <= key, "unsorted for {src:?}");
            last = key;
            assert!(
                d.span.start <= d.span.end
                    && d.span.end <= src.len()
                    && src.is_char_boundary(d.span.start)
                    && src.is_char_boundary(d.span.end),
                "bad span {:?} for {src:?}",
                d.span
            );
            assert_eq!(d.severity, d.code.severity(), "severity for {src:?}");
        }
    }
    assert!(lone_seen > 0, "no input had a lone caret");
}
