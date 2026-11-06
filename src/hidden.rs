//! Hidden spans (SPEC 3.4): finds every Note `[[ ]]` and Boneyard `/* */` in one scan, and fuses
//! the physical lines a span crosses into one logical line with its visible text.

use crate::lines::{self, Line};
use crate::tree::{NodeKind, Span};

/// One Note or Boneyard, markers included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Hidden {
    /// `NodeKind::Note` or `NodeKind::Boneyard`.
    pub(crate) kind: NodeKind,
    pub(crate) span: Span,
}

/// Finds the hidden spans of `src[from..]`, in order. `lines` are its physical lines.
///
/// The first opener wins and there is no nesting. A Boneyard closes at the first `*/` after its
/// two opener bytes. A Note closes at the first `]]`, searched no further than the next blank line
/// that is not exactly two spaces. An unclosed opener is text and the scan resumes after it.
pub(crate) fn scan(src: &str, from: usize, lines: &[Line]) -> Vec<Hidden> {
    let bytes = src.as_bytes();
    let mut out = Vec::new();
    // Once a search fails, later openers fail too; remembering that keeps the scan linear.
    let mut boneyard_can_close = true;
    let mut note_fails_before = 0;
    let mut i = from;
    while i + 1 < bytes.len() {
        let found = match &bytes[i..i + 2] {
            b"/*" if boneyard_can_close => {
                let end = src[i + 2..].find("*/").map(|j| i + 2 + j + 2);
                boneyard_can_close = end.is_some();
                end.map(|end| (NodeKind::Boneyard, end))
            }
            b"[[" if i >= note_fails_before => match note_end(src, i, lines) {
                Ok(end) => Some((NodeKind::Note, end)),
                Err(stop) => {
                    note_fails_before = stop;
                    None
                }
            },
            b"/*" | b"[[" => None,
            _ => {
                i += 1;
                continue;
            }
        };
        match found {
            Some((kind, end)) => {
                out.push(Hidden {
                    kind,
                    span: Span::new(i, end),
                });
                i = end;
            }
            None => i += 2,
        }
    }
    out
}

/// The end of the Note opened at `open`, or `Err` with the offset its search stopped at.
fn note_end(src: &str, open: usize, lines: &[Line]) -> Result<usize, usize> {
    let mut k = lines.partition_point(|l| l.text.end < open);
    let mut at = open + 2;
    loop {
        if let Some(j) = src[at..lines[k].text.end].find("]]") {
            return Ok(at + j + 2);
        }
        k += 1;
        let Some(next) = lines.get(k) else {
            return Err(src.len());
        };
        let text = &src[next.text.start..next.text.end];
        if lines::is_blank(text) && text != "  " {
            return Err(next.text.start);
        }
        at = next.text.start;
    }
}

/// A run of a logical line: visible source text, or the index of a hidden span.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Piece {
    Visible(Span),
    Hidden(usize),
}

/// One or more physical lines fused by the hidden spans that cross their line breaks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Logical {
    /// The line break that ends the last physical line.
    pub(crate) brk: Option<Span>,
    /// The line's text in order, with no empty `Visible` piece.
    pub(crate) pieces: Vec<Piece>,
    /// The visible pieces joined: the text recognition reads (SPEC 3.4, rule 2).
    pub(crate) visible: String,
}

impl Logical {
    /// Whitespace-only visible text and no hidden piece.
    pub(crate) fn is_blank(&self) -> bool {
        lines::is_blank(&self.visible) && !self.has_hidden()
    }

    /// Whitespace-only visible text and at least one hidden piece (SPEC 3.4, rule 3).
    pub(crate) fn is_invisible(&self) -> bool {
        lines::is_blank(&self.visible) && self.has_hidden()
    }

    fn has_hidden(&self) -> bool {
        self.pieces.iter().any(|p| matches!(p, Piece::Hidden(_)))
    }
}

/// Fuses `lines` into logical lines along the `hidden` spans from [`scan`].
pub(crate) fn logical(src: &str, lines: &[Line], hidden: &[Hidden]) -> Vec<Logical> {
    let mut out = Vec::new();
    let mut h = 0;
    let mut k = 0;
    while k < lines.len() {
        let mut pieces = Vec::new();
        let mut visible = String::new();
        let mut push = |pieces: &mut Vec<Piece>, s: Span| {
            if !s.is_empty() {
                pieces.push(Piece::Visible(s));
                visible.push_str(&src[s.start..s.end]);
            }
        };
        let mut at = lines[k].text.start;
        while let Some(span) = hidden
            .get(h)
            .map(|hd| hd.span)
            .filter(|s| s.start < lines[k].text.end)
        {
            push(&mut pieces, Span::new(at, span.start));
            pieces.push(Piece::Hidden(h));
            h += 1;
            at = span.end;
            // A span never ends on a line break, so it ends inside some later line's text.
            while lines[k].text.end < at {
                k += 1;
            }
        }
        push(&mut pieces, Span::new(at, lines[k].text.end));
        out.push(Logical {
            brk: lines[k].brk,
            pieces,
            visible,
        });
        k += 1;
    }
    out
}
