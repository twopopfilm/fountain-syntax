#![cfg(feature = "hash")]
//! Scheme 2 scene, preamble, document, title-page and source hashes (`SPEC.md` 11.3 to 11.6 and
//! the HASH2 table of 11.9), read through `parse(..).screenplay()`.

use fountain_syntax::{
    Action, Cue, DialogueBlock, DialogueItem, Digest, Document, Element, HASH_SCHEME_CURRENT,
    HashScheme, Heading, Lyrics, Preamble, Run, Scene, Screenplay, Span, Synopsis, Text,
    TitleEntry, TitlePage, parse,
};
use sha2::{Digest as _, Sha256};

const H: &str = "INT. KITCHEN - DAY";

/// The scene hash, then its element hashes, as text.
type Sig = (String, Vec<String>);

fn texts(ds: &[Digest]) -> Vec<String> {
    ds.iter().map(Digest::to_string).collect()
}

fn scene(src: &str) -> Scene {
    parse(src).screenplay().scenes.remove(0)
}

fn sig(src: &str) -> Sig {
    let s = scene(src);
    (s.hash().to_string(), texts(&s.element_hashes()))
}

/// The source of one scene whose body is `body`.
fn body(b: &str) -> String {
    format!("{H}\n\n{b}\n")
}

fn eh(b: &str) -> Vec<String> {
    sig(&body(b)).1
}

fn dh(src: &str) -> String {
    parse(src).screenplay().hash().to_string()
}

fn ph(src: &str) -> String {
    parse(src).screenplay().preamble.hash().to_string()
}

fn th(src: &str) -> String {
    parse(src).screenplay().title_page_hash().to_string()
}

/// Every scene hash, in order.
fn scene_hashes(src: &str) -> Vec<String> {
    let sp = parse(src).screenplay();
    sp.scenes.iter().map(|s| s.hash().to_string()).collect()
}

/// The scene bodies give the same scene hash, element hashes and document hash.
fn same(a: &str, b: &str) {
    let (sa, sb) = (body(a), body(b));
    assert_eq!(sig(&sa), sig(&sb), "{a:?} against {b:?}");
    assert_eq!(dh(&sa), dh(&sb), "{a:?} against {b:?}");
}

/// The scene bodies give different scene hashes and different document hashes.
fn differ(a: &str, b: &str) {
    let (sa, sb) = (body(a), body(b));
    assert_ne!(sig(&sa).0, sig(&sb).0, "{a:?} against {b:?}");
    assert_ne!(dh(&sa), dh(&sb), "{a:?} against {b:?}");
}

/// The same, for whole sources.
fn same_src(a: &str, b: &str) {
    assert_eq!(sig(a), sig(b), "{a:?} against {b:?}");
    assert_eq!(dh(a), dh(b), "{a:?} against {b:?}");
}

fn plain(s: &str) -> Text {
    Text {
        span: None,
        runs: vec![Run::Plain(s.to_string())],
    }
}

/// `domain(name) || payload`, hashed, as the spec writes it out.
fn sha(name: &str, payload: &[u8]) -> String {
    let bytes = [format!("fountain-syntax/{name}/2\0").as_bytes(), payload].concat();
    Digest {
        scheme: HashScheme::V2,
        bytes: Sha256::digest(bytes).into(),
    }
    .to_string()
}

fn u32be(n: usize) -> Vec<u8> {
    u32::try_from(n).unwrap().to_be_bytes().to_vec()
}

// ---- known answers (SPEC 11.5) ----

#[test]
fn hash2_40_heading_only() {
    let expected = "fs2:59bb346594a6fee2987e19850929c14dfc8ceb86ee6293e8eb24a72ebd617248";
    let s = scene(&format!("{H}\n"));
    assert_eq!(s.hash().to_string(), expected);
    assert!(s.element_hashes().is_empty());
    // The pre-image is 50 bytes: 24 of domain, 4 + 18 of heading, 4 of count.
    let pre = [u32be(18), H.as_bytes().to_vec(), u32be(0)].concat();
    assert_eq!(pre.len() + 24, 50);
    assert_eq!(sha("scene", &pre), expected);
}

#[test]
fn hash2_41_scene() {
    let s = scene(&body("Mara fills the kettle."));
    assert_eq!(
        texts(&s.element_hashes()),
        ["fs2:1cd74cd5159cd81b65c689eff8d55d8dc7bfa6de082dbb341873240f6595850b"]
    );
    assert_eq!(
        s.hash().to_string(),
        "fs2:39ad14125a7c97078c25a78852cc36793e0fd890ce68c33491a1a08fa3bf9c78"
    );
}

#[test]
fn hash2_42_scene() {
    let s = scene(&format!("{H}\n\nMARA (V.O.) (CONT'D)\n(quietly)\nTea?\n"));
    assert_eq!(
        texts(&s.element_hashes()),
        [
            "fs2:1a556f7deab4f5fa5628f2e5d969f50285b3c00f75fe18d6a8dd00f26646af95",
            "fs2:07e731a7acbe2ca8fa63f9a0a9e38dfb1abb4af25c2287e3e715e0bfe8ab45a3",
            "fs2:b37c03f30bd33f9849e35026d903755b6bd92eacdd9bc69a90b303295bc651c1",
        ]
    );
    assert_eq!(
        s.hash().to_string(),
        "fs2:1605f74bdad918ae3f826b7c5ea6f8b36b59466c10f2dc426514262b359d62aa"
    );
}

const TEA: &str = "Title: Tea Time\n\nINT. KITCHEN - DAY\n\nMara fills the kettle.\n";

#[test]
fn hash2_45_document() {
    let doc = parse(TEA);
    let sp = doc.screenplay();
    assert_eq!(TEA.len(), 60);
    assert_eq!(
        sp.preamble.hash().to_string(),
        "fs2:354e078bdd8af223f493f82eec629de6f22e72660aef1936ebf92739fd443cac"
    );
    assert_eq!(
        sp.hash().to_string(),
        "fs2:2b182de81221644c685c35b4917275421b0c85a053d77b012275aceaf641987f"
    );
    assert_eq!(
        sp.title_page_hash().to_string(),
        "fs2:9ea6372e0b7d474e70cd1a48cda79753a36058c111d4792a4c4d21700b0d962c"
    );
    let whole = "fs2:84f98daa89e4fd67b4dc773e449db8f969237b077469878eb521229d96634482";
    assert_eq!(sp.source_hash(&doc).to_string(), whole);
    assert_eq!(doc.source_hash(Span::new(0, 60)).to_string(), whole);
    assert_eq!(doc.root().source_hash().to_string(), whole);

    let span = sp.scenes[0].span.unwrap();
    assert_eq!(span.len(), 42);
    let part = "fs2:9e15d23ca2039b27e7ed261a564741edddedecd07a07db5139b77b8100f93a00";
    assert_eq!(sp.scenes[0].source_hash(&doc).unwrap().to_string(), part);
    assert_eq!(doc.source_hash(span).to_string(), part);

    // The same values, built from the layout of 11.4 by hand.
    let preamble = sp.preamble.hash().bytes.to_vec();
    assert_eq!(preamble, raw(&sp.preamble.hash()));
    assert_eq!(sha("preamble", &u32be(0)), sp.preamble.hash().to_string());
    let document = [preamble, u32be(1), raw(&sp.scenes[0].hash())].concat();
    assert_eq!(sha("document", &document), sp.hash().to_string());
    let entry = [
        u32be(5),
        b"TITLE".to_vec(),
        u32be(1),
        u32be(8),
        b"Tea Time".to_vec(),
    ]
    .concat();
    assert_eq!(
        sha("title-page", &[u32be(1), entry].concat()),
        sp.title_page_hash().to_string()
    );
    assert_eq!(sha("source", TEA.as_bytes()), whole);
}

// ---- G1: what does not change a hash ----

#[test]
fn hash2_1_spaces_and_tabs() {
    let a = "Mara fills the kettle.\n\nMARA\n(quietly)\nTea? Now.\n\nCUT TO:\n\n~La la";
    let b = "  Mara \t fills   the kettle.  \n\nMARA  \n(\t quietly \t)\n \tTea?\t Now. \t\n\nCUT TO:\n\n~ La \t la ";
    same(a, b);
    same("MARA\n(quietly)\nHi", "MARA\n(   quietly   )\nHi");
    // The heading too.
    same_src(&format!("{H}\n\nx\n"), "INT.\t KITCHEN   -  DAY  \n\nx\n");
}

#[test]
fn hash2_2_line_endings() {
    let lf = "Title: T\n\nINT. KITCHEN - DAY\n\nMara waits.\nShe sits.\n\nMARA\nHi\nthere\n";
    let crlf = lf.replace('\n', "\r\n");
    let cr = lf.replace('\n', "\r");
    for other in [&crlf, &cr] {
        same_src(lf, other);
        assert_eq!(th(lf), th(other));
    }
}

#[test]
fn hash2_3_rewrapping() {
    same(
        "Mara waits by the\nwindow and then sits.",
        "Mara waits by the window\nand then sits.",
    );
    same(
        "Mara waits by the window and then sits.",
        "Mara\nwaits by\nthe window and\nthen sits.",
    );
    same("MARA\nHello there\nfriend", "MARA\nHello\nthere friend");
    same("MARA\nHello there friend", "MARA\nHello\nthere\nfriend");
    // Two `~` lines are two entries however they are spaced.
    same("~a\n~b", "~  a  \n~\tb");
    differ("~a\n~b", "~a b");
}

#[test]
fn hash2_4_blank_lines() {
    same("a\n\nb\n\nMARA\nHi", "a\n\n\n\nb\n\n\nMARA\nHi");
    same_src(
        &format!("Title: T\n\n{H}\n\nx\n\nINT. B - DAY\n\ny\n"),
        &format!("Title: T\n\n\n\n{H}\n\n\nx\n\n\n\nINT. B - DAY\n\n\ny\n"),
    );
}

#[test]
fn hash2_5_hidden_text() {
    let base = "Mara waits.\n\nMARA\nHello there\n\nCUT TO:";
    // An inline Note and Boneyard.
    same(
        base,
        "Mara [[a note]] waits.\n\nMARA\nHello /* cut */ there\n\nCUT TO:",
    );
    // A whole-line Note and Boneyard inside an Action paragraph.
    same(
        "Mara waits.\nShe sits.",
        "Mara waits.\n[[a note]]\nShe sits.",
    );
    same(
        "Mara waits.\nShe sits.",
        "Mara waits.\n/* cut */\nShe sits.",
    );
    // A Boneyard over two lines, inside an Action paragraph: `Mara waits.`
    same(
        "Mara waits.\nShe sits.",
        "Mara waits.\n/* two\nlines */\nShe sits.",
    );
    same("Mara waits.", "Mara /* aside\nmore */ waits.");
    same("Mara waits.", "Mara [[an aside\nmore]] waits.");
    // A Section and a Page Break between elements.
    same(
        base,
        "# Act\n\nMara waits.\n\n===\n\nMARA\nHello there\n\n## Beat\n\nCUT TO:",
    );
    // A whole-line Note or Boneyard between two Dialogue lines neither adds a record nor splits
    // the run.
    for hidden in ["[[a note]]", "/* cut */", "[[a\nnote]]", "/* two\nlines */"] {
        let split = format!("MARA\nHello\n{hidden}\nthere");
        same("MARA\nHello there", &split);
        assert_eq!(eh(&split).len(), 2, "{split:?}");
    }
    // Hidden text inside the heading and the cue.
    same_src(
        &format!("{H}\n\nx\n"),
        "INT. KITCHEN [[n]] - /* c */ DAY\n\nx\n",
    );
    same("@Mara\nHi", "@Mara [[n]]\nHi");
}

#[test]
fn hash2_6_scene_numbers() {
    let plain_src = format!("{H}\n\nMara waits.\n");
    for numbered in [
        format!("{H} #12A#\n\nMara waits.\n"),
        format!("{H} #7#\n\nMara waits.\n"),
    ] {
        same_src(&plain_src, &numbered);
    }
    same_src(&format!("{H} #1#\n\nx\n"), &format!("{H} #2#\n\nx\n"));
    let s = scene(&format!("{H} #12A#\n\nx\n"));
    assert_eq!(s.number.as_ref().map(|n| n.text.as_str()), Some("12A"));
}

#[cfg(feature = "locked-scene-numbers")]
#[test]
fn hash2_6_locked_scene_numbers() {
    let plain_src = format!("{H}\n\nMara waits.\n");
    let locked = format!("{H} ##12A#\n\nMara waits.\n");
    assert!(scene(&locked).number.as_ref().is_some_and(|n| n.locked));
    same_src(&plain_src, &locked);
    same_src(&format!("{H} #12A#\n\nx\n"), &format!("{H} ##12A#\n\nx\n"));
    same_src(&format!("{H} ##12A#\n\nx\n"), &format!("{H} ##99#\n\nx\n"));
}

#[test]
fn hash2_7_forcing_sigils() {
    same_src(".INT. KITCHEN - DAY\n\nx\n", &format!("{H}\n\nx\n"));
    same("!Mara waits.", "Mara waits.");
    same("@Mara\nHi", "MARA\nHi");
    same("@mara\nHi", "MARA\nHi");
    same(">CUT TO:", "CUT TO:");
    same("> CUT TO:", "CUT TO:");
}

#[test]
fn hash2_8_contd() {
    let bare = "MARA\nHi";
    for ext in ["(CONT'D)", "(cont'd)", "(CONT\u{2019}D)", "(Cont'd)"] {
        same(bare, &format!("MARA {ext}\nHi"));
    }
    let vo = "MARA (V.O.)\nHi";
    same(vo, "MARA (V.O.) (CONT'D)\nHi");
    same(vo, "MARA (CONT'D) (V.O.)\nHi");
    same(vo, "MARA (v.o.) (cont\u{2019}d)\nHi");
}

#[test]
fn hash2_9_case() {
    same_src("int. kitchen - day\n\nx\n", &format!("{H}\n\nx\n"));
    same_src(".Kitchen\n\nx\n", ".KITCHEN\n\nx\n");
    same("@Mara (v.o.) (Filtered)\nHi", "MARA (V.O.) (FILTERED)\nHi");
}

#[test]
fn hash2_10_nfc_and_nfd() {
    same_src(
        "INT. CAF\u{c9} - DAY\n\nx\n",
        "INT. CAFE\u{301} - DAY\n\nx\n",
    );
    same("Caf\u{e9} au lait", "Cafe\u{301} au lait");
    same("@Zo\u{eb}\nHi", "@Zoe\u{308}\nHi");
    same("@MARA (\u{e9}t\u{e9})\nHi", "@MARA (e\u{301}te\u{301})\nHi");
    same("~caf\u{e9}\n~bar", "~cafe\u{301}\n~bar");
    same("MARA\nCaf\u{e9}", "MARA\nCafe\u{301}");
    // A title-page value; and a key, which the parser takes only with alphanumerics (SPEC 4.1),
    // so an NFD key is built in memory.
    assert_eq!(
        th("K: caf\u{e9}\n\nINT. A\n"),
        th("K: cafe\u{301}\n\nINT. A\n")
    );
    let keyed = |key: &str| Screenplay {
        title_page: Some(TitlePage {
            entries: vec![TitleEntry {
                key: plain(key),
                values: vec![plain("x")],
                span: None,
            }],
            stray_lines: Vec::new(),
            span: None,
        }),
        preamble: Preamble::default(),
        scenes: Vec::new(),
    };
    assert_eq!(
        keyed("Caf\u{e9}").title_page_hash(),
        keyed("Cafe\u{301}").title_page_hash()
    );
    assert_eq!(
        keyed("Caf\u{e9}").title_page_hash(),
        th("Caf\u{e9}: x\n\nINT. A\n").parse::<Digest>().unwrap()
    );
}

#[test]
fn hash2_11_caret_spacing() {
    let base = "MARA\nHi\n\nOTTO ^\nHey";
    same(base, "MARA\nHi\n\nOTTO^\nHey");
    same(base, "MARA\nHi\n\nOTTO ^ \nHey");
    same(base, "MARA\nHi\n\nOTTO  ^\nHey");
}

#[test]
fn hash2_12_title_page_is_in_no_story_hash() {
    let story = format!("{H}\n\nx\n");
    let with = |title: &str| format!("{title}{story}");
    let reference = with("");
    for src in [
        with("Title: T\n\n"),
        with("Title: Other\nAuthor: A\n\n"),
        with("Notes: x\n  y\n\n"),
    ] {
        same_src(&reference, &src);
        assert_eq!(ph(&reference), ph(&src));
        assert_eq!(scene_hashes(&reference), scene_hashes(&src));
    }
    assert_ne!(th(&with("Title: T\n\n")), th(&with("Title: Other\n\n")));
}

// ---- G2: what changes a hash ----

/// The parts of a scene whose records are, in order: Action, Character, Parenthetical,
/// Dialogue, Transition, Lyrics, Synopsis.
fn parts(heading: &str, p: [&str; 7]) -> String {
    format!(
        "{heading}\n\n{}\n\n{}\n({})\n{}\n\n{}\n\n~{}\n\n= {}\n",
        p[0], p[1], p[2], p[3], p[4], p[5], p[6]
    )
}

const BASE: [&str; 7] = [
    "Mara fills the kettle.",
    "MARA",
    "quietly",
    "Tea?",
    "CUT TO:",
    "La la",
    "A beat",
];

#[test]
fn hash2_13_word_change_changes_only_hashes_above() {
    let base = parts(H, BASE);
    let b = scene(&base);
    assert_eq!(b.element_hashes().len(), 7, "{:?}", b.elements);
    // The cue is a record too, so a different name is a different word.
    let words = [
        "Mara fills the teapot.",
        "OTTO",
        "loudly",
        "Coffee?",
        "SMASH CUT TO:",
        "Lo lo",
        "Another beat",
    ];
    for (i, word) in words.iter().enumerate() {
        let mut p = BASE;
        p[i] = word;
        let c = scene(&parts(H, p));
        let (be, ce) = (b.element_hashes(), c.element_hashes());
        assert_eq!(ce.len(), 7);
        for j in 0..7 {
            assert_eq!(be[j] == ce[j], i != j, "record {i} against {j}");
        }
        assert_ne!(b.hash(), c.hash(), "record {i}");
        assert_ne!(dh(&base), dh(&parts(H, p)), "record {i}");
    }
    // The heading: no element hash changes.
    let c = scene(&parts("INT. GARDEN - DAY", BASE));
    assert_eq!(b.element_hashes(), c.element_hashes());
    assert_ne!(b.hash(), c.hash());
    assert_ne!(dh(&base), dh(&parts("INT. GARDEN - DAY", BASE)));
}

#[cfg(feature = "shot")]
#[test]
fn hash2_13_shot_word_change() {
    assert_ne!(eh("!!CLOSE ON THE KETTLE"), eh("!!CLOSE ON THE CUP"));
    differ("!!CLOSE ON THE KETTLE", "!!CLOSE ON THE CUP");
}

#[test]
fn hash2_14_case_of_text() {
    differ("Mara fills the kettle.", "mara fills the kettle.");
    differ("MARA\nTea?", "MARA\ntea?");
}

#[test]
fn hash2_15_emphasis_and_escapes() {
    differ("Mara waits.", "*Mara* waits.");
    differ("Mara waits.", "_Mara_ waits.");
    differ("*Mara* waits.", "**Mara** waits.");
    differ("a*b", "a\\*b");
    differ("MARA\nHi", "MARA\n*Hi*");
    differ("~la", "~*la*");
}

#[test]
fn hash2_16_kind_changes_with_the_same_words() {
    // (one kind, the other kind, the index of the record in each that has the same words)
    let pairs = [
        ("x y", "> x y", 0, 0),
        ("MARA\nTea?", "Tea?", 1, 0),
        ("x", "y\n\n= x", 0, 1),
        ("quietly", "MARA\n(quietly)\nTea?", 0, 1),
    ];
    for (a, b, ia, ib) in pairs {
        assert_ne!(eh(a)[ia], eh(b)[ib], "{a:?} against {b:?}");
        differ(a, b);
    }
}

#[test]
fn hash2_17_cue_extensions() {
    let bare = eh("MARA\nHi");
    let vo = eh("MARA (V.O.)\nHi");
    let os = eh("MARA (O.S.)\nHi");
    let both = eh("MARA (V.O.) (FILTERED)\nHi");
    let swapped = eh("MARA (FILTERED) (V.O.)\nHi");
    let all = [&bare, &vo, &os, &both, &swapped];
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a[0], b[0]);
            assert_eq!(a[1], b[1], "the Dialogue record does not change");
        }
    }
    differ("MARA\nHi", "MARA (V.O.)\nHi");
    differ("MARA (V.O.)\nHi", "MARA (V.O.) (FILTERED)\nHi");
    differ("MARA (V.O.) (FILTERED)\nHi", "MARA (FILTERED) (V.O.)\nHi");
}

#[test]
fn hash2_18_dual_flag() {
    let single = "MARA\nHi\n\nOTTO\nHey";
    let dual = "MARA\nHi\n\nOTTO ^\nHey";
    let (s, d) = (eh(single), eh(dual));
    assert_eq!(s.len(), 4);
    assert_ne!(s[0], d[0], "the left cue");
    assert_ne!(s[2], d[2], "the right cue");
    assert_eq!((&s[1], &s[3]), (&d[1], &d[3]));
    differ(single, dual);
    // A caret with no partner changes nothing.
    same("OTTO ^\nHey", "OTTO\nHey");
    // Unpairing changes it back: an Action between the blocks leaves the caret cue with no
    // partner, so both cues hash as in the single form and differ from the paired form.
    let unpaired = eh(&dual.replace("\n\nOTTO", "\n\nA beat.\n\nOTTO"));
    assert_eq!(unpaired.len(), 5);
    assert_eq!(unpaired[0], s[0], "the left cue falls back");
    assert_eq!(unpaired[3], s[2], "the right cue falls back");
    assert_ne!(unpaired[0], d[0]);
    assert_ne!(unpaired[3], d[2]);
}

#[test]
fn hash2_19_centered_flag() {
    differ("> FIN <", "FIN");
    differ("> FIN <", "> FIN");
    assert_ne!(eh("> FIN <"), eh("FIN"));
}

#[test]
fn hash2_20_lyrics() {
    differ("~a b", "~a\n~b");
    differ("~a\n\n~b", "~a\n~b");
    same("~a\n~\n~b", "~a\n~b");
    same("~a\n~  \n~b", "~a\n~b");
}

#[test]
fn hash2_21_order() {
    differ("Mara waits.\n\nOtto sits.", "Otto sits.\n\nMara waits.");
    differ("MARA\nHi\nthere", "MARA\nthere\nHi");
    differ("MARA\nHi\n\nOTTO\nHey", "OTTO\nHey\n\nMARA\nHi");
    // Two scenes.
    let ab = "INT. A - DAY\n\nx\n\nINT. B - DAY\n\ny\n";
    let ba = "INT. B - DAY\n\ny\n\nINT. A - DAY\n\nx\n";
    assert_ne!(dh(ab), dh(ba));
    let mut s = scene_hashes(ab);
    s.reverse();
    assert_eq!(s, scene_hashes(ba));
}

#[test]
fn hash2_22_adding_and_removing_elements() {
    differ("a", "a\n\nb");
    differ("a\n\nb", "a");
    differ("a", "a\n\n> b");
    differ("a\n\nb", "a\n\n= b\n\nb");
    differ("a", "a\n\n~la");
    // A Synopsis changes the hash, a Section does not.
    differ("a", "a\n\n= beat");
    same("a", "a\n\n# Act");
    // A Synopsis right after the heading is the scene's own.
    assert_ne!(
        sig(&format!("{H}\n\n= beat\n\na\n")).0,
        sig(&format!("{H}\n\na\n")).0
    );
}

#[test]
fn hash2_23_preamble() {
    let story = format!("{H}\n\nx\n");
    let none = story.clone();
    let one = format!("Fade in.\n\n{story}");
    let other = format!("Fade out.\n\n{story}");
    let two = format!("Fade in.\n\n> CUT\n\n{story}");
    let srcs = [&none, &one, &other, &two];
    for (i, a) in srcs.iter().enumerate() {
        for b in &srcs[i + 1..] {
            assert_ne!(ph(a), ph(b));
            assert_ne!(dh(a), dh(b));
            assert_eq!(scene_hashes(a), scene_hashes(b));
        }
    }
    // An empty preamble has one fixed hash.
    assert_eq!(ph(&none), ph(&format!("Title: T\n\n{story}")));
    assert_eq!(ph(&none), ph(""));
    assert_eq!(ph(&none), Preamble::default().hash().to_string());
    assert_eq!(ph(&none), sha("preamble", &u32be(0)));
}

#[test]
fn hash2_24_the_boundary() {
    differ("Mara waits.\nShe sits.", "Mara waits.\n\nShe sits.");
    differ("MARA\nHello\nthere", "MARA\nHello\n\nthere");
    differ("MARA\nHi", "MARA\n\nHi");
    differ("MARA (V.O.)\nHi", "MARA (V.O.)\n\nHi");
}

#[test]
fn hash2_25_scene_membership() {
    // A Synopsis directly before a heading is in the scene that follows.
    let plain_src = "INT. A - DAY\n\nx\n\nINT. B - DAY\n\ny\n";
    let before = "INT. A - DAY\n\nx\n\n= beat\n\nINT. B - DAY\n\ny\n";
    let (p, b) = (scene_hashes(plain_src), scene_hashes(before));
    assert_eq!(p[0], b[0], "not in the scene before");
    assert_ne!(p[1], b[1], "in the scene that follows");
    // A Synopsis directly after a heading is in its own scene.
    let after = "INT. A - DAY\n\n= beat\n\nx\n\nINT. B - DAY\n\ny\n";
    let a = scene_hashes(after);
    assert_ne!(p[0], a[0]);
    assert_eq!(p[1], a[1]);
    // Between two headings, the next one wins.
    let between = "INT. A - DAY\n\n= beat\n\nINT. B - DAY\n\ny\n";
    let bw = scene_hashes(between);
    assert_eq!(
        bw[0],
        scene_hashes("INT. A - DAY\n\nINT. B - DAY\n\ny\n")[0]
    );
    assert_ne!(bw[1], p[1]);
    // Before the first heading: scene 1, not the preamble.
    let first = "INT. A - DAY\n\nx\n";
    for pre in ["= beat\n\n", "# Act\n\n= beat\n\n"] {
        let src = format!("{pre}{first}");
        assert_eq!(ph(&src), ph(first), "{pre:?}");
        let with = &scene_hashes(&src)[0];
        if pre.contains("= beat") {
            assert_ne!(with, &scene_hashes(first)[0], "{pre:?}");
        }
    }
    // A Section alone changes nothing: it is in `Scene::sections` and not hashed.
    assert_eq!(ph("# Act\n\nINT. A - DAY\n\nx\n"), ph(first));
    assert_eq!(
        scene_hashes("# Act\n\nINT. A - DAY\n\nx\n"),
        scene_hashes(first)
    );
}

// ---- injectivity, scenes and documents ----

#[test]
fn hash2_30_scene_and_document_injectivity() {
    // A heading `A` with the Action `B`, against a heading `A B` with no element.
    let (a, ab) = (".A\n\nB\n", ".A B\n");
    assert_ne!(sig(a).0, sig(ab).0);
    assert_ne!(dh(a), dh(ab));
    // Two scenes against one scene with the same elements.
    let two = ".A\n\nx\n\n.B\n\ny\n";
    let one = ".A\n\nx\n\ny\n";
    assert_ne!(dh(two), dh(one));
    // The same scene twice against once.
    assert_ne!(dh(".A\n\nx\n\n.A\n\nx\n"), dh(".A\n\nx\n"));
    // A scene and the preamble that holds its elements.
    assert_ne!(dh("x\n\ny\n"), dh(".A\n\nx\n\ny\n"));
}

// ---- scene, element and document relations ----

#[test]
fn hash2_32_position_and_number_do_not_matter() {
    let k = "INT. K - DAY\n\nMara waits.\n\nMARA\nHi\n";
    let first = scene_hashes(&format!("{k}\nINT. A - DAY\n\nx\n"))[0].clone();
    let second = scene_hashes(&format!("INT. A - DAY\n\nx\n\n{k}"))[1].clone();
    let numbered =
        scene_hashes("INT. A - DAY\n\nx\n\nINT. K - DAY #42#\n\nMara waits.\n\nMARA\nHi\n")[1]
            .clone();
    assert_eq!(first, second);
    assert_eq!(first, numbered);
    let twin = scene_hashes(&format!("{k}\n{k}"));
    assert_eq!(twin.len(), 2);
    assert_eq!(twin[0], twin[1]);
    let sp = parse(&format!("{k}\n{k}")).screenplay();
    assert_eq!(sp.scenes[0].element_hashes(), sp.scenes[1].element_hashes());
    assert_ne!(sp.scenes[0].span, sp.scenes[1].span);
}

fn raw(d: &Digest) -> Vec<u8> {
    d.bytes.to_vec()
}

#[test]
fn hash2_33_element_hashes_make_the_scene_hash() {
    let src = "= opening\n\nINT. K - DAY\n\nMara waits.\n\nMARA (V.O.)\nHello\n(quietly)\nthere\n= beat\n\n~la\n";
    let s = scene(src);
    let mut expected: Vec<Digest> = s
        .synopses
        .iter()
        .flat_map(|x| Element::Synopsis(x.clone()).hashes())
        .collect();
    expected.extend(s.elements.iter().flat_map(Element::hashes));
    assert_eq!(s.synopses.len(), 1);
    assert!(expected.len() >= 8, "{}", expected.len());
    assert_eq!(s.element_hashes(), expected);
    // Recompute the scene hash from the heading and those digests.
    let heading = "INT. K - DAY";
    let pre = [
        u32be(heading.len()),
        heading.as_bytes().to_vec(),
        u32be(expected.len()),
        expected.iter().flat_map(raw).collect(),
    ]
    .concat();
    assert_eq!(sha("scene", &pre), s.hash().to_string());
}

#[test]
fn hash2_34_hierarchy() {
    let build = |p: [&str; 7]| {
        format!(
            "INT. OTHER - DAY\n\nUnrelated.\n\nMARA\nHi\n\n{}",
            parts(H, p)
        )
    };
    let base_src = build(BASE);
    let base = parse(&base_src).screenplay();
    let words = [
        "Mara fills the teapot.",
        "OTTO",
        "loudly",
        "Coffee?",
        "SMASH CUT TO:",
        "Lo lo",
        "Another beat",
    ];
    for (i, word) in words.iter().enumerate() {
        let mut p = BASE;
        p[i] = word;
        let edited = parse(&build(p)).screenplay();
        let (be, ee) = (
            base.scenes[1].element_hashes(),
            edited.scenes[1].element_hashes(),
        );
        let changed: Vec<usize> = (0..7).filter(|&j| be[j] != ee[j]).collect();
        assert_eq!(changed, [i], "exactly its own element hash");
        assert_ne!(base.scenes[1].hash(), edited.scenes[1].hash());
        assert_ne!(base.hash(), edited.hash());
        // The other scene is untouched.
        assert_eq!(base.scenes[0].hash(), edited.scenes[0].hash());
        assert_eq!(
            base.scenes[0].element_hashes(),
            edited.scenes[0].element_hashes()
        );
        assert_eq!(base.preamble.hash(), edited.preamble.hash());
    }
}

#[test]
fn hash2_35_document_hash() {
    let ab = "INT. A - DAY\n\nx\n\nINT. B - DAY\n\ny\n";
    let c = "\n\nINT. C - DAY\n\nz\n";
    let reference = dh(ab);
    assert_ne!(reference, dh(&format!("{ab}{c}")), "a scene added");
    assert_ne!(reference, dh("INT. A - DAY\n\nx\n"), "a scene removed");
    assert_ne!(
        reference,
        dh("INT. B - DAY\n\ny\n\nINT. A - DAY\n\nx\n"),
        "reordered"
    );
    assert_ne!(reference, dh(&format!("Fade in.\n\n{ab}")), "the preamble");
    assert_eq!(
        reference,
        dh(&format!("Title: T\nAuthor: A\n\n{ab}")),
        "the title page"
    );
    // The scene hashes alone do not move when scenes are only added.
    assert_eq!(scene_hashes(ab)[..], scene_hashes(&format!("{ab}{c}"))[..2]);
}

#[test]
fn hash2_36_title_page_hash() {
    let t = |title: &str| th(&format!("{title}\n\nINT. A - DAY\n\nx\n"));
    let base = t("Title: Tea Time\nAuthor: A");
    // Changes.
    for other in [
        "Titel: Tea Time\nAuthor: A",
        "Title: Tea Times\nAuthor: A",
        "Title: Tea\n   Time\nAuthor: A",
        "Title: Tea Time\nAuthor: A\nCredit: c",
        "Author: A\nTitle: Tea Time",
        "Title: Tea Time",
    ] {
        assert_ne!(t(other), base, "{other:?}");
    }
    // Does not change: key case, NFC against NFD, spacing.
    for other in [
        "TITLE: Tea Time\nauthor: A",
        "Title:    Tea \t Time  \nAuthor:A",
        "  Title: Tea Time\nAuthor: A  ",
    ] {
        assert_eq!(t(other), base, "{other:?}");
    }
    assert_eq!(t("Title:\n   a\n   b"), t("Title: a\n   b"));
    assert_ne!(t("Title: a b"), t("Title: a\n   b"));
    // A stray line is not hashed.
    let stray = parse("Title: X\n  stray\n\nINT. A\n").screenplay();
    let plain_page = parse("Title: X\n\nINT. A\n").screenplay();
    assert!(
        stray
            .title_page
            .as_ref()
            .is_some_and(|p| !p.stray_lines.is_empty())
    );
    assert_eq!(stray.title_page_hash(), plain_page.title_page_hash());
    // It does not depend on the story.
    assert_eq!(
        th("Title: T\n\nINT. A - DAY\n\nx\n"),
        th("Title: T\n\nINT. B - DAY\n\ny\n\nz\n")
    );
    // No title page: zero entries.
    let zero = sha("title-page", &u32be(0));
    assert_eq!(th("INT. A\n"), zero);
    assert_eq!(th(""), zero);
    let empty_page = Screenplay {
        title_page: Some(TitlePage {
            entries: Vec::new(),
            stray_lines: Vec::new(),
            span: None,
        }),
        preamble: Preamble::default(),
        scenes: Vec::new(),
    };
    assert_eq!(empty_page.title_page_hash().to_string(), zero);
    // An entry whose key is empty after normalisation is kept.
    let blank_key = Screenplay {
        title_page: Some(TitlePage {
            entries: vec![TitleEntry {
                key: plain("  "),
                values: vec![plain("v"), plain(" "), plain("w")],
                span: None,
            }],
            stray_lines: Vec::new(),
            span: None,
        }),
        ..empty_page.clone()
    };
    let entry = [
        u32be(0),
        u32be(2),
        u32be(1),
        b"v".to_vec(),
        u32be(1),
        b"w".to_vec(),
    ]
    .concat();
    assert_eq!(
        blank_key.title_page_hash().to_string(),
        sha("title-page", &[u32be(1), entry].concat())
    );
}

#[test]
fn hash2_37_source_hashes() {
    let src = "Title: T\n\nINT. A - DAY\n\nMara waits.\n\nShe sits.\n\nINT. B - DAY\n\ny\n";
    let edits = [
        src.replace("Mara waits.", "  Mara waits."),
        src.replace("Mara waits.", "Mara waits. "),
        src.replace('\n', "\r\n"),
        src.replace("Mara waits.\n\n", "Mara waits.\n\n\n"),
    ];
    let doc = parse(src);
    let sp = doc.screenplay();
    for edited in &edits {
        let d = parse(edited);
        let e = d.screenplay();
        assert_ne!(sp.source_hash(&doc), e.source_hash(&d), "{edited:?}");
        assert_ne!(
            sp.scenes[0].source_hash(&doc),
            e.scenes[0].source_hash(&d),
            "{edited:?}"
        );
        assert_ne!(doc.root().source_hash(), d.root().source_hash());
        // The story hashes do not see any of it.
        assert_eq!(sp.hash(), e.hash(), "{edited:?}");
    }
    // The edit outside the scene's span leaves that span's hash alone.
    let other = parse(&src.replace("Title: T", "Title:  T"));
    assert_eq!(
        sp.scenes[1].source_hash(&doc),
        other.screenplay().scenes[1].source_hash(&other)
    );
    // Equal bytes give equal hashes, whichever document they sit in.
    let again = parse(src);
    assert_eq!(doc.root().source_hash(), again.root().source_hash());
    assert_eq!(sp.source_hash(&doc), again.screenplay().source_hash(&again));
    // Any node, and any span.
    let mut visited = 0;
    for child in doc.root().children() {
        if let fountain_syntax::Child::Node(n) = child {
            visited += 1;
            assert_eq!(n.source_hash(), doc.source_hash(n.span()));
            assert_eq!(
                n.source_hash().to_string(),
                sha("source", n.text().as_bytes())
            );
        }
    }
    assert!(visited > 0, "no node was visited");
    assert_eq!(
        doc.source_hash(Span::new(0, 0)).to_string(),
        sha("source", b"")
    );
    // A scene built in memory has no span.
    let mut built = sp.scenes[0].clone();
    built.span = None;
    assert_eq!(built.source_hash(&doc), None);
    assert!(sp.scenes[0].source_hash(&doc).is_some());
}

#[test]
fn hash2_39_methods_give_v2() {
    assert_eq!(HASH_SCHEME_CURRENT, 2);
    assert_eq!(HASH_SCHEME_CURRENT, HashScheme::V2.number());
    let doc = parse(TEA);
    let sp = doc.screenplay();
    let s = &sp.scenes[0];
    let mut all = vec![
        s.hash(),
        sp.hash(),
        sp.title_page_hash(),
        sp.preamble.hash(),
        sp.source_hash(&doc),
        s.source_hash(&doc).unwrap(),
        doc.source_hash(Span::new(0, 3)),
        doc.root().source_hash(),
    ];
    all.extend(s.element_hashes());
    all.extend(s.elements.iter().flat_map(Element::hashes));
    assert!(all.len() > 8);
    for d in all {
        assert_eq!(d.scheme, HashScheme::V2);
    }
}

// ---- the review focus items ----

#[test]
fn hash2_28_interjections_without_lines() {
    let lyrics = eh("~la");
    let synopsis = eh("Walk.\n\n= beat")[1..].to_vec();
    let cue = eh("MARA\nHello")[..1].to_vec();
    assert_eq!((lyrics.len(), synopsis.len(), cue.len()), (1, 1, 1));

    // A cue followed only by interjections: the records keep source order.
    let ls = eh("MARA\n~la\n= beat");
    assert_eq!(ls, [cue.clone(), lyrics.clone(), synopsis.clone()].concat());
    let sl = eh("MARA\n= beat\n~la");
    assert_eq!(sl, [cue.clone(), synopsis.clone(), lyrics.clone()].concat());
    assert_ne!(ls, sl);
    assert_eq!(eh("MARA\n~la"), [cue.clone(), lyrics.clone()].concat());

    // A cue followed by a lone Synopsis line is an Action and a Synopsis in the view (OFF-21),
    // so a block with only a Synopsis interjection is built in memory.
    let block = |items: Vec<DialogueItem>| {
        Element::Dialogue(DialogueBlock {
            cue: Cue {
                name: plain("MARA"),
                extensions: Vec::new(),
                forced: false,
                span: None,
            },
            items,
            unpaired_caret: false,
        })
    };
    let lyr = || {
        DialogueItem::Interjection(Box::new(Element::Lyrics(Lyrics {
            lines: vec![plain("la")],
        })))
    };
    let syn = || {
        DialogueItem::Interjection(Box::new(Element::Synopsis(Synopsis {
            text: plain("beat"),
        })))
    };
    let got = |els: Vec<DialogueItem>| texts(&block(els).hashes());
    assert_eq!(got(vec![syn()]), [cue.clone(), synopsis.clone()].concat());
    assert_eq!(got(vec![syn(), lyr()]), sl);
    assert_eq!(got(vec![lyr(), syn()]), ls);
}

#[test]
fn degenerate_input_hashes() {
    for src in ["", "\n", "Title: T\n", "x\n"] {
        let doc = parse(src);
        let sp = doc.screenplay();
        assert!(sp.scenes.is_empty());
        let _ = (sp.hash(), sp.title_page_hash(), sp.source_hash(&doc));
        assert_eq!(
            sp.source_hash(&doc),
            doc.source_hash(Span::new(0, src.len()))
        );
    }
    let doc = parse("");
    let sp = doc.screenplay();
    assert_eq!(sp.source_hash(&doc).to_string(), sha("source", b""));
    assert_eq!(
        sp.title_page_hash().to_string(),
        sha("title-page", &u32be(0))
    );
    let mut document = raw(&sp.preamble.hash());
    document.extend(u32be(0));
    assert_eq!(sp.hash().to_string(), sha("document", &document));

    // A scene built in memory, with every span `None`, hashes like the parsed scene.
    let parsed = scene("INT. KITCHEN - DAY\n\nMara fills the kettle.\n");
    let mut built = Scene::new(Heading {
        text: plain("INT. KITCHEN - DAY"),
        forced: false,
        span: None,
    });
    built.elements = vec![Element::Action(Action {
        text: plain("Mara fills the kettle."),
        centered: false,
        forced: false,
        interjections: Vec::new(),
    })];
    assert_eq!(built.hash(), parsed.hash());
    assert_eq!(built.element_hashes(), parsed.element_hashes());
    assert_eq!(built.source_hash(&parse("")), None);
    // A scene whose synopsis is built in memory too.
    let mut with_synopsis = built.clone();
    with_synopsis.synopses.push(Synopsis {
        text: plain("beat"),
    });
    assert_ne!(with_synopsis.hash(), built.hash());

    // A Document with a span past nothing: an empty span hashes the empty source.
    let d: Document = parse("abc");
    assert_eq!(
        d.source_hash(Span::new(1, 1)).to_string(),
        sha("source", b"")
    );
}

/// HASH2-31
#[test]
fn hash2_31_domain_separation() {
    let names = [
        "element",
        "scene",
        "preamble",
        "document",
        "title-page",
        "source",
    ];
    let digests: Vec<String> = names.iter().map(|n| sha(n, b"payload")).collect();
    for (i, a) in digests.iter().enumerate() {
        for b in &digests[i + 1..] {
            assert_ne!(a, b);
        }
    }
    // A scene with no elements and a preamble with no elements differ.
    let empty_scene = scene("INT. A - DAY\n");
    assert!(empty_scene.elements.is_empty());
    assert_ne!(empty_scene.hash(), Preamble::default().hash());
}
