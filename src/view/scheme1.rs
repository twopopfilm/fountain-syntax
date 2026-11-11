//! The scheme 1 lines of a scene (`SPEC.md` 11.7): the importer-compatible text that `hash_v1`
//! hashes. Compiled in every build; only the digest needs the `hash` feature.
//!
//! Scheme 1 does no Unicode normalisation (unlike scheme 2): white space is collapsed with
//! `split_whitespace` and headings and cues are upper-cased with `to_uppercase`, and that is all.

use super::inline::typed;
use super::{Action, DialogueBlock, DialogueItem, Element, Scene, Text};

/// Each run of Unicode `White_Space` becomes one space; the ends are trimmed.
fn collapse(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn line(text: &Text) -> String {
    collapse(&typed(text))
}

/// The element lines under construction: `group` is the open Action group (typed texts joined
/// with a space), which the next element that ends it turns into one line.
#[derive(Default)]
struct Lines {
    out: Vec<String>,
    group: String,
}

impl Lines {
    /// Add `s` as a line, dropping it when empty.
    fn push(&mut self, s: String) {
        push_nonempty(&mut self.out, s);
    }

    /// End the open Action group: its collapsed line, if it has any text.
    fn flush(&mut self) {
        let group = std::mem::take(&mut self.group);
        self.push(collapse(&group));
    }

    fn element(&mut self, el: &Element) {
        match el {
            // A whole-line Note or Boneyard is skipped and does not end the group.
            Element::Note(_) | Element::Boneyard(_) => {}
            Element::Action(a) if !a.centered => {
                self.group.push(' ');
                self.group.push_str(&typed(&a.text));
                self.synopsis_interjections(a);
            }
            Element::Action(a) => {
                self.flush();
                self.push(line(&a.text));
                self.synopsis_interjections(a);
            }
            Element::Dialogue(b) => {
                self.flush();
                block_lines(b, &mut self.out);
            }
            Element::DualDialogue(d) => {
                self.flush();
                block_lines(&d.left, &mut self.out);
                block_lines(&d.right, &mut self.out);
            }
            other => {
                self.flush();
                kept_lines(other, &mut self.out);
            }
        }
    }

    /// A Synopsis interjection ends the group and gives its line after the group's. A Section or
    /// a Page Break interjection gives nothing and leaves the group open.
    fn synopsis_interjections(&mut self, a: &Action) {
        let mut synopses = a
            .interjections
            .iter()
            .filter(|e| matches!(e, Element::Synopsis(_)))
            .peekable();
        if synopses.peek().is_some() {
            self.flush();
            synopses.for_each(|e| kept_lines(e, &mut self.out));
        }
    }
}

/// Add `s` to `out` as a line, dropping it when empty.
fn push_nonempty(out: &mut Vec<String>, s: String) {
    if !s.is_empty() {
        out.push(s);
    }
}

/// The lines of the kinds that give plain text: Transition, Lyrics, Synopsis and Shot. Everything
/// else gives none here (Sections and Page Breaks are not hashed).
fn kept_lines(el: &Element, out: &mut Vec<String>) {
    let mut push = |s: String| push_nonempty(out, s);
    match el {
        Element::Transition(t) => push(line(&t.text)),
        Element::Lyrics(l) => l.lines.iter().for_each(|t| push(line(t))),
        Element::Synopsis(s) => push(line(&s.text)),
        #[cfg(feature = "shot")]
        Element::Shot(s) => push(line(&s.text)),
        _ => {}
    }
}

/// The cue line, then the lines of the block. While a run of Lines is open an interjection's
/// lines are held; they follow the run's line (as in scheme 2's `block_records`).
fn block_lines(b: &DialogueBlock, out: &mut Vec<String>) {
    let name = line(&b.cue.name).to_uppercase();
    if !name.is_empty() {
        out.push(name);
    }
    let mut run: Option<Vec<String>> = None;
    let mut held = Vec::new();
    let close = |run: &mut Option<Vec<String>>, held: &mut Vec<String>, out: &mut Vec<String>| {
        if let Some(lines) = run.take() {
            out.extend(Some(collapse(&lines.join(" "))).filter(|s| !s.is_empty()));
        }
        out.append(held);
    };
    for item in &b.items {
        match item {
            DialogueItem::Line(t) => run.get_or_insert_with(Vec::new).push(typed(t)),
            DialogueItem::Parenthetical(t) => {
                close(&mut run, &mut held, out);
                // The text keeps its parentheses: strip one of each, then collapse again.
                let inner = line(t);
                let inner = inner.strip_prefix('(').unwrap_or(&inner);
                let inner = inner.strip_suffix(')').unwrap_or(inner);
                out.extend(Some(collapse(inner)).filter(|s| !s.is_empty()));
            }
            DialogueItem::Interjection(el) => {
                kept_lines(el, if run.is_some() { &mut held } else { out });
            }
        }
    }
    close(&mut run, &mut held, out);
}

impl Scene {
    /// The scheme 1 heading line (`SPEC.md` 11.7, rule 3): the heading's text as typed, with
    /// hidden spans deleted, each run of Unicode white space collapsed to one space, then
    /// upper-cased. The forcing `.` and the scene number are not part of it. No Unicode
    /// normalisation is applied.
    pub fn heading_line_v1(&self) -> String {
        line(&self.heading.text).to_uppercase()
    }

    /// The scheme 1 element lines in order (`SPEC.md` 11.7, rule 4), over the scene's hash scope:
    /// its Synopses after the heading, its elements, then the Synopses that trail the last element.
    ///
    /// Consecutive non-centered Actions are one line; a Dialogue block gives its upper-cased cue
    /// name, then a line per Parenthetical and per run of Dialogue lines; Lyrics give a line per
    /// `~` line. Lines are collapsed, have no sigil or kind prefix, and empty lines are dropped.
    /// Sections, Page Breaks, Notes and Boneyards give none.
    pub fn element_lines_v1(&self) -> Vec<String> {
        let (synopses, elements, trailing) = self.hash_scope_v1();
        let mut lines = Lines::default();
        for s in synopses {
            lines.flush();
            lines.push(line(&s.text));
        }
        for el in elements {
            lines.element(el);
        }
        lines.flush();
        for s in trailing {
            lines.push(line(&s.text));
        }
        lines.out
    }

    /// The scheme 1 normalised text (`SPEC.md` 11.7, rule 1): the heading line, a newline, then
    /// the element lines joined with newlines and no newline after the last. A scene with no
    /// lines is the heading line and a newline. `hash_v1` (with the `hash` feature) hashes these bytes.
    pub fn normalized_text_v1(&self) -> String {
        format!(
            "{}\n{}",
            self.heading_line_v1(),
            self.element_lines_v1().join("\n")
        )
    }
}
