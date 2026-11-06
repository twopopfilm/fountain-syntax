//! Inline text for the typed view (SPEC 4.11, `OFF-14`): emphasis, escapes and hidden runs.
//!
//! A pure function layer with no dependency on the tree. The builder hands [`text`] an ordered
//! list of [`Piece`]s for one piece of text (visible text, line breaks, hidden atoms) and gets a
//! [`Text`] back.

use super::{Boneyard, Note, Run, Text};
use crate::Span;

/// One piece of the source of a [`Text`], in source order.
#[derive(Debug)]
pub(crate) enum Piece<'a> {
    /// Visible text. `span` covers exactly `text` in the source.
    Text {
        /// The characters, as written.
        text: &'a str,
        /// Where they are in the source.
        span: Span,
    },
    /// A line break as written (`\n` or `\r\n`). Emphasis never crosses it; it stays in the
    /// output, inside a [`Run::Plain`].
    Break {
        /// The break, as written.
        text: &'a str,
        /// Where it is in the source.
        span: Span,
    },
    /// An inline Note. Opaque: never scanned for emphasis. It may sit inside an emphasis span.
    Note(Note),
    /// An inline Boneyard. Opaque, like [`Piece::Note`].
    Boneyard(Boneyard),
}

/// Which spaces and tabs [`text`] drops.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Trim {
    /// Spaces and tabs at both ends of the whole text.
    Both,
    /// Spaces and tabs at the end of every line; each line keeps its leading whitespace.
    ActionLines,
}

/// Build the [`Text`] for `pieces`.
///
/// `trim` is applied first (hidden atoms and line breaks are never trimmed and stop the trim).
/// Then each line is scanned for emphasis. The `span` of the result runs from the first to the
/// last byte that survives the trim; it is `None` when nothing does.
pub(crate) fn text<'a>(pieces: impl IntoIterator<Item = Piece<'a>>, trim: Trim) -> Text {
    let mut lines = split_lines(pieces);
    let last = lines.len() - 1;
    for (i, line) in lines.iter_mut().enumerate() {
        if trim == Trim::Both && i == 0 {
            line.trim_start();
        }
        if trim == Trim::ActionLines || i == last {
            line.trim_end();
        }
    }

    let mut out = Vec::new();
    let mut span: Option<Span> = None;
    for line in lines {
        for item in &line.items {
            let covered = match item {
                Item::Text(t, start) => Some(Span::new(*start, start + t.len())),
                Item::Hidden(_, span) => *span,
            };
            cover(&mut span, covered);
        }
        for run in scan(line.items) {
            push_run(&mut out, run);
        }
        if let Some((text, brk_span)) = line.brk {
            cover(&mut span, Some(brk_span));
            push_str(&mut out, text);
        }
    }
    Text { span, runs: out }
}

/// Grow `span` to include `add`.
fn cover(span: &mut Option<Span>, add: Option<Span>) {
    let Some(add) = add else { return };
    *span = Some(match *span {
        Some(s) => Span::new(s.start.min(add.start), s.end.max(add.end)),
        None => add,
    });
}

/// A piece of a line: text starting at a source byte, or a hidden atom with its own span.
enum Item<'a> {
    Text(&'a str, usize),
    Hidden(Run, Option<Span>),
}

/// The items of one line and the break that ends it (`None` for the last line).
struct Line<'a> {
    items: Vec<Item<'a>>,
    brk: Option<(&'a str, Span)>,
}

/// Cut `pieces` at their line breaks. Always returns at least one (possibly empty) line.
fn split_lines<'a>(pieces: impl IntoIterator<Item = Piece<'a>>) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let mut items = Vec::new();
    for piece in pieces {
        match piece {
            Piece::Text { text, span } => items.push(Item::Text(text, span.start)),
            Piece::Note(n) => {
                let span = n.span;
                items.push(Item::Hidden(Run::Note(n), span));
            }
            Piece::Boneyard(b) => {
                let span = b.span;
                items.push(Item::Hidden(Run::Boneyard(b), span));
            }
            Piece::Break { text, span } => lines.push(Line {
                items: std::mem::take(&mut items),
                brk: Some((text, span)),
            }),
        }
    }
    lines.push(Line { items, brk: None });
    lines
}

impl Line<'_> {
    /// Drop spaces and tabs from the start, up to the first hidden atom.
    fn trim_start(&mut self) {
        while let Some(Item::Text(t, start)) = self.items.first_mut() {
            let trimmed = t.trim_start_matches([' ', '\t']);
            *start += t.len() - trimmed.len();
            *t = trimmed;
            if !t.is_empty() {
                break;
            }
            self.items.remove(0);
        }
    }

    /// Drop spaces and tabs from the end, back to the last hidden atom.
    fn trim_end(&mut self) {
        while let Some(Item::Text(t, _)) = self.items.last_mut() {
            *t = t.trim_end_matches([' ', '\t']);
            if !t.is_empty() {
                break;
            }
            self.items.pop();
        }
    }
}

/// A character of a line, an escaped mark, or a hidden atom.
enum Tok {
    Ch(char),
    Esc(char),
    Hidden(Run),
}

impl Tok {
    /// True for a character that is not whitespace; escapes and hidden atoms count.
    fn solid(&self) -> bool {
        !matches!(self, Tok::Ch(c) if c.is_whitespace())
    }

    fn alnum(&self) -> bool {
        matches!(self, Tok::Ch(c) if c.is_alphanumeric())
    }
}

/// The marks of a matched pair, set at the first token of each mark's run.
#[derive(Clone, Copy)]
enum Mark {
    Open(char, usize),
    Close,
}

/// An opener waiting for its closer: where its run starts, and its character and length.
struct Opener {
    at: usize,
    ch: char,
    len: usize,
}

/// Escape and emphasis for one line (`OFF-14`).
fn scan(items: Vec<Item<'_>>) -> Vec<Run> {
    let mut toks = Vec::new();
    for item in items {
        match item {
            Item::Text(t, _) => toks.extend(t.chars().map(Tok::Ch)),
            Item::Hidden(run, _) => toks.push(Tok::Hidden(run)),
        }
    }
    let toks = escape(toks);
    let marks = pair(&toks);
    build(toks, &marks)
}

/// Turn a backslash and the `*` or `_` after it into [`Tok::Esc`]. `\\` stays two literal
/// backslashes; a backslash before anything else, or at the end, stays literal.
fn escape(toks: Vec<Tok>) -> Vec<Tok> {
    let mut out = Vec::with_capacity(toks.len());
    let mut it = toks.into_iter().peekable();
    while let Some(tok) = it.next() {
        match (&tok, it.peek()) {
            (Tok::Ch('\\'), Some(Tok::Ch(c @ ('*' | '_')))) => {
                out.push(Tok::Esc(*c));
                it.next();
            }
            (Tok::Ch('\\'), Some(Tok::Ch('\\'))) => {
                out.extend([tok, Tok::Ch('\\')]);
                it.next();
            }
            _ => out.push(tok),
        }
    }
    out
}

/// Find the matching marks. Runs of `*` of length 1 to 3 and single `_` are marks; every other
/// run is literal.
fn pair(toks: &[Tok]) -> Vec<Option<Mark>> {
    let mut marks = vec![None; toks.len()];
    let mut stack: Vec<Opener> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let Tok::Ch(ch @ ('*' | '_')) = toks[i] else {
            i += 1;
            continue;
        };
        let end = i + toks[i..]
            .iter()
            .take_while(|t| matches!(t, Tok::Ch(c) if *c == ch))
            .count();
        let len = end - i;
        let before = i.checked_sub(1).map(|k| &toks[k]);
        let after = toks.get(end);
        let max = if ch == '*' { 3 } else { 1 };
        if len <= max {
            let intraword =
                ch == '_' && before.is_some_and(Tok::alnum) && after.is_some_and(Tok::alnum);
            let can_close = before.is_some_and(Tok::solid) && !intraword;
            let can_open = after.is_some_and(Tok::solid) && !intraword;
            let found = can_close
                .then(|| stack.iter().rposition(|o| o.ch == ch && o.len == len))
                .flatten();
            if let Some(p) = found {
                // Openers above the match stay literal: the first-opened span wins.
                marks[stack[p].at] = Some(Mark::Open(ch, len));
                marks[i] = Some(Mark::Close);
                stack.truncate(p);
            } else if can_open {
                stack.push(Opener { at: i, ch, len });
            }
        }
        i = end;
    }
    marks
}

/// Fold the tokens into runs, nesting the matched pairs.
fn build(toks: Vec<Tok>, marks: &[Option<Mark>]) -> Vec<Run> {
    let mut out = Vec::new();
    // Open spans: the mark that opened each, and the runs collected so far.
    let mut open: Vec<(char, usize, Vec<Run>)> = Vec::new();
    let mut skip = 0;
    for (i, tok) in toks.into_iter().enumerate() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let into = open.last_mut().map_or(&mut out, |o| &mut o.2);
        match marks[i] {
            Some(Mark::Open(ch, len)) => {
                open.push((ch, len, Vec::new()));
                skip = len - 1;
            }
            Some(Mark::Close) => {
                if let Some((ch, len, runs)) = open.pop() {
                    let run = Run::Emphasis {
                        bold: ch == '*' && len >= 2,
                        italic: ch == '*' && len != 2,
                        underline: ch == '_',
                        runs,
                    };
                    push_run(open.last_mut().map_or(&mut out, |o| &mut o.2), run);
                    skip = len - 1;
                }
            }
            None => match tok {
                Tok::Ch(c) => push_str(into, c.encode_utf8(&mut [0; 4])),
                Tok::Esc(c) => into.push(Run::Escaped(c)),
                Tok::Hidden(run) => into.push(run),
            },
        }
    }
    out
}

/// Append `run`, merging it into a `Plain` run before it.
fn push_run(runs: &mut Vec<Run>, run: Run) {
    match run {
        Run::Plain(s) => push_str(runs, &s),
        other => runs.push(other),
    }
}

/// Append plain text, merging it into a `Plain` run before it.
fn push_str(runs: &mut Vec<Run>, s: &str) {
    match runs.last_mut() {
        Some(Run::Plain(p)) => p.push_str(s),
        _ => runs.push(Run::Plain(s.to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vis(s: &str) -> Piece<'_> {
        vis_at(s, 0)
    }

    fn vis_at(s: &str, start: usize) -> Piece<'_> {
        Piece::Text {
            text: s,
            span: Span::new(start, start + s.len()),
        }
    }

    fn brk(s: &str, start: usize) -> Piece<'_> {
        Piece::Break {
            text: s,
            span: Span::new(start, start + s.len()),
        }
    }

    fn note(s: &str, span: Span) -> Note {
        Note {
            text: s.to_owned(),
            span: Some(span),
        }
    }

    fn boneyard(s: &str, span: Span) -> Boneyard {
        Boneyard {
            text: s.to_owned(),
            span: Some(span),
        }
    }

    /// One line of visible text, trimmed at both ends.
    fn runs(s: &str) -> Vec<Run> {
        text([vis(s)], Trim::Both).runs
    }

    fn plain(s: &str) -> Run {
        Run::Plain(s.to_owned())
    }

    fn em(bold: bool, italic: bool, underline: bool, runs: Vec<Run>) -> Run {
        Run::Emphasis {
            bold,
            italic,
            underline,
            runs,
        }
    }

    fn italic(runs: Vec<Run>) -> Run {
        em(false, true, false, runs)
    }

    fn bold(runs: Vec<Run>) -> Run {
        em(true, false, false, runs)
    }

    fn under(runs: Vec<Run>) -> Run {
        em(false, false, true, runs)
    }

    #[test]
    fn el_emphasis_1_each_style_and_nesting() {
        assert_eq!(runs("*a*"), [italic(vec![plain("a")])]);
        assert_eq!(runs("**a**"), [bold(vec![plain("a")])]);
        assert_eq!(runs("***a***"), [em(true, true, false, vec![plain("a")])]);
        assert_eq!(runs("_a_"), [under(vec![plain("a")])]);
        assert_eq!(
            runs("so *very* **loud** and _flat_ now"),
            [
                plain("so "),
                italic(vec![plain("very")]),
                plain(" "),
                bold(vec![plain("loud")]),
                plain(" and "),
                under(vec![plain("flat")]),
                plain(" now"),
            ]
        );
        assert_eq!(
            runs("_*x*_"),
            [under(vec![italic(vec![plain("x")])])],
            "underline around italic"
        );
        assert_eq!(
            runs("*a **b** c*"),
            [italic(vec![
                plain("a "),
                bold(vec![plain("b")]),
                plain(" c")
            ])]
        );
        assert_eq!(
            runs("**a *b* c**"),
            [bold(vec![
                plain("a "),
                italic(vec![plain("b")]),
                plain(" c")
            ])]
        );
        // Two spans of the same kind side by side stay two spans.
        assert_eq!(
            runs("*a* *b*"),
            [
                italic(vec![plain("a")]),
                plain(" "),
                italic(vec![plain("b")])
            ]
        );
    }

    #[test]
    fn el_emphasis_2_escapes_and_unmatched_literal() {
        assert_eq!(runs("a * b"), [plain("a * b")]);
        assert_eq!(runs("*open"), [plain("*open")]);
        assert_eq!(runs("close*"), [plain("close*")]);
        assert_eq!(runs("_open"), [plain("_open")]);
        assert_eq!(runs("**a*"), [plain("**a*")]);
        assert_eq!(
            runs("****a****"),
            [plain("****a****")],
            "four marks are literal"
        );
        assert_eq!(
            runs("__a__"),
            [plain("__a__")],
            "a doubled underscore is literal"
        );
        assert_eq!(
            runs("\\*not italic*"),
            [Run::Escaped('*'), plain("not italic*")]
        );
        assert_eq!(
            runs("a *b* *c"),
            [plain("a "), italic(vec![plain("b")]), plain(" *c")]
        );
    }

    #[test]
    fn off_14a_open_and_close_need_non_whitespace() {
        assert_eq!(runs("* not open*"), [plain("* not open*")]);
        assert_eq!(runs("*not close *"), [plain("*not close *")]);
        assert_eq!(runs("a * b * c"), [plain("a * b * c")]);
        assert_eq!(runs("_ a_"), [plain("_ a_")]);
        assert_eq!(runs("_a _"), [plain("_a _")]);
        assert_eq!(runs("*\ta*"), [plain("*\ta*")], "a tab is whitespace");
    }

    #[test]
    fn off_14b_no_span_across_line_break() {
        let t = text(
            [vis("*across"), brk("\n", 7), vis_at("lines*", 8)],
            Trim::Both,
        );
        assert_eq!(t.runs, [plain("*across\nlines*")]);
        let t = text([vis("*a"), brk("\r\n", 2), vis_at("b*", 4)], Trim::Both);
        assert_eq!(t.runs, [plain("*a\r\nb*")], "CRLF is kept as written");
        // Each line is scanned on its own; the break joins the plain text around it.
        let t = text([vis("*a*"), brk("\n", 3), vis_at("*b*", 4)], Trim::Both);
        assert_eq!(
            t.runs,
            [
                italic(vec![plain("a")]),
                plain("\n"),
                italic(vec![plain("b")])
            ]
        );
        // An opener whose line ends before a closer does not capture the next line.
        let t = text([vis("x *a"), brk("\n", 4), vis_at("b* *c*", 5)], Trim::Both);
        assert_eq!(t.runs, [plain("x *a\nb* "), italic(vec![plain("c")])]);
    }

    #[test]
    fn off_14c_intraword_underscore() {
        assert_eq!(runs("snake_case_name"), [plain("snake_case_name")]);
        assert_eq!(runs("_a_b_"), [under(vec![plain("a_b")])]);
        assert_eq!(runs("_snake_case"), [plain("_snake_case")]);
        assert_eq!(
            runs("un*frigging*believable"),
            [
                plain("un"),
                italic(vec![plain("frigging")]),
                plain("believable")
            ],
            "`*` may be intraword"
        );
        assert_eq!(
            runs("x _y_ z"),
            [plain("x "), under(vec![plain("y")]), plain(" z")]
        );
    }

    #[test]
    fn off_14d_crossing_and_mismatched_runs() {
        // Nested with a different run length: italic with bold inside.
        assert_eq!(
            runs("*bold **inside** italic*"),
            [italic(vec![
                plain("bold "),
                bold(vec![plain("inside")]),
                plain(" italic")
            ])]
        );
        // Crossing: the first-opened span wins and the `_` marks are literal.
        assert_eq!(
            runs("*start _nest* end_"),
            [italic(vec![plain("start _nest")]), plain(" end_")]
        );
        // A closer of another length does not match.
        assert_eq!(runs("***triple*"), [plain("***triple*")]);
        assert_eq!(runs("*a**"), [plain("*a**")]);
        // The closer matches the nearest opener of its own length; the shorter one above it
        // becomes literal.
        assert_eq!(
            runs("**a *b** c*"),
            [bold(vec![plain("a *b")]), plain(" c*")]
        );
    }

    #[test]
    fn off_14e_escapes() {
        assert_eq!(
            runs("\\_literal\\_"),
            [Run::Escaped('_'), plain("literal"), Run::Escaped('_')]
        );
        assert_eq!(runs("\\\\"), [plain("\\\\")]);
        assert_eq!(runs("\\#"), [plain("\\#")]);
        assert_eq!(runs("a\\"), [plain("a\\")]);
        assert_eq!(runs("\\"), [plain("\\")]);
        // An escaped mark neither opens nor closes.
        assert_eq!(runs("*a\\*"), [plain("*a"), Run::Escaped('*')]);
        // `\\` is two literal backslashes, so the `*` after it is a mark.
        assert_eq!(runs("\\\\*a*"), [plain("\\\\"), italic(vec![plain("a")])]);
        assert_eq!(text([vis("\\*x\\_")], Trim::Both).plain(), "*x_");
    }

    #[test]
    fn off_14f_hidden_spans_not_scanned() {
        let n = note("*x*", Span::new(5, 12));
        let t = text(
            [
                vis_at("a *b ", 0),
                Piece::Note(n.clone()),
                vis_at(" c* d", 12),
            ],
            Trim::Both,
        );
        assert_eq!(
            t.runs,
            [
                plain("a "),
                italic(vec![plain("b "), Run::Note(n), plain(" c")]),
                plain(" d")
            ]
        );
        // Marks inside a hidden atom never pair with marks outside it.
        let b = boneyard("_", Span::new(2, 7));
        let t = text([vis("_a"), Piece::Boneyard(b.clone())], Trim::Both);
        assert_eq!(t.runs, [plain("_a"), Run::Boneyard(b.clone())]);
        // A hidden atom counts as non-whitespace for opener and closer.
        let t = text([vis("*"), Piece::Boneyard(b.clone()), vis("*")], Trim::Both);
        assert_eq!(t.runs, [italic(vec![Run::Boneyard(b)])]);
    }

    #[test]
    fn trim_both_ends_only() {
        let t = text([vis(" \t*a* \t")], Trim::Both);
        assert_eq!(t.runs, [italic(vec![plain("a")])]);
        assert_eq!(t.span, Some(Span::new(2, 5)));
        // Inner lines keep their whitespace.
        let t = text([vis("  a "), brk("\n", 4), vis_at("  b  ", 5)], Trim::Both);
        assert_eq!(t.runs, [plain("a \n  b")]);
        assert_eq!(t.span, Some(Span::new(2, 8)));
        // A hidden atom stops the trim.
        let n = note("n", Span::new(2, 7));
        let t = text(
            [vis("  "), Piece::Note(n.clone()), vis_at(" x ", 7)],
            Trim::Both,
        );
        assert_eq!(t.runs, [Run::Note(n), plain(" x")]);
        assert_eq!(t.span, Some(Span::new(2, 9)));
    }

    #[test]
    fn trim_action_lines_keeps_leading_drops_trailing() {
        let t = text(
            [
                vis_at("  a  ", 10),
                brk("\r\n", 15),
                vis_at("\tb \t", 17),
                brk("\n", 21),
                vis_at("  ", 22),
            ],
            Trim::ActionLines,
        );
        assert_eq!(t.runs, [plain("  a\r\n\tb\n")]);
        assert_eq!(t.span, Some(Span::new(10, 22)));
        // Emphasis still works after the trim.
        let t = text([vis_at("  *a*  ", 0)], Trim::ActionLines);
        assert_eq!(t.runs, [plain("  "), italic(vec![plain("a")])]);
        assert_eq!(t.span, Some(Span::new(0, 5)));
    }

    #[test]
    fn empty_input_has_no_span() {
        assert_eq!(text(std::iter::empty(), Trim::Both), Text::default());
        assert_eq!(text([vis(" \t ")], Trim::Both), Text::default());
        assert_eq!(text([vis("  ")], Trim::ActionLines), Text::default());
    }

    #[test]
    fn span_covers_first_to_last_byte() {
        let n = note("n", Span::new(1, 6));
        let b = boneyard("b", Span::new(13, 20));
        let t = text(
            [
                Piece::Note(n),
                vis_at(" mid ", 6),
                brk("\n", 11),
                vis_at("x", 12),
                Piece::Boneyard(b),
            ],
            Trim::Both,
        );
        assert_eq!(t.span, Some(Span::new(1, 20)));
    }
}
