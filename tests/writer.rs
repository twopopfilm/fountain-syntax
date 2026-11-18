//! The writer (SPEC 10): `write` and `Display` give the source back byte for byte, and
//! `canonical` writes a view built in memory.

mod common;

use common::{corpus_files, parse_checked};
use fountain_syntax::{
    Action, Boneyard, Cue, DialogueBlock, DialogueItem, DualDialogue, Element, Extension, Heading,
    Lyrics, Note, PageBreak, Preamble, Run, Scene, SceneNumber, Screenplay, Section, Synopsis,
    Text, TitleEntry, TitlePage, Transition, canonical,
};

fn assert_round_trips(src: &str) {
    let doc = parse_checked(src);
    assert_eq!(doc.write(), src, "write changed {src:?}");
    assert_eq!(doc.to_string(), src, "Display changed {src:?}");
}

#[test]
fn wr_1_corpus_scripts_round_trip() {
    for (_, src) in corpus_files() {
        assert_round_trips(&src);
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

fn t(s: &str) -> Text {
    Text {
        span: None,
        runs: vec![Run::Plain(s.to_owned())],
    }
}

fn heading(s: &str) -> Heading {
    Heading {
        text: t(s),
        forced: false,
        span: None,
    }
}

fn action(s: &str) -> Element {
    Element::Action(Action {
        text: t(s),
        centered: false,
        forced: false,
        interjections: Vec::new(),
    })
}

fn block(name: &str, items: Vec<DialogueItem>) -> DialogueBlock {
    DialogueBlock {
        cue: Cue {
            name: t(name),
            extensions: Vec::new(),
            forced: false,
            span: None,
        },
        items,
        unpaired_caret: false,
    }
}

fn line(s: &str) -> DialogueItem {
    DialogueItem::Line(t(s))
}

fn note(s: &str) -> Note {
    Note {
        text: s.to_owned(),
        span: None,
    }
}

fn transition(s: &str) -> Element {
    Element::Transition(Transition {
        text: t(s),
        forced: false,
    })
}

fn section(depth: usize, s: &str) -> Section {
    Section { depth, text: t(s) }
}

fn synopsis(s: &str) -> Synopsis {
    Synopsis { text: t(s) }
}

/// A view built in memory with every element kind. No scene and not the preamble ends with a
/// Section or Synopsis, which the parser would move to the next scene.
fn built() -> Screenplay {
    let entry = |key: &str, values: &[&str]| TitleEntry {
        key: t(key),
        values: values.iter().map(|v| t(v)).collect(),
        span: None,
    };
    let mut mara = block(
        "MARA",
        vec![
            DialogueItem::Parenthetical(t("(quietly)")),
            line("Hi."),
            DialogueItem::Line(Text::default()),
            line("There."),
            DialogueItem::Interjection(Box::new(Element::Note(note("beat")))),
            line("Yo."),
        ],
    );
    mara.cue.extensions.push(Extension { text: t("V.O.") });
    let mut ben = block("BEN", vec![line("Alone.")]);
    ben.unpaired_caret = true;

    let mut one = Scene::new(heading("INT. HOUSE - DAY"));
    one.number = Some(SceneNumber {
        text: "1".to_owned(),
        #[cfg(feature = "locked-scene-numbers")]
        locked: false,
        span: None,
    });
    one.synopses.push(synopsis("After the heading."));
    one.elements = vec![
        Element::Section(section(2, "Beat")),
        Element::Action(Action {
            text: t("Mara runs.\n~not lyrics\nShe stops."),
            centered: false,
            forced: false,
            interjections: vec![Element::Synopsis(synopsis("mid"))],
        }),
        Element::Action(Action {
            text: Text {
                span: None,
                runs: vec![
                    Run::Emphasis {
                        bold: false,
                        italic: true,
                        underline: false,
                        runs: vec![
                            Run::Plain("a ".to_owned()),
                            Run::Note(note("n")),
                            Run::Plain(" b ".to_owned()),
                            Run::Boneyard(Boneyard {
                                text: "x".to_owned(),
                                span: None,
                            }),
                        ],
                    },
                    Run::Plain(" ".to_owned()),
                    Run::Escaped('*'),
                    Run::Plain("c".to_owned()),
                ],
            },
            centered: false,
            forced: false,
            interjections: Vec::new(),
        }),
        Element::Action(Action {
            text: t("THE END"),
            centered: true,
            forced: false,
            interjections: Vec::new(),
        }),
        Element::Dialogue(mara),
        Element::DualDialogue(DualDialogue {
            left: block("NORA", vec![line("Stop.")]),
            right: block("JUNE", vec![line("Wait.")]),
        }),
        action("Beat."),
        Element::Dialogue(ben),
        transition("CUT TO:"),
    ];

    let mut two = Scene::new(heading("EXT. ROOF - NIGHT"));
    two.sections.push(section(1, "Act Two"));
    two.elements = vec![
        Element::Lyrics(Lyrics {
            lines: vec![t("Down by the river"), t("we wait.")],
        }),
        Element::Synopsis(synopsis("In the elements.")),
        Element::PageBreak(PageBreak { span: None }),
        Element::Boneyard(Boneyard {
            text: " cut ".to_owned(),
            span: None,
        }),
        Element::Note(note("whole line")),
        #[cfg(feature = "shot")]
        Element::Shot(fountain_syntax::Shot {
            text: t("CLOSE ON the coin"),
        }),
        transition("FADE OUT."),
    ];

    Screenplay {
        title_page: Some(TitlePage {
            entries: vec![entry("Title", &["Big Fish"]), entry("Author", &["A", "B"])],
            stray_lines: vec![t("a stray line")],
            span: None,
        }),
        preamble: Preamble {
            elements: vec![
                Element::Section(section(1, "Act One")),
                action("Black. A hum."),
            ],
        },
        scenes: vec![one, two],
    }
}

fn views() -> Vec<Screenplay> {
    let mut views = vec![built()];
    for src in [
        include_str!("corpus/twopop/blue_rev2.fountain"),
        include_str!("corpus/twopop/edge_cases.fountain"),
        include_str!("corpus/twopop/two_scene.fountain"),
    ] {
        views.push(parse_checked(src).screenplay());
    }
    views
}

/// Asserts the views are equal apart from `span` and `forced`, which record how the text was
/// written, not what it says.
#[cfg(feature = "serde")]
fn same(a: &Screenplay, b: &Screenplay, text: &str) {
    fn strip(v: &mut serde_json::Value) {
        match v {
            serde_json::Value::Object(m) => {
                m.remove("span");
                m.remove("forced");
                m.values_mut().for_each(strip);
            }
            serde_json::Value::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let [mut a, mut b] = [a, b].map(|v| serde_json::to_value(v).expect("a view serialises"));
    strip(&mut a);
    strip(&mut b);
    assert_eq!(a, b, "canonical text:\n{text}");
}

#[cfg(feature = "serde")]
#[test]
fn wr_5_canonical_reparses_to_an_equal_view() {
    for view in views() {
        let text = canonical(&view);
        let back = parse_checked(&text).screenplay();
        same(&view, &back, &text);
    }
}

/// Without `serde`, the scene hashes still show that `canonical` keeps what the hash reads.
#[cfg(feature = "hash")]
#[test]
fn wr_5_canonical_keeps_the_scene_hashes() {
    for view in views() {
        let text = canonical(&view);
        let back = parse_checked(&text).screenplay();
        assert_eq!(
            view.scenes.len(),
            back.scenes.len(),
            "canonical text:\n{text}"
        );
        for (a, b) in view.scenes.iter().zip(&back.scenes) {
            assert_eq!(a.hash_v1(), b.hash_v1(), "canonical text:\n{text}");
            assert_eq!(a.hash(), b.hash(), "canonical text:\n{text}");
        }
    }
}

#[test]
fn wr_6_canonical_is_stable() {
    for view in views() {
        let c = canonical(&view);
        assert_eq!(canonical(&parse_checked(&c).screenplay()), c);
    }
}

#[test]
fn wr_5_an_empty_title_page_is_left_out() {
    let view = Screenplay {
        title_page: Some(TitlePage {
            entries: Vec::new(),
            stray_lines: Vec::new(),
            span: None,
        }),
        preamble: Preamble::default(),
        scenes: vec![Scene::new(heading("INT. A"))],
    };
    let c = canonical(&view);
    assert_eq!(c, "INT. A\n");
    // The re-parse has no title page, so only the text is compared (WR-6).
    let back = parse_checked(&c).screenplay();
    assert!(back.title_page.is_none());
    assert_eq!(canonical(&back), c);
}

#[cfg(feature = "locked-scene-numbers")]
#[test]
fn wr_8_a_locked_scene_number_is_written_with_two_marks() {
    let mut scene = Scene::new(heading("INT. A"));
    scene.number = Some(SceneNumber {
        text: "12A".to_owned(),
        locked: true,
        span: None,
    });
    let c = canonical(&Screenplay {
        title_page: None,
        preamble: Preamble::default(),
        scenes: vec![scene],
    });
    assert_eq!(c, "INT. A ##12A#\n");
    let back = parse_checked(&c).screenplay();
    let n = back.scenes[0].number.as_ref().expect("the number survives");
    assert_eq!((n.text.as_str(), n.locked), ("12A", true));
    assert_eq!(back.scenes[0].heading.text.plain(), "INT. A");
}

#[test]
fn wr_8_sigil_only_where_needed() {
    let scene = |h: &str, elements: Vec<Element>| {
        let mut scene = Scene::new(heading(h));
        scene.elements = elements;
        canonical(&Screenplay {
            title_page: None,
            preamble: Preamble::default(),
            scenes: vec![scene],
        })
    };
    let preamble = |elements: Vec<Element>, title_page: Option<TitlePage>| {
        canonical(&Screenplay {
            title_page,
            preamble: Preamble { elements },
            scenes: Vec::new(),
        })
    };
    let said = |name: &str| Element::Dialogue(block(name, vec![line("Hi.")]));

    assert_eq!(scene("INT. A", vec![]), "INT. A\n");
    assert_eq!(scene("ROOF", vec![]), ".ROOF\n");
    assert_eq!(
        scene("INT. A", vec![transition("CUT TO:")]),
        "INT. A\n\nCUT TO:\n"
    );
    assert_eq!(
        scene("INT. A", vec![transition("FADE OUT.")]),
        "INT. A\n\n>FADE OUT.\n"
    );
    assert_eq!(scene("INT. A", vec![said("MARA")]), "INT. A\n\nMARA\nHi.\n");
    assert_eq!(
        scene("INT. A", vec![said("mara")]),
        "INT. A\n\n@mara\nHi.\n"
    );
    assert_eq!(
        scene("INT. A", vec![action("Mara waits.")]),
        "INT. A\n\nMara waits.\n"
    );
    for (text, written) in [
        ("INT. NOT A HEADING", "!INT. NOT A HEADING"),
        ("CUT TO:", "!CUT TO:"),
        ("MARA\nruns", "!MARA\nruns"),
    ] {
        assert_eq!(
            scene("INT. A", vec![action(text)]),
            format!("INT. A\n\n{written}\n")
        );
    }

    // With no title page, a first line that reads as a title entry needs the sigil, even after a
    // Section; after a title page it does not.
    assert_eq!(preamble(vec![action("Time: 5pm")], None), "!Time: 5pm\n");
    assert_eq!(
        preamble(
            vec![Element::Section(section(1, "Act")), action("Time: 5pm")],
            None
        ),
        "# Act\n\n!Time: 5pm\n"
    );
    let page = TitlePage {
        entries: vec![TitleEntry {
            key: t("Title"),
            values: vec![t("T")],
            span: None,
        }],
        stray_lines: Vec::new(),
        span: None,
    };
    assert_eq!(
        preamble(vec![action("Time: 5pm")], Some(page)),
        "Title: T\n\nTime: 5pm\n"
    );
}

/// The cue extension texts of every Dialogue block, in source order.
fn cue_extensions(view: &Screenplay) -> Vec<Vec<String>> {
    let elements = view
        .preamble
        .elements
        .iter()
        .chain(view.scenes.iter().flat_map(|s| &s.elements));
    elements
        .filter_map(|e| match e {
            Element::Dialogue(b) => Some(b.cue.extensions.iter().map(|x| x.text.plain()).collect()),
            _ => None,
        })
        .collect()
}

/// EXT-contd-2
#[test]
fn ext_contd_2() {
    // The extension must be parsed at all, or the strip check below passes vacuously.
    let contd = parse_checked("\nMARA (CONT'D)\nHi.\n").screenplay();
    assert_eq!(cue_extensions(&contd), [["CONT'D"]]);
    for src in [
        "\nMARA\nHi.\n",
        "\nMARA (CONT'D)\nHi.\n",
        "\nMARA (cont'd)\nHi.\n",
        "\nMARA (CONT\u{2019}D)\nHi.\n",
        "\nMARA\nHi.\n\nOTTO\nHey.\n\nMARA\nBye.\n",
    ] {
        assert_round_trips(src);
        let view = parse_checked(src).screenplay();
        let back = parse_checked(&canonical(&view)).screenplay();
        assert_eq!(
            cue_extensions(&view),
            cue_extensions(&back),
            "canonical changed the cue extensions of {src:?}"
        );
    }
}

/// EXT-locked-scene-numbers-4
#[cfg(feature = "locked-scene-numbers")]
#[test]
fn ext_locked_scene_numbers_4() {
    let numbered = "INT. A ##12A#\n\nMara waits.\n\nEXT. B #3#\n\nx\n";
    assert_round_trips(numbered);
    let view = parse_checked(numbered).screenplay();
    let numbers: Vec<_> = view
        .scenes
        .iter()
        .map(|s| s.number.as_ref().map(|n| (n.text.as_str(), n.locked)))
        .collect();
    assert_eq!(numbers, [Some(("12A", true)), Some(("3", false))]);

    #[cfg(feature = "hash")]
    {
        let plain = parse_checked("INT. A\n\nMara waits.\n\nEXT. B\n\nx\n").screenplay();
        assert_eq!(view.scenes.len(), plain.scenes.len());
        for (a, b) in view.scenes.iter().zip(&plain.scenes) {
            assert_eq!(a.hash(), b.hash());
            assert_eq!(a.hash_v1(), b.hash_v1());
        }
        assert_eq!(view.hash(), plain.hash());
    }
}
