mod common;

use common::parse_checked;
use fountain_syntax::parse;

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
    for _ in 0..5_000 {
        let count = xorshift(&mut state) % 49;
        let src: String = (0..count)
            .map(|_| FRAGMENTS[(xorshift(&mut state) % FRAGMENTS.len() as u64) as usize])
            .collect();
        let doc = parse_checked(&src);
        let _ = doc.screenplay();
        let diags = doc.diagnostics();
        assert_eq!(diags, parse(&src).diagnostics(), "unstable for {src:?}");
        let mut last = 0;
        for d in diags {
            assert!(last <= d.span.start, "unsorted for {src:?}");
            last = d.span.start;
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
}
