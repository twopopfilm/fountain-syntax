//! The canonical writer (`SPEC.md` 10.3): Fountain text for a view built in memory.

use core::fmt::Write as _;

use crate::view::inline::fountain;
use crate::{
    Action, Child, DialogueBlock, DialogueItem, Element, NodeKind, Scene, Screenplay, Section,
    Synopsis, TitlePage, parse,
};

/// A canonical Fountain text for a view built in memory (`SPEC.md` 10.3).
///
/// The text has `\n` line breaks, a final line break and one blank line between blocks. A heading,
/// Action, cue or Transition gets its sigil (`.`, `!`, `@` or `>`) only when, written without
/// it, it would parse as a different element. Spans and `forced` flags are ignored: they record
/// how a parsed text was written, not what it says. An empty Dialogue line is written as two
/// spaces, the only form that parses as one. A parsed scene's Synopses that came from before its
/// heading are written there again, so the re-parse keeps its scheme 1 hash scope.
///
/// Parsing the text gives a view equal to `screenplay`, spans and `forced` aside, and writing
/// that view again gives the same text. The function never panics, but these views don't
/// round-trip:
///
/// - a blank line inside Action;
/// - `Escaped` of a character other than `*` and `_`;
/// - a Plain run whose `*` or `_` would pair, or that ends in `\` before an `Escaped`, or that
///   holds `[[` or `/*`;
/// - adjacent emphasis spans (`*a**b*`);
/// - `]]` in a Note or `*/` in a Boneyard;
/// - an Action with interjections but only one line;
/// - a Dialogue line that reads as a Parenthetical or starts with `!`, `@x`, `.x`, `>`, `~`, `#`,
///   `=` or `===` (the last four read as Lyrics, a Section, a Synopsis or a Page Break);
/// - a heading whose text ends in ` #n#`;
/// - an `unpaired_caret` block right after another block;
/// - Sections or Synopses at the end of the preamble or of a scene;
/// - stray title lines placed among the entries, or shaped `key: value` (it re-parses as an entry);
/// - an empty name or empty text where the element needs text;
/// - a heading, Action, cue or Transition whose text starts with its own sigil (`.`, `!`, `@` or
///   `>`): it parses as the same element without that character;
/// - an empty title page, a title page of stray lines only, or a single title entry with no value:
///   an empty `TitlePage` is left out, so it re-parses as no title page, and the other two re-parse
///   as Action;
/// - a title entry whose first value is empty but which has later values: `values: ["", "A"]` is
///   written `Key:` followed by the continuation line and re-parses as `["A"]`, which is not
///   stable;
/// - a line break in text that holds one line (a Dialogue line, a title value, a heading, a cue,
///   centered Action): it re-parses as several lines or elements, and for centered text the output
///   is not stable;
/// - an empty or split `Plain` run (`Run::Plain("")`, or two adjacent `Plain` runs): the parser
///   merges and drops them, so the view differs; empty text is `runs: []`;
/// - emphasis with both `bold` and `underline`, which is written `_x_` and loses the bold; a
///   `Section` with `depth: 0`, for which no `#` is written;
/// - an Interjection that has no one-line form, and any `Element` variant this version does not
///   know (the enum is `#[non_exhaustive]`): they are left out of the output without notice.
///
/// ```
/// use fountain_syntax::{Heading, Run, Scene, Screenplay, Text, canonical, parse};
///
/// let heading = |s: &str| Heading {
///     text: Text { span: None, runs: vec![Run::Plain(s.into())] },
///     forced: false,
///     span: None,
/// };
/// let view = Screenplay {
///     title_page: None,
///     preamble: Default::default(),
///     scenes: vec![Scene::new(heading("INT. HOUSE - DAY")), Scene::new(heading("ROOF"))],
/// };
/// // `ROOF` alone would read as Action, so it gets the `.`; the first heading needs none.
/// let text = canonical(&view);
/// assert_eq!(text, "INT. HOUSE - DAY\n\n.ROOF\n");
///
/// let back = parse(&text).screenplay();
/// assert_eq!(back.scenes[1].heading.text.plain(), "ROOF");
/// assert_eq!(canonical(&back), text);
/// ```
pub fn canonical(screenplay: &Screenplay) -> String {
    // A title page with no text is left out: a blank block would only add blank lines.
    let page = screenplay.title_page.as_ref().map(title_page);
    let page = page.filter(|p| !p.is_empty());
    let mut w = Writer {
        out: String::new(),
        open: page.is_none(),
    };
    if let Some(page) = &page {
        w.block(page);
    }
    for e in &screenplay.preamble.elements {
        w.element(e);
    }
    for scene in &screenplay.scenes {
        w.scene(scene);
    }
    w.out
}

struct Writer {
    out: String,
    /// True while no title page has been written and only Sections, Synopses, Notes and
    /// Boneyards have: a block can still read as the title page, so trials see the output so far.
    open: bool,
}

impl Writer {
    fn block(&mut self, text: &str) {
        if !self.out.is_empty() {
            self.out.push('\n');
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    /// Writes `block`, with `sigil` in front if, parsed after the output so far (or after a
    /// neutral Action once that can no longer matter), it starts a node other than `kind`.
    fn trial(&mut self, kind: NodeKind, sigil: char, block: String) {
        let prefix = match (self.open, self.out.is_empty()) {
            (false, _) => "x\n\n".to_owned(),
            (true, true) => String::new(),
            (true, false) => format!("{}\n", self.out),
        };
        let src = format!("{prefix}{block}\n");
        let found = parse(&src).root().children().find_map(|c| match c {
            Child::Node(n) if n.kind() != NodeKind::Gap && n.span().start == prefix.len() => {
                Some(n.kind())
            }
            _ => None,
        });
        self.open = false;
        if found == Some(kind) {
            self.block(&block);
        } else {
            self.block(&format!("{sigil}{block}"));
        }
    }

    fn scene(&mut self, scene: &Scene) {
        // The Synopses before the heading are those the private scheme 1 scope leaves out.
        let leading = scene.synopses.len() - scene.hash_scope_v1().0.len();
        for s in &scene.sections {
            self.block(&section(s));
        }
        for s in &scene.synopses[..leading] {
            self.block(&synopsis(s));
        }
        let mut heading = fountain(&scene.heading.text);
        if let Some(n) = &scene.number {
            #[cfg(feature = "locked-scene-numbers")]
            let lock = if n.locked { "#" } else { "" };
            #[cfg(not(feature = "locked-scene-numbers"))]
            let lock = "";
            let _ = write!(heading, " {lock}#{}#", n.text);
        }
        self.trial(NodeKind::SceneHeading, '.', heading);
        for s in &scene.synopses[leading..] {
            self.block(&synopsis(s));
        }
        for e in &scene.elements {
            self.element(e);
        }
    }

    fn element(&mut self, e: &Element) {
        match e {
            Element::Action(a) if a.centered => self.block(&format!("> {} <", fountain(&a.text))),
            Element::Action(a) => self.trial(NodeKind::Action, '!', action(a)),
            Element::Dialogue(d) => {
                self.trial(NodeKind::DialogueBlock, '@', dialogue(d, d.unpaired_caret))
            }
            Element::DualDialogue(pair) => {
                self.trial(NodeKind::DialogueBlock, '@', dialogue(&pair.left, false));
                self.trial(NodeKind::DialogueBlock, '@', dialogue(&pair.right, true));
            }
            Element::Transition(t) => self.trial(NodeKind::Transition, '>', fountain(&t.text)),
            _ => {
                if let Some(text) = line(e) {
                    self.block(&text);
                }
            }
        }
        self.open &= matches!(
            e,
            Element::Section(_) | Element::Synopsis(_) | Element::Note(_) | Element::Boneyard(_)
        );
    }
}

fn title_page(page: &TitlePage) -> String {
    let mut lines = Vec::new();
    for entry in &page.entries {
        let key = fountain(&entry.key);
        let mut values = entry.values.iter().map(fountain);
        match values.next() {
            Some(v) if !v.is_empty() => lines.push(format!("{key}: {v}")),
            _ => lines.push(format!("{key}:")),
        }
        lines.extend(values.map(|v| format!("    {v}")));
    }
    lines.extend(page.stray_lines.iter().map(fountain));
    lines.join("\n")
}

/// The paragraph with its interjections after the first line, and a `!` on each later line that
/// would otherwise start another element.
fn action(a: &Action) -> String {
    let text = fountain(&a.text);
    let mut lines = text.split('\n');
    let mut out = lines.next().unwrap_or_default().to_owned();
    for e in &a.interjections {
        if let Some(l) = line(e) {
            out.push('\n');
            out.push_str(&l);
        }
    }
    for l in lines {
        out.push('\n');
        if l.trim_start()
            .starts_with(['!', '@', '.', '>', '~', '#', '='])
        {
            out.push('!');
        }
        out.push_str(l);
    }
    out
}

fn dialogue(d: &DialogueBlock, caret: bool) -> String {
    let mut out = fountain(&d.cue.name);
    for x in &d.cue.extensions {
        let _ = write!(out, " ({})", fountain(&x.text));
    }
    if caret {
        out.push_str(" ^");
    }
    for item in &d.items {
        let l = match item {
            DialogueItem::Line(t) => match fountain(t) {
                // Two spaces are the only way to write an empty Dialogue line.
                s if s.is_empty() => "  ".to_owned(),
                s => s,
            },
            DialogueItem::Parenthetical(t) => fountain(t),
            DialogueItem::Interjection(e) => match line(e) {
                Some(l) => l,
                None => continue,
            },
        };
        out.push('\n');
        out.push_str(&l);
    }
    out
}

fn section(s: &Section) -> String {
    format!("{} {}", "#".repeat(s.depth), fountain(&s.text))
}

fn synopsis(s: &Synopsis) -> String {
    format!("= {}", fountain(&s.text))
}

/// The text of an element that needs no trial; `None` for the others.
fn line(e: &Element) -> Option<String> {
    Some(match e {
        Element::Lyrics(l) => l
            .lines
            .iter()
            .map(|t| format!("~{}", fountain(t)))
            .collect::<Vec<_>>()
            .join("\n"),
        Element::Section(s) => section(s),
        Element::Synopsis(s) => synopsis(s),
        Element::PageBreak(_) => "===".to_owned(),
        Element::Note(n) => format!("[[{}]]", n.text),
        Element::Boneyard(b) => format!("/*{}*/", b.text),
        #[cfg(feature = "shot")]
        Element::Shot(s) => format!("!!{}", fountain(&s.text)),
        _ => return None,
    })
}
