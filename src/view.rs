//! The typed screenplay view (SPEC 9): plain owned data with public fields, so a value can be built
//! in memory (with [`Scene::new`] for a scene), serialised and read back without a
//! [`Document`](crate::Document).
//!
//! A `span` is an optional back-reference to the source: `None` for a value that was built and not
//! parsed.

use crate::Span;

mod build;
pub(crate) mod inline;
mod note;
mod scheme1;

/// A whole screenplay: the title page, the elements before the first scene, and the scenes.
///
/// With the `serde` feature, a view reads back from JSON written by a build with other features,
/// with two limits. A `SceneNumber` without a `locked` field reads as not locked. Data that holds
/// an `Element::Shot` cannot be read by a build without the `shot` feature: the variant is unknown
/// to it, so reading fails with an error.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Screenplay {
    /// The title page, if the source has one.
    pub title_page: Option<TitlePage>,
    /// The elements before the first heading, except those attached to scene 1.
    pub preamble: Preamble,
    /// The scenes, in source order.
    pub scenes: Vec<Scene>,
}

/// The title page: `Key: value` entries in source order.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TitlePage {
    /// The entries, in source order.
    pub entries: Vec<TitleEntry>,
    /// Lines of the block that belong to no entry (OFF-17d).
    pub stray_lines: Vec<Text>,
    /// The bytes the title page covers.
    pub span: Option<Span>,
}

/// One title page entry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TitleEntry {
    /// The key, as written.
    pub key: Text,
    /// The value lines: the text after the colon, if any, then each continuation line.
    pub values: Vec<Text>,
    /// The bytes the entry covers.
    pub span: Option<Span>,
}

impl TitlePage {
    /// Every entry whose key equals `key`, ignoring Unicode case, in source order.
    pub fn get<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a TitleEntry> + 'a {
        self.entries.iter().filter(move |e| e.key_is(key))
    }

    /// The first entry whose key is any of `keys`, ignoring case.
    fn first_of(&self, keys: &[&str]) -> Option<&TitleEntry> {
        self.entries
            .iter()
            .find(|e| keys.iter().any(|k| e.key_is(k)))
    }

    /// The first `Title` entry.
    pub fn title(&self) -> Option<&TitleEntry> {
        self.first_of(&["Title"])
    }

    /// The first `Credit` entry.
    pub fn credit(&self) -> Option<&TitleEntry> {
        self.first_of(&["Credit"])
    }

    /// The first `Author` or `Authors` entry.
    pub fn author(&self) -> Option<&TitleEntry> {
        self.first_of(&["Author", "Authors"])
    }

    /// The first `Source` entry.
    pub fn source(&self) -> Option<&TitleEntry> {
        self.first_of(&["Source"])
    }

    /// The first `Draft date` entry.
    pub fn draft_date(&self) -> Option<&TitleEntry> {
        self.first_of(&["Draft date"])
    }

    /// The first `Contact` entry.
    pub fn contact(&self) -> Option<&TitleEntry> {
        self.first_of(&["Contact"])
    }

    /// The first `Notes` entry.
    pub fn notes(&self) -> Option<&TitleEntry> {
        self.first_of(&["Notes"])
    }

    /// The first `Copyright` entry.
    pub fn copyright(&self) -> Option<&TitleEntry> {
        self.first_of(&["Copyright"])
    }

    /// The first `Date` entry.
    pub fn date(&self) -> Option<&TitleEntry> {
        self.first_of(&["Date"])
    }
}

impl TitleEntry {
    fn key_is(&self, key: &str) -> bool {
        self.key.plain().to_lowercase() == key.to_lowercase()
    }
}

/// A scene: its heading and everything up to the next heading.
///
/// The struct has a private field (what hash scheme 1 counts as the scene, `SPEC.md` 11.7), so
/// outside the crate a scene is built with [`Scene::new`] and then its public fields are set.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Scene {
    /// The scene heading.
    pub heading: Heading,
    /// The scene number, if the heading has one.
    pub number: Option<SceneNumber>,
    /// The run of Sections right before the heading.
    pub sections: Vec<Section>,
    /// The Synopses right before or right after the heading.
    pub synopses: Vec<Synopsis>,
    /// Everything else in the scene, in source order.
    pub elements: Vec<Element>,
    /// From the first byte of the first attached Section or Synopsis, or of the heading, to the
    /// last byte of the last element.
    pub span: Option<Span>,
    /// Which Synopses hash scheme 1 counts in this scene; the default for a scene built in memory.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "HashScopeV1::is_default")
    )]
    hash_scope_v1: HashScopeV1,
}

/// The part of a scene's hash scope that the view places differently (`SPEC.md` 11.7). The scope
/// is `synopses[leading..]`, then `elements`, then `trailing`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct HashScopeV1 {
    /// How many of the scene's `synopses` came from before its heading: they belong to the
    /// previous scene's scope.
    leading: usize,
    /// The Synopses directly before the next heading: the view attaches them to the next scene,
    /// but scheme 1 keeps them in this one.
    trailing: Vec<Synopsis>,
}

impl HashScopeV1 {
    #[cfg(feature = "serde")]
    fn is_default(&self) -> bool {
        *self == Self::default()
    }
}

impl Scene {
    /// A scene built in memory: the heading, no number, no Sections, Synopses or elements, and no
    /// span. Set the public fields to fill it in.
    pub fn new(heading: Heading) -> Scene {
        Scene {
            heading,
            number: None,
            sections: Vec::new(),
            synopses: Vec::new(),
            elements: Vec::new(),
            span: None,
            hash_scope_v1: HashScopeV1::default(),
        }
    }

    /// The hash scope of scheme 1: the Synopses that count (after the `leading` ones), the
    /// elements, and the trailing Synopses. `leading` is clamped, so a scene edited after parsing
    /// cannot panic.
    pub(crate) fn hash_scope_v1(&self) -> (&[Synopsis], &[Element], &[Synopsis]) {
        let leading = self.hash_scope_v1.leading.min(self.synopses.len());
        (
            &self.synopses[leading..],
            &self.elements,
            &self.hash_scope_v1.trailing,
        )
    }
}

/// A scene heading.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Heading {
    /// The heading text.
    pub text: Text,
    /// True when the heading was forced with a leading `.`.
    pub forced: bool,
    /// The bytes the heading covers.
    pub span: Option<Span>,
}

/// A scene number such as the `12A` of `#12A#`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SceneNumber {
    /// The number as written, without the `#` marks.
    pub text: String,
    /// True for a locked number (`##12A#`).
    #[cfg(feature = "locked-scene-numbers")]
    #[cfg_attr(feature = "serde", serde(default))]
    pub locked: bool,
    /// The bytes the scene number covers.
    pub span: Option<Span>,
}

/// The elements before the first scene heading, apart from the title page and apart from the
/// Sections and Synopses directly before that heading, which belong to the first scene.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Preamble {
    /// The elements, in source order.
    pub elements: Vec<Element>,
}

/// One block of a scene. New variants may be added.
///
/// `Shot` exists only with the `shot` feature: serialised data that holds one cannot be read by a
/// build without that feature (unlike [`Document`](crate::Document), which is re-parsed from its source and so reads as a different tree instead of failing).
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Element {
    /// An Action paragraph or centered text.
    Action(Action),
    /// A Character cue with its dialogue.
    Dialogue(DialogueBlock),
    /// Two dialogue blocks side by side.
    DualDialogue(DualDialogue),
    /// A Transition.
    Transition(Transition),
    /// A run of Lyrics lines.
    Lyrics(Lyrics),
    /// A Section line.
    Section(Section),
    /// A Synopsis line.
    Synopsis(Synopsis),
    /// A Page Break.
    PageBreak(PageBreak),
    /// A whole-line Note.
    Note(Note),
    /// A whole-line Boneyard.
    Boneyard(Boneyard),
    /// A `!!` Shot line.
    #[cfg(feature = "shot")]
    Shot(Shot),
}

/// A `!!` Shot line.
#[cfg(feature = "shot")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Shot {
    /// The shot text, without the `!!`.
    pub text: Text,
}

/// An Action paragraph, or centered text.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Action {
    /// The paragraph text. Tabs are kept; see [`Action::display`].
    pub text: Text,
    /// True for centered text (`> text <`).
    pub centered: bool,
    /// True when the paragraph was forced with a leading `!`.
    pub forced: bool,
    /// A Section, Synopsis or Page Break line inside the paragraph (OFF-21b).
    pub interjections: Vec<Element>,
}

impl Action {
    /// The visible text for display: [`Text::plain`] with each tab replaced by four spaces
    /// (OFF-18c). The stored text keeps its tabs.
    pub fn display(&self) -> String {
        self.text.plain().replace('\t', "    ")
    }
}

/// A Transition such as `CUT TO:`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transition {
    /// The transition text, without the forcing `>`.
    pub text: Text,
    /// True when the transition was forced with a leading `>`.
    pub forced: bool,
}

/// A run of Lyrics lines.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Lyrics {
    /// The lines, without the leading `~`.
    pub lines: Vec<Text>,
}

/// A Character cue with its Parentheticals and Dialogue.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DialogueBlock {
    /// The Character cue.
    pub cue: Cue,
    /// The lines after the cue, in source order.
    pub items: Vec<DialogueItem>,
    /// True when the cue has a `^` but no dialogue block to pair with (OFF-11a).
    pub unpaired_caret: bool,
}

/// One line of a [`DialogueBlock`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DialogueItem {
    /// A Dialogue line.
    Line(Text),
    /// A Parenthetical line. The text keeps its parentheses.
    Parenthetical(Text),
    /// A Lyrics, Section, Synopsis, Page Break, Note or Boneyard line inside the block. Boxed to
    /// keep the enum small (`clippy::large_enum_variant`). In a dual-dialogue pair, lines skipped
    /// between the two halves (Sections, Synopses, Page Breaks, Notes and Boneyards) are appended to
    /// the end of `left.items` as Interjections.
    Interjection(Box<Element>),
}

/// A Character cue.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cue {
    /// The name, without extensions, the forcing `@` or the dual-dialogue caret.
    pub name: Text,
    /// The extensions such as `V.O.`, in source order.
    pub extensions: Vec<Extension>,
    /// True when the cue was forced with a leading `@`.
    pub forced: bool,
    /// The bytes the cue covers.
    pub span: Option<Span>,
}

/// A cue extension such as the `V.O.` of `MARA (V.O.)`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Extension {
    /// The text inside the parentheses.
    pub text: Text,
}

impl Extension {
    /// True when the trimmed text is `CONT'D` (straight or curly apostrophe), ignoring case.
    pub fn is_contd(&self) -> bool {
        let plain = self.text.plain().trim().to_lowercase();
        plain == "cont'd" || plain == "cont\u{2019}d"
    }
}

/// Two dialogue blocks side by side. Lines skipped between the two halves (Sections, Synopses,
/// Page Breaks, Notes and Boneyards) are appended to the end of `left.items` as Interjections.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DualDialogue {
    /// The first block.
    pub left: DialogueBlock,
    /// The second block, whose cue has the caret.
    pub right: DialogueBlock,
}

/// A Section line (`# Act One`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Section {
    /// The number of `#` marks.
    pub depth: usize,
    /// The section text, without the marks.
    pub text: Text,
}

/// A Synopsis line (`= text`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Synopsis {
    /// The synopsis text, without the `=`.
    pub text: Text,
}

/// A Page Break (`===`).
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PageBreak {
    /// The bytes the page break covers.
    pub span: Option<Span>,
}

/// A `[[ ]]` Note.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Note {
    /// The text between the brackets.
    pub text: String,
    /// The bytes the note covers, brackets included.
    pub span: Option<Span>,
}

/// A convention some tools use inside a Note. New variants may be added.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NoteConvention {
    /// A colour word alone, as typed.
    Colour(String),
    /// A marker, with an optional colour.
    Marker {
        /// The colour word as typed, if any.
        colour: Option<String>,
        /// The rest of the note.
        text: String,
    },
    /// A `tag: text` note.
    Tag {
        /// The tag.
        tag: String,
        /// The rest of the note.
        text: String,
    },
}

/// A `/* */` Boneyard.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Boneyard {
    /// The text between the delimiters.
    pub text: String,
    /// The bytes the boneyard covers, delimiters included.
    pub span: Option<Span>,
}

/// Inline content: plain runs, emphasis, escapes and hidden spans. Line breaks inside the text
/// are kept, as written, in the runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Text {
    /// The bytes the text covers: its runs and any Note or Boneyard attached to it. It can reach
    /// beyond the span of its parent element and include bytes that are in none of its runs, such
    /// as the line breaks around an attached atom. In `Title: a\n  [[n]]\n` the Note is a child of
    /// the title page, not of the entry, yet it joins the value's text.
    pub span: Option<Span>,
    /// The runs, in source order.
    pub runs: Vec<Run>,
}

/// A piece of inline [`Text`].
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Run {
    /// Plain text.
    Plain(String),
    /// Emphasised text, which may nest. Nesting is capped at 32 open spans per line; a deeper
    /// opener stays literal.
    Emphasis {
        /// True for bold.
        bold: bool,
        /// True for italic.
        italic: bool,
        /// True for underline.
        underline: bool,
        /// The emphasised runs.
        runs: Vec<Run>,
    },
    /// The character after a backslash.
    Escaped(char),
    /// An inline Note.
    Note(Note),
    /// An inline Boneyard.
    Boneyard(Boneyard),
}

impl Text {
    /// The visible text: the escaped characters, no sigils, no hidden spans, no emphasis marks.
    pub fn plain(&self) -> String {
        let mut out = String::new();
        push_plain(&self.runs, &mut out);
        out
    }
}

fn push_plain(runs: &[Run], out: &mut String) {
    for run in runs {
        match run {
            Run::Plain(s) => out.push_str(s),
            Run::Emphasis { runs, .. } => push_plain(runs, out),
            Run::Escaped(c) => out.push(*c),
            Run::Note(_) | Run::Boneyard(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(s: &str) -> Text {
        Text {
            span: None,
            runs: vec![Run::Plain(s.to_owned())],
        }
    }

    fn plains(synopses: &[Synopsis]) -> Vec<String> {
        synopses.iter().map(|s| s.text.plain()).collect()
    }

    fn heading(s: &str) -> Heading {
        Heading {
            text: text(s),
            forced: false,
            span: None,
        }
    }

    #[test]
    fn a_parsed_scene_records_which_synopses_its_hash_scope_keeps() {
        let src = "= before one\n\nINT. ONE - DAY\n\n= after one\n\nMara waits.\n\n\
                   # Act\n\n= before two\n\n## Beat\n\n= before two b\n\nINT. TWO - DAY\n\n= after two\n";
        let scenes = crate::parse(src).screenplay().scenes;
        let [one, two] = scenes.as_slice() else {
            panic!("two scenes");
        };
        // The view gives scene 1 the Synopsis before its heading and the one after it.
        assert_eq!(plains(&one.synopses), ["before one", "after one"]);
        assert_eq!(one.hash_scope_v1.leading, 1);
        assert_eq!(
            plains(&one.hash_scope_v1.trailing),
            ["before two", "before two b"]
        );
        let (synopses, elements, trailing) = one.hash_scope_v1();
        assert_eq!(plains(synopses), ["after one"]);
        assert_eq!(elements.len(), 1);
        assert_eq!(plains(trailing), ["before two", "before two b"]);

        // Scene 2 holds both Synopses before its heading, but they are not in its scope.
        assert_eq!(
            plains(&two.synopses),
            ["before two", "before two b", "after two"]
        );
        assert_eq!(two.hash_scope_v1.leading, 2);
        assert!(two.hash_scope_v1.trailing.is_empty());
        let (synopses, elements, trailing) = two.hash_scope_v1();
        assert_eq!(plains(synopses), ["after two"]);
        assert!(elements.is_empty() && trailing.is_empty());
    }

    #[test]
    fn scene_new_has_the_in_memory_defaults() {
        let mut scene = Scene::new(heading("INT. A - DAY"));
        assert_eq!(scene.heading, heading("INT. A - DAY"));
        assert!(scene.number.is_none() && scene.span.is_none());
        assert!(
            scene.sections.is_empty() && scene.synopses.is_empty() && scene.elements.is_empty()
        );
        assert_eq!(scene.hash_scope_v1, HashScopeV1::default());

        scene.synopses.push(Synopsis { text: text("beat") });
        scene
            .elements
            .push(Element::PageBreak(PageBreak { span: None }));
        let (synopses, elements, trailing) = scene.hash_scope_v1();
        assert_eq!(synopses, scene.synopses.as_slice());
        assert_eq!(elements, scene.elements.as_slice());
        assert!(trailing.is_empty());
    }

    #[test]
    fn a_leading_count_past_the_synopses_is_clamped() {
        let mut scene = Scene::new(heading("INT. A - DAY"));
        scene.synopses.push(Synopsis { text: text("beat") });
        scene.hash_scope_v1.leading = 5;
        assert!(scene.hash_scope_v1().0.is_empty());
    }

    fn entry(key: &str, value: &str) -> TitleEntry {
        TitleEntry {
            key: text(key),
            values: vec![text(value)],
            span: None,
        }
    }

    fn page(keys: &[(&str, &str)]) -> TitlePage {
        TitlePage {
            entries: keys.iter().map(|(k, v)| entry(k, v)).collect(),
            stray_lines: Vec::new(),
            span: None,
        }
    }

    fn contd(s: &str) -> bool {
        Extension { text: text(s) }.is_contd()
    }

    #[test]
    fn ext_contd_1_is_contd() {
        assert!(contd("CONT'D"));
        assert!(contd("cont'd"));
        assert!(contd("CONT\u{2019}D"));
        assert!(contd(" Cont'd "));
        assert!(!contd("V.O."));
        assert!(!contd("CONT'D 2"));
    }

    fn action(s: &str) -> Action {
        Action {
            text: text(s),
            centered: false,
            forced: false,
            interjections: Vec::new(),
        }
    }

    #[test]
    fn el_action_4_tab_expansion_is_display_only() {
        let a = action("\tHe waits.\tAlone.");
        assert_eq!(a.display(), "    He waits.    Alone.");
        assert_eq!(a.text.plain(), "\tHe waits.\tAlone.");
        assert_eq!(a.text, text("\tHe waits.\tAlone."));
    }

    #[test]
    fn off_18c_tabs_to_spaces_in_display_only() {
        let a = action("a\tb");
        assert_eq!(a.display(), "a    b");
        assert!(a.text.plain().contains('\t'));
    }

    #[test]
    fn ext_open_keys_2_typed_accessors() {
        let p = page(&[
            ("Title", "Big Fish"),
            ("Credit", "Written by"),
            ("Authors", "A and B"),
            ("Source", "a novel"),
            ("Draft date", "1 May"),
            ("Contact", "x"),
            ("Notes", "n"),
            ("Copyright", "c"),
            ("Date", "d"),
            ("Revision", "unknown key"),
        ]);
        let v = |e: Option<&TitleEntry>| e.map(|e| e.values[0].plain());
        assert_eq!(v(p.title()).as_deref(), Some("Big Fish"));
        assert_eq!(v(p.credit()).as_deref(), Some("Written by"));
        assert_eq!(v(p.author()).as_deref(), Some("A and B"));
        assert_eq!(v(p.source()).as_deref(), Some("a novel"));
        assert_eq!(v(p.draft_date()).as_deref(), Some("1 May"));
        assert_eq!(v(p.contact()).as_deref(), Some("x"));
        assert_eq!(v(p.notes()).as_deref(), Some("n"));
        assert_eq!(v(p.copyright()).as_deref(), Some("c"));
        assert_eq!(v(p.date()).as_deref(), Some("d"));
        assert_eq!(p.get("Revision").count(), 1);
        assert!(page(&[("Title", "t")]).author().is_none());
        // The first of `Author` / `Authors` in source order wins.
        let p = page(&[("Authors", "first"), ("Author", "second")]);
        assert_eq!(v(p.author()).as_deref(), Some("first"));
    }

    #[test]
    fn el_title_page_3_typed_access_ignores_case() {
        let p = page(&[
            ("TITLE", "a"),
            ("title", "b"),
            ("Straße", "c"),
            ("Draft Date", "d"),
        ]);
        let got: Vec<String> = p.get("Title").map(|e| e.values[0].plain()).collect();
        assert_eq!(got, ["a", "b"]);
        assert_eq!(p.get("STRAßE").count(), 1);
        assert_eq!(p.get("missing").count(), 0);
        let date = p.draft_date().map(|e| e.values[0].plain());
        assert_eq!(date.as_deref(), Some("d"));
    }

    #[test]
    fn text_plain_covers_every_run() {
        let note = Note {
            text: "hidden".to_owned(),
            span: None,
        };
        let boneyard = Boneyard {
            text: "gone".to_owned(),
            span: None,
        };
        let t = Text {
            span: None,
            runs: vec![
                Run::Plain("a ".to_owned()),
                Run::Emphasis {
                    bold: true,
                    italic: true,
                    underline: false,
                    runs: vec![
                        Run::Plain("b".to_owned()),
                        Run::Emphasis {
                            bold: false,
                            italic: false,
                            underline: true,
                            runs: vec![Run::Plain("c".to_owned())],
                        },
                    ],
                },
                Run::Escaped('*'),
                Run::Note(note),
                Run::Boneyard(boneyard),
                Run::Plain(" d".to_owned()),
            ],
        };
        assert_eq!(t.plain(), "a bc* d");
        assert_eq!(Text::default().plain(), "");
    }
}
