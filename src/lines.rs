//! The physical-line layer: splits the source at `\n`, `\r\n` and `\r` and nothing else (SPEC 3.2).

use crate::tree::Span;

/// One physical line: its text and the line break that ends it, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Line {
    pub(crate) text: Span,
    pub(crate) brk: Option<Span>,
}

/// Splits `src[from..]` into lines. A final line break makes no extra empty line.
pub(crate) fn physical(src: &str, from: usize) -> Vec<Line> {
    let bytes = src.as_bytes();
    let mut lines = Vec::new();
    let mut start = from;
    let mut i = from;
    while i < bytes.len() {
        let brk_len = match bytes[i] {
            b'\n' => 1,
            b'\r' if bytes.get(i + 1) == Some(&b'\n') => 2,
            b'\r' => 1,
            _ => {
                i += 1;
                continue;
            }
        };
        lines.push(Line {
            text: Span::new(start, i),
            brk: Some(Span::new(i, i + brk_len)),
        });
        i += brk_len;
        start = i;
    }
    if start < bytes.len() {
        lines.push(Line {
            text: Span::new(start, bytes.len()),
            brk: None,
        });
    }
    lines
}

/// True for an empty string or one of spaces and tabs only (SPEC 3.3).
pub(crate) fn is_blank(text: &str) -> bool {
    text.bytes().all(|b| b == b' ' || b == b'\t')
}
