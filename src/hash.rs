//! Scheme 2 hashing (`SPEC.md` 11.3 and 11.4). Compiled only with the `hash` feature.

use crate::digest::{Digest, HashScheme};
use crate::tree::{Document, Node, Span};
use crate::view::inline::typed;
use crate::view::{
    Cue, DialogueBlock, DialogueItem, Element, Preamble, Scene, Screenplay, Synopsis, TitleEntry,
};
use sha2::{Digest as _, Sha256};
use unicode_normalization::UnicodeNormalization;

/// NFC, collapse each run of Unicode `White_Space` to one space and trim; with `upper`, then the
/// full upper-case mapping and NFC again. `char::is_whitespace` is the `White_Space` property.
pub(crate) fn normalise(s: &str, upper: bool) -> String {
    let nfc: String = s.nfc().collect();
    let collapsed = nfc.split_whitespace().collect::<Vec<_>>().join(" ");
    if upper {
        collapsed.to_uppercase().nfc().collect()
    } else {
        collapsed
    }
}

/// The byte encoder of `SPEC.md` 11.4.
pub(crate) struct Pre(pub(crate) Vec<u8>);

impl Pre {
    /// A `u32`, four bytes big endian. A count that does not fit is outside the scheme.
    pub(crate) fn u32(&mut self, n: usize) -> &mut Self {
        let n = u32::try_from(n).expect("a count or length over u32::MAX is outside scheme 2");
        self.0.extend_from_slice(&n.to_be_bytes());
        self
    }

    pub(crate) fn bool(&mut self, b: bool) -> &mut Self {
        self.0.push(u8::from(b));
        self
    }

    pub(crate) fn string(&mut self, s: &str) -> &mut Self {
        self.u32(s.len());
        self.0.extend_from_slice(s.as_bytes());
        self
    }

    pub(crate) fn list(&mut self, items: &[String]) -> &mut Self {
        self.u32(items.len());
        for item in items {
            self.string(item);
        }
        self
    }
}

/// `fountain-syntax/<name>/2` and a NUL.
pub(crate) fn domain(name: &str) -> Vec<u8> {
    format!("fountain-syntax/{name}/2\0").into_bytes()
}

/// SHA-256 of `bytes`, as a scheme 2 digest.
pub(crate) fn digest(bytes: &[u8]) -> Digest {
    Digest {
        scheme: HashScheme::V2,
        bytes: Sha256::digest(bytes).into(),
    }
}

const ACTION: u8 = 0x01;
const CHARACTER: u8 = 0x02;
const DIALOGUE: u8 = 0x03;
const PARENTHETICAL: u8 = 0x04;
const TRANSITION: u8 = 0x05;
const LYRICS: u8 = 0x06;
const SYNOPSIS: u8 = 0x07;
#[cfg(feature = "shot")]
const SHOT: u8 = 0x08;

/// A record of `tag` and one string; `None` when the normalised text is empty (dropped, 11.3).
fn text_record(tag: u8, text: String) -> Option<Vec<u8>> {
    (!text.is_empty()).then(|| {
        let mut p = Pre(vec![tag]);
        p.string(&text);
        p.0
    })
}

/// The Synopsis record, shared with the scene's `synopses`.
pub(crate) fn synopsis_record(s: &Synopsis, out: &mut Vec<Vec<u8>>) {
    out.extend(text_record(SYNOPSIS, normalise(&typed(&s.text), false)));
}

/// The kept records of one view element, in order (`SPEC.md` 11.3).
pub(crate) fn records(el: &Element, out: &mut Vec<Vec<u8>>) {
    match el {
        Element::Action(a) => {
            let text = normalise(&typed(&a.text), false);
            if !text.is_empty() {
                let mut p = Pre(vec![ACTION]);
                p.string(&text).bool(a.centered);
                out.push(p.0);
            }
            for inter in &a.interjections {
                records(inter, out);
            }
        }
        Element::Dialogue(b) => block_records(b, false, out),
        Element::DualDialogue(d) => {
            block_records(&d.left, true, out);
            block_records(&d.right, true, out);
        }
        Element::Transition(t) => {
            out.extend(text_record(TRANSITION, normalise(&typed(&t.text), false)));
        }
        Element::Lyrics(l) => {
            let lines: Vec<String> = l
                .lines
                .iter()
                .map(|t| normalise(&typed(t), false))
                .filter(|s| !s.is_empty())
                .collect();
            if !lines.is_empty() {
                let mut p = Pre(vec![LYRICS]);
                p.list(&lines);
                out.push(p.0);
            }
        }
        Element::Synopsis(s) => synopsis_record(s, out),
        #[cfg(feature = "shot")]
        Element::Shot(s) => out.extend(text_record(SHOT, normalise(&typed(&s.text), false))),
        // Sections, Page Breaks, Notes and Boneyards are not hashed in the story facet. Elements
        // added later (the enum is non-exhaustive) give no record until a scheme names them.
        _ => {}
    }
}

fn cue_record(cue: &Cue, dual: bool, out: &mut Vec<Vec<u8>>) {
    let name = normalise(&typed(&cue.name), true);
    if name.is_empty() {
        return;
    }
    let extensions: Vec<String> = cue
        .extensions
        .iter()
        .filter(|e| !e.is_contd())
        .map(|e| normalise(&typed(&e.text), true))
        .collect();
    let mut p = Pre(vec![CHARACTER]);
    p.string(&name).list(&extensions).bool(dual);
    out.push(p.0);
}

/// The Dialogue record of the open run of lines, then the records held back while it was open.
fn close_run(run: &mut Option<Vec<String>>, held: &mut Vec<Vec<u8>>, out: &mut Vec<Vec<u8>>) {
    if let Some(lines) = run.take() {
        out.extend(text_record(DIALOGUE, normalise(&lines.join(" "), false)));
    }
    out.append(held);
}

fn block_records(b: &DialogueBlock, dual: bool, out: &mut Vec<Vec<u8>>) {
    cue_record(&b.cue, dual, out);
    // While a run of Lines is open an interjection's records are held; they follow the run's
    // record. An interjection with no open run is emitted at once.
    let mut run: Option<Vec<String>> = None;
    let mut held = Vec::new();
    for item in &b.items {
        match item {
            DialogueItem::Line(t) => run.get_or_insert_with(Vec::new).push(typed(t)),
            DialogueItem::Parenthetical(t) => {
                close_run(&mut run, &mut held, out);
                // The text keeps its parentheses: strip one of each, then collapse again.
                let inner = normalise(&typed(t), false);
                let inner = inner.strip_prefix('(').unwrap_or(&inner);
                let inner = inner.strip_suffix(')').unwrap_or(inner);
                out.extend(text_record(PARENTHETICAL, normalise(inner, false)));
            }
            DialogueItem::Interjection(el) => {
                records(el, if run.is_some() { &mut held } else { out });
            }
        }
    }
    close_run(&mut run, &mut held, out);
}

impl Element {
    /// The scheme 2 hash of each record this element gives, in order.
    ///
    /// A Dialogue block gives its Character record and then its Dialogue and Parenthetical
    /// records; a dual pair gives the left block's records and then the right's; an Action gives
    /// its own record and then those of its Synopsis interjections. A record whose text is empty
    /// after normalisation is dropped. A Section, a Page Break, a Note and a Boneyard give no
    /// record, and neither does a Character with an empty name.
    pub fn hashes(&self) -> Vec<Digest> {
        let mut recs = Vec::new();
        records(self, &mut recs);
        record_hashes(&recs)
    }
}

/// `domain(name)`, then `parts` in order, hashed.
fn hash_of(name: &str, parts: &[&[u8]]) -> Digest {
    digest(&[domain(name).as_slice(), &parts.concat()].concat())
}

/// The element hash of each record.
fn record_hashes(recs: &[Vec<u8>]) -> Vec<Digest> {
    let domain = domain("element");
    recs.iter()
        .map(|r| digest(&[domain.as_slice(), r].concat()))
        .collect()
}

/// `u32(n)` and then the raw bytes of each digest.
fn counted(digests: &[Digest]) -> Vec<u8> {
    let mut p = Pre(Vec::new());
    p.u32(digests.len());
    for d in digests {
        p.0.extend_from_slice(&d.bytes);
    }
    p.0
}

impl Scene {
    /// The scheme 2 hash of each kept record of the scene, in order: its Synopses before and
    /// right after the heading, then its elements (`SPEC.md` 11.3). Sections are not hashed.
    pub fn element_hashes(&self) -> Vec<Digest> {
        let mut recs = Vec::new();
        for s in &self.synopses {
            synopsis_record(s, &mut recs);
        }
        for el in &self.elements {
            records(el, &mut recs);
        }
        record_hashes(&recs)
    }

    /// The scheme 2 scene hash: the heading (without its forcing `.` and scene number,
    /// normalised and upper-cased) and the element hashes of [`Scene::element_hashes`]. It does
    /// not depend on the scene's position, number or span.
    pub fn hash(&self) -> Digest {
        let heading = normalise(&typed(&self.heading.text), true);
        let mut p = Pre(Vec::new());
        p.string(&heading);
        hash_of("scene", &[&p.0, &counted(&self.element_hashes())])
    }

    /// The source-facet hash of the scene's span in `document`, or `None` for a scene built in
    /// memory, which has no span. `document` must be the one the scene was read from.
    ///
    /// # Panics
    ///
    /// If the scene's span is not a valid span of `document`, as [`Document::source_hash`]
    /// does. Passing a document other than the one the scene was read from is a caller error.
    pub fn source_hash(&self, document: &Document) -> Option<Digest> {
        self.span.map(|span| document.source_hash(span))
    }
}

impl Preamble {
    /// The scheme 2 preamble hash: the element hashes of its elements, in order.
    pub fn hash(&self) -> Digest {
        let mut recs = Vec::new();
        for el in &self.elements {
            records(el, &mut recs);
        }
        hash_of("preamble", &[&counted(&record_hashes(&recs))])
    }
}

impl Screenplay {
    /// The scheme 2 document hash: the preamble hash and the scene hashes, in order. The title
    /// page is not part of it.
    pub fn hash(&self) -> Digest {
        let scenes: Vec<Digest> = self.scenes.iter().map(Scene::hash).collect();
        hash_of(
            "document",
            &[&self.preamble.hash().bytes, &counted(&scenes)],
        )
    }

    /// The scheme 2 title-page hash: each entry's key (normalised, upper-cased) and value lines
    /// (normalised, case kept, empty lines dropped). Stray lines are not hashed, and a
    /// screenplay with no title page hashes as a title page with no entries.
    pub fn title_page_hash(&self) -> Digest {
        let entries = self.title_page.as_ref().map_or(&[][..], |t| &t.entries[..]);
        let mut p = Pre(Vec::new());
        p.u32(entries.len());
        for TitleEntry { key, values, .. } in entries {
            let lines: Vec<String> = values
                .iter()
                .map(|v| normalise(&typed(v), false))
                .filter(|v| !v.is_empty())
                .collect();
            p.string(&normalise(&typed(key), true)).list(&lines);
        }
        hash_of("title-page", &[&p.0])
    }

    /// The source-facet hash of the whole input of `document`, which must be the one this
    /// screenplay was read from.
    pub fn source_hash(&self, document: &Document) -> Digest {
        document.source_hash(Span::new(0, document.source().len()))
    }
}

impl Document {
    /// The source-facet hash of the exact bytes of `span`: it changes on any edit to them.
    ///
    /// # Panics
    ///
    /// If `span` is out of range, not on character boundaries of [`Document::source`], or has
    /// `start > end`.
    pub fn source_hash(&self, span: Span) -> Digest {
        let text = self
            .source()
            .get(span.start..span.end)
            .expect("a span inside the source, on character boundaries");
        hash_of("source", &[text.as_bytes()])
    }
}

impl Node<'_> {
    /// The source-facet hash of the bytes the node covers.
    pub fn source_hash(&self) -> Digest {
        hash_of("source", &[self.text().as_bytes()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;
    use crate::view::Text;

    /// The text of the first Action of `src`.
    fn action_text(src: &str) -> Text {
        match parse(src).screenplay().preamble.elements.remove(0) {
            Element::Action(a) => a.text,
            other => panic!("not an Action: {other:?}"),
        }
    }

    #[test]
    fn typed_text_rebuilds_marks() {
        let t = action_text(
            "*it* **bold** ***both*** _und_ \\*lit\\* x[[note]]y /* cut */z **a *b* c**",
        );
        assert_eq!(
            typed(&t),
            "*it* **bold** ***both*** _und_ \\*lit\\* xy z **a *b* c**"
        );
    }

    #[test]
    fn collapse_uses_white_space_only() {
        let white = "\u{9}\u{a}\u{b}\u{c}\u{d} \u{85}\u{a0}\u{1680}\u{2000}\u{2001}\u{2002}\u{2003}\u{2004}\u{2005}\u{2006}\u{2007}\u{2008}\u{2009}\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}";
        for c in white.chars() {
            assert_eq!(normalise(&format!("{c}a{c}{c}b{c}"), false), "a b", "{c:?}");
        }
        for c in [
            '\u{1c}', '\u{1d}', '\u{1e}', '\u{1f}', '\u{200b}', '\u{feff}', '\u{180e}',
        ] {
            let s = format!("a{c}b");
            assert_eq!(normalise(&s, false), s, "{c:?}");
        }
    }

    #[test]
    fn upper_case_is_full_mapping_then_nfc() {
        assert_eq!(normalise("stra\u{df}e", true), "STRASSE");
        assert_eq!(normalise("\u{149}", true), "\u{2bc}N");
        // U+0390 upper-cases to three code points, which NFC composes to two.
        assert_eq!(normalise("\u{390}", true), "\u{3aa}\u{301}");
        // Case is kept when `upper` is false, and the locale never matters.
        assert_eq!(normalise("Maria i", false), "Maria i");
        assert_eq!(normalise("maria i", true), "MARIA I");
    }

    #[test]
    fn hash2_31_domain_separation() {
        assert_eq!(domain("element"), b"fountain-syntax/element/2\0");
        let names = [
            "element",
            "scene",
            "preamble",
            "document",
            "title-page",
            "source",
        ];
        let digests: Vec<Digest> = names
            .iter()
            .map(|n| digest(&[domain(n), b"payload".to_vec()].concat()))
            .collect();
        for (i, a) in digests.iter().enumerate() {
            assert_eq!(a.scheme, HashScheme::V2);
            for b in &digests[i + 1..] {
                assert_ne!(a, b);
            }
        }
        // A scene with no elements and a preamble with no elements differ.
        let empty_scene = parse("INT. A - DAY\n").screenplay().scenes.remove(0);
        assert!(empty_scene.elements.is_empty());
        assert_ne!(empty_scene.hash(), Preamble::default().hash());
    }

    #[test]
    fn primitives_are_big_endian_and_length_prefixed() {
        let mut p = Pre(Vec::new());
        p.u32(1)
            .bool(true)
            .string("é")
            .list(&["a".into(), String::new()]);
        assert_eq!(
            p.0,
            [
                &[0, 0, 0, 1][..],
                &[1],
                &[0, 0, 0, 2, 0xc3, 0xa9],
                &[0, 0, 0, 2],
                &[0, 0, 0, 1, b'a'],
                &[0, 0, 0, 0],
            ]
            .concat()
        );
    }

    #[test]
    fn records_have_the_tag_layout() {
        let rec = |src: &str| {
            let mut out = Vec::new();
            for el in &parse(src).screenplay().preamble.elements {
                records(el, &mut out);
            }
            out
        };
        // Character: tag 2, name, the extension list without CONT'D, dual.
        let cue = rec("MARA (V.O.) (CONT'D)\nHi");
        assert_eq!(
            cue[0],
            [
                &[2][..],
                &[0, 0, 0, 4],
                b"MARA",
                &[0, 0, 0, 1, 0, 0, 0, 4],
                b"V.O.",
                &[0]
            ]
            .concat()
        );
        // Dual: both cues carry dual = 1.
        let dual = rec("MARA\nHi\n\nOTTO ^\nHey");
        assert_eq!(dual.len(), 4);
        assert_eq!(dual[0].last(), Some(&1));
        assert_eq!(dual[2].last(), Some(&1));
        // Lyrics: tag 6 and a list of the non-empty lines.
        assert_eq!(
            rec("~a\n~ \n~b")[0],
            [
                &[6][..],
                &[0, 0, 0, 2],
                &[0, 0, 0, 1],
                b"a",
                &[0, 0, 0, 1],
                b"b"
            ]
            .concat()
        );
        // Centered Action: tag 1, text without the marks, centered = 1.
        assert_eq!(
            rec("> hi <")[0],
            [&[1][..], &[0, 0, 0, 2], b"hi", &[1]].concat()
        );
        #[cfg(feature = "shot")]
        assert_eq!(rec("!!hi")[0], [&[8][..], &[0, 0, 0, 2], b"hi"].concat());
    }
}
