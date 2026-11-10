#![cfg(feature = "hash")]
//! Scheme 2 element hashes (`SPEC.md` 11.3 to 11.5), read through `parse(..).screenplay()`.

use fountain_syntax::{
    Action, Boneyard, Cue, DialogueBlock, DialogueItem, Element, Extension, Lyrics, Note, Run,
    Synopsis, Text, parse,
};

/// The text forms of `Element::hashes` over the preamble and every scene of `src`, in order.
fn hashes(src: &str) -> Vec<String> {
    let sp = parse(src).screenplay();
    let scenes = sp.scenes.iter().flat_map(|s| s.elements.iter());
    sp.preamble
        .elements
        .iter()
        .chain(scenes)
        .flat_map(Element::hashes)
        .map(|d| d.to_string())
        .collect()
}

/// The hashes of the elements of `body`, written as the content of one scene.
fn h(body: &str) -> Vec<String> {
    hashes(&format!("INT. KITCHEN - DAY\n\n{body}\n"))
}

fn plain(s: &str) -> Text {
    Text {
        span: None,
        runs: vec![Run::Plain(s.to_string())],
    }
}

fn hidden(run: Run) -> Text {
    Text {
        span: None,
        runs: vec![run],
    }
}

fn cue(name: &str, extensions: &[&str]) -> Cue {
    Cue {
        name: plain(name),
        extensions: extensions
            .iter()
            .map(|e| Extension { text: plain(e) })
            .collect(),
        forced: false,
        span: None,
    }
}

fn block(cue: Cue, items: Vec<DialogueItem>) -> Element {
    Element::Dialogue(DialogueBlock {
        cue,
        items,
        unpaired_caret: false,
    })
}

#[test]
fn hash2_41_action() {
    assert_eq!(
        h("Mara fills the kettle."),
        ["fs2:1cd74cd5159cd81b65c689eff8d55d8dc7bfa6de082dbb341873240f6595850b"]
    );
    // Forced or not, wrapped or not, makes no difference to the record.
    assert_eq!(h("!Mara fills\nthe kettle."), h("Mara fills the kettle."));
}

#[test]
fn hash2_42_cue_parenthetical_dialogue() {
    assert_eq!(
        h("MARA (V.O.) (CONT'D)\n(quietly)\nTea?"),
        [
            "fs2:1a556f7deab4f5fa5628f2e5d969f50285b3c00f75fe18d6a8dd00f26646af95",
            "fs2:07e731a7acbe2ca8fa63f9a0a9e38dfb1abb4af25c2287e3e715e0bfe8ab45a3",
            "fs2:b37c03f30bd33f9849e35026d903755b6bd92eacdd9bc69a90b303295bc651c1",
        ]
    );
}

#[test]
fn hash2_43_nfc() {
    let expected = ["fs2:dd8fe5281b946df777823637c8a3b957734f80784d8574d0c49b9f0fa8e7f050"];
    assert_eq!(h("Caf\u{e9}"), expected);
    assert_eq!(h("Cafe\u{301}"), expected);
}

#[test]
fn hash2_44_centered_dual_lyrics() {
    assert_eq!(
        h("> The end <"),
        ["fs2:d110baae28cb16a64562de38a45bed9c1ec7afda132311c1b01a9793c98cd192"]
    );
    // MARA, Hi, the right-hand cue, Hey.
    let dual = h("MARA\nHi\n\nOTTO (v.o.) (cont'd) (filtered) ^\nHey");
    assert_eq!(dual.len(), 4);
    assert_eq!(
        dual[2],
        "fs2:e9959f1d4a68e8749ae4c32bdb89ab58869100e8fcc05016cdc7b26070247dea"
    );
    assert_eq!(
        h("~La la\n~Lo lo"),
        ["fs2:29e754ee12c86a5ed2ccefa29c894e9a2336092760afb4abf2b78e9dc3b8be44"]
    );
}

#[test]
fn hash2_26_dropped_records() {
    let note = Run::Note(Note {
        text: "aside".to_string(),
        span: None,
    });
    let boneyard = Run::Boneyard(Boneyard {
        text: "cut".to_string(),
        span: None,
    });
    let action = |text: Text| {
        Element::Action(Action {
            text,
            centered: false,
            forced: true,
            interjections: Vec::new(),
        })
    };
    let dropped = [
        action(hidden(note)),
        action(hidden(boneyard)),
        action(plain("  ")),
        Element::Synopsis(Synopsis { text: plain("") }),
        Element::Lyrics(Lyrics {
            lines: vec![plain(""), plain("  ")],
        }),
        Element::Lyrics(Lyrics { lines: Vec::new() }),
        // An empty name drops the cue; the empty Parenthetical `()` and the blank Line are dropped too.
        block(
            cue("", &[]),
            vec![
                DialogueItem::Parenthetical(plain("()")),
                DialogueItem::Parenthetical(plain("( )")),
                DialogueItem::Line(plain(" ")),
            ],
        ),
    ];
    for el in &dropped {
        assert!(el.hashes().is_empty(), "{el:?}");
    }
    // Inside a block the dropped pieces vanish one by one while the others stay.
    let kept = block(
        cue("MARA", &[]),
        vec![
            DialogueItem::Parenthetical(plain("()")),
            DialogueItem::Line(plain("Tea?")),
        ],
    );
    let kept: Vec<String> = kept.hashes().iter().map(|d| d.to_string()).collect();
    assert_eq!(kept, h("MARA\nTea?"));
    // An empty extension group is kept, so the cue differs from the bare cue.
    let empty_ext = block(cue("MARA", &[""]), Vec::new()).hashes();
    let bare = block(cue("MARA", &[]), Vec::new()).hashes();
    assert_eq!((empty_ext.len(), bare.len()), (1, 1));
    assert_ne!(empty_ext, bare);
}

#[test]
fn hash2_27_parenthetical_text() {
    let quiet = h("MARA\n(quietly)\nTea?");
    assert_eq!(h("MARA\n( quietly )\nTea?"), quiet);
    assert_ne!(h("MARA\n(Quietly)\nTea?"), quiet);
}

#[test]
fn hash2_28_dialogue_record_order() {
    let cue_and_run = h("MARA\nHello there");
    let lyrics = h("~la");
    // A Synopsis right after the heading is the scene's own, not an element, so lead with an Action.
    let synopsis = h("Walk.\n\n= beat")[1..].to_vec();
    assert_eq!((cue_and_run.len(), lyrics.len(), synopsis.len()), (2, 1, 1));

    // Inside a run: the Lyrics record comes after the run's Dialogue record.
    assert_eq!(
        h("MARA\nHello\n~la\nthere"),
        [cue_and_run.clone(), lyrics].concat()
    );

    // Between the cue and the first line: the record comes before the run's.
    let before = [
        cue_and_run[..1].to_vec(),
        synopsis.clone(),
        cue_and_run[1..].to_vec(),
    ]
    .concat();
    assert_eq!(h("MARA\n= beat\nHello there"), before);

    // After the last line: it keeps its place, after the run's record.
    assert_eq!(
        h("MARA\nHello there\n= beat"),
        [cue_and_run, synopsis.clone()].concat()
    );

    // Before a Parenthetical: after the run's record, before the Parenthetical.
    let paren = h("MARA\n(quietly)\nx")[1].clone();
    let run = h("MARA\nHello")[1].clone();
    let tail = h("MARA\nx")[1].clone();
    let got = h("MARA\nHello\n= beat\n(quietly)\nx");
    assert_eq!(got[1..], [run, synopsis[0].clone(), paren, tail]);

    // An Action's interjection comes after the Action's record.
    assert_eq!(h("Walk.\n= beat"), [h("Walk."), synopsis].concat());

    // A dual pair gives the left block's records, then the right's.
    let left = h("MARA\nHi");
    let dual = h("MARA\nHi\n\nOTTO ^\nHey");
    assert_eq!(dual.len(), 4);
    assert_ne!(dual[0], left[0], "the dual flag is set on the left cue");
    assert_eq!(dual[1], left[1]);
    assert_eq!(dual[3], h("OTTO\nHey")[1]);
}

#[cfg(feature = "shot")]
#[test]
fn hash2_29_shot() {
    let shot = h("!!CLOSE ON THE KETTLE");
    assert_eq!(shot.len(), 1);
    assert_eq!(h("!!  CLOSE ON   THE KETTLE"), shot);
    // The marks are not part of the text, and a Shot is not an Action.
    assert_ne!(shot, h("CLOSE ON THE KETTLE"));
}

#[test]
fn hash2_30_injectivity() {
    // Each pair must differ; only the bytes of the records can tell them apart.
    let pairs = [
        ("ab\n\nc", "a\n\nbc"),
        ("MARA\nab\n(p)\nc", "MARA\na\n(p)\nbc"),
        ("~a b", "~a\n~b"),
        ("X (A) (B)\nHi", "X (A B)\nHi"),
        ("X (Y)\nHi", "X Y\nHi"),
    ];
    for (a, b) in pairs {
        let (ha, hb) = (h(a), h(b));
        assert!(!ha.is_empty() && !hb.is_empty(), "{a:?} against {b:?}");
        assert_ne!(ha, hb, "{a:?} against {b:?}");
    }
    // The Action, the Transition and the Synopsis of the same word are three different records.
    let (action, transition, synopsis) = (h("x"), h("> x"), h("y\n\n= x")[1..].to_vec());
    assert_eq!((action.len(), transition.len(), synopsis.len()), (1, 1, 1));
    assert_ne!(action, transition);
    assert_ne!(action, synopsis);
    assert_ne!(transition, synopsis);
}
