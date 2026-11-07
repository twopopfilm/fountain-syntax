//! No Note or Boneyard of the tree goes missing from the typed view, or shows up twice (SPEC 9).

use fountain_syntax::{
    Action, Boneyard, Cue, DialogueBlock, DialogueItem, Document, Element, Node, NodeKind, Note,
    NoteConvention, PageBreak, Run, Screenplay, Span, Text, parse,
};

/// `(is_note, span)` of every Note and Boneyard node of the tree, sorted.
fn tree_atoms(node: Node<'_>, out: &mut Vec<(bool, Span)>) {
    match node.kind() {
        NodeKind::Note => out.push((true, node.span())),
        NodeKind::Boneyard => out.push((false, node.span())),
        _ => {}
    }
    for child in node.children() {
        if let fountain_syntax::Child::Node(n) = child {
            tree_atoms(n, out);
        }
    }
}

type Atoms = Vec<(bool, Option<Span>)>;

fn runs(runs: &[Run], out: &mut Atoms) {
    for run in runs {
        match run {
            Run::Note(n) => out.push((true, n.span)),
            Run::Boneyard(b) => out.push((false, b.span)),
            Run::Emphasis { runs: inner, .. } => self::runs(inner, out),
            Run::Plain(_) | Run::Escaped(_) => {}
        }
    }
}

fn text(t: &Text, out: &mut Atoms) {
    runs(&t.runs, out);
}

fn action(a: &Action, out: &mut Atoms) {
    text(&a.text, out);
    a.interjections.iter().for_each(|e| element(e, out));
}

fn cue(c: &Cue, out: &mut Atoms) {
    text(&c.name, out);
    c.extensions.iter().for_each(|e| text(&e.text, out));
}

fn block(b: &DialogueBlock, out: &mut Atoms) {
    cue(&b.cue, out);
    for item in &b.items {
        match item {
            DialogueItem::Line(t) | DialogueItem::Parenthetical(t) => text(t, out),
            DialogueItem::Interjection(e) => element(e, out),
        }
    }
}

fn element(e: &Element, out: &mut Atoms) {
    match e {
        Element::Action(a) => action(a, out),
        Element::Dialogue(b) => block(b, out),
        Element::DualDialogue(d) => {
            block(&d.left, out);
            block(&d.right, out);
        }
        Element::Transition(t) => text(&t.text, out),
        Element::Lyrics(l) => l.lines.iter().for_each(|t| text(t, out)),
        Element::Section(s) => text(&s.text, out),
        Element::Synopsis(s) => text(&s.text, out),
        Element::PageBreak(_) => {}
        Element::Note(n) => out.push((true, n.span)),
        Element::Boneyard(b) => out.push((false, b.span)),
        #[cfg(feature = "shot")]
        Element::Shot(s) => text(&s.text, out),
        _ => {}
    }
}

fn view_atoms(v: &Screenplay) -> Atoms {
    let mut out = Vec::new();
    if let Some(tp) = &v.title_page {
        for e in &tp.entries {
            text(&e.key, &mut out);
            e.values.iter().for_each(|t| text(t, &mut out));
        }
        tp.stray_lines.iter().for_each(|t| text(t, &mut out));
    }
    v.preamble
        .elements
        .iter()
        .for_each(|e| element(e, &mut out));
    for s in &v.scenes {
        text(&s.heading.text, &mut out);
        s.sections.iter().for_each(|x| text(&x.text, &mut out));
        s.synopses.iter().for_each(|x| text(&x.text, &mut out));
        s.elements.iter().for_each(|e| element(e, &mut out));
    }
    out
}

fn view(src: &str) -> Screenplay {
    parse(src).screenplay()
}

fn pre(src: &str) -> Vec<Element> {
    view(src).preamble.elements
}

fn note(text: &str, start: usize, end: usize) -> Element {
    Element::Note(Note {
        text: text.to_owned(),
        span: Some(Span::new(start, end)),
    })
}

fn boneyard(text: &str, start: usize, end: usize) -> Element {
    Element::Boneyard(Boneyard {
        text: text.to_owned(),
        span: Some(Span::new(start, end)),
    })
}

fn page_break(start: usize, end: usize) -> Element {
    Element::PageBreak(PageBreak {
        span: Some(Span::new(start, end)),
    })
}

fn interjections(items: &[DialogueItem]) -> Vec<Element> {
    items
        .iter()
        .filter_map(|i| match i {
            DialogueItem::Interjection(e) => Some((**e).clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn off_19_note_on_a_page_break_line_is_kept() {
    // The Note is a sibling right after the Page Break (R15), with its own span.
    assert_eq!(
        pre("=== [[note]]\n"),
        [page_break(0, 12), note("note", 4, 12)]
    );
}

#[test]
fn off_16_boneyard_before_page_break_is_kept() {
    assert_eq!(
        pre("/* x */===\n"),
        [page_break(0, 10), boneyard(" x ", 0, 7)]
    );
    // Both atoms of one line, in source order.
    assert_eq!(
        pre("/* x */=== [[n]]\n"),
        [page_break(0, 16), boneyard(" x ", 0, 7), note("n", 11, 16)]
    );
}

#[test]
fn off_19_atoms_on_a_page_break_inside_an_action_are_interjections() {
    let els = pre("a\n=== [[n]]\nb");
    let Element::Action(a) = &els[0] else {
        panic!("not Action: {els:?}")
    };
    assert_eq!(a.text.plain(), "a\nb");
    assert_eq!(a.interjections, [page_break(2, 11), note("n", 6, 11)]);
    let els = pre("a\n/* x */===\nb");
    let Element::Action(a) = &els[0] else {
        panic!("not Action: {els:?}")
    };
    assert_eq!(a.interjections, [page_break(2, 12), boneyard(" x ", 2, 9)]);
}

#[test]
fn off_19_atoms_on_a_page_break_inside_a_dialogue_block_are_interjections() {
    let els = pre("MARA\nHello\n=== [[n]]\nAgain\n");
    let Element::Dialogue(b) = &els[0] else {
        panic!("not Dialogue: {els:?}")
    };
    assert_eq!(
        interjections(&b.items),
        [page_break(11, 20), note("n", 15, 20)]
    );
}

#[test]
fn off_19_atoms_on_a_page_break_between_dual_halves_end_the_left_items() {
    // A Note after a Page Break is transparent for pairing, like any whole-line Note.
    let els = pre("MARA\nHi\n\n=== [[n]]\n\nTOM ^\nYo\n");
    assert_eq!(els.len(), 1, "{els:?}");
    let Element::DualDialogue(d) = &els[0] else {
        panic!("not DualDialogue: {els:?}")
    };
    assert_eq!(
        interjections(&d.left.items),
        [page_break(9, 18), note("n", 13, 18)]
    );
}

#[test]
fn off_19_page_break_atoms_keep_the_scene_span_and_the_outline_run() {
    // The extra Note lies inside the Page Break's span, so the scene still ends with the break.
    let v = view("= syn\n\nINT. X\n\n=== [[n]]\n");
    assert_eq!(v.scenes[0].synopses.len(), 1);
    assert_eq!(
        v.scenes[0].elements,
        [page_break(15, 24), note("n", 19, 24)]
    );
    assert_eq!(v.scenes[0].span, Some(Span::new(0, 24)));
    // `/* x */===` puts the Boneyard first in the break, yet the scene ends at the break's end.
    let v = view("INT. X\n\n/* x */===\n");
    assert_eq!(v.scenes[0].span, Some(Span::new(0, 18)));
    // A Page Break line breaks the run before a heading, as a whole-line Note does.
    let v = view("= syn\n=== [[n]]\n\nINT. X\n");
    assert!(v.scenes[0].synopses.is_empty());
    assert_eq!(v.preamble.elements.len(), 3);
}

#[test]
fn off_8_note_inside_scene_number_is_kept() {
    let v = view("INT. A #1[[n]]#\n");
    let s = &v.scenes[0];
    assert_eq!(s.number.as_ref().map(|n| n.text.as_str()), Some("1"));
    assert_eq!(
        s.heading.text.runs,
        [
            Run::Plain("INT. A ".to_owned()),
            Run::Note(Note {
                text: "n".to_owned(),
                span: Some(Span::new(9, 14))
            })
        ]
    );
    let v = view("INT. A #1/* x */#\n");
    let s = &v.scenes[0];
    assert_eq!(s.number.as_ref().map(|n| n.text.as_str()), Some("1"));
    assert!(
        matches!(s.heading.text.runs.last(), Some(Run::Boneyard(b)) if b.span == Some(Span::new(9, 16)))
    );
}

#[test]
fn text_span_of_a_title_value_can_reach_beyond_its_entry() {
    let tp = view("Title: a\n  [[n]]\n")
        .title_page
        .expect("a title page");
    let e = &tp.entries[0];
    assert_eq!(e.values.len(), 1);
    assert_eq!(e.values[0].plain(), "a");
    assert_eq!(e.values[0].span, Some(Span::new(7, 16)));
    assert_eq!(e.span, Some(Span::new(0, 8)));
}

#[test]
fn ext_notes_view_3_documented_surprising_results() {
    let conv = |s: &str| {
        Note {
            text: s.to_owned(),
            span: None,
        }
        .conventions()
    };
    assert_eq!(
        conv("color teal:"),
        Some(NoteConvention::Colour("teal:".to_owned()))
    );
    assert_eq!(
        conv("marker 12:30"),
        Some(NoteConvention::Marker {
            colour: Some("12".to_owned()),
            text: "30".to_owned()
        })
    );
    assert_eq!(
        conv("http://x"),
        Some(NoteConvention::Tag {
            tag: "http".to_owned(),
            text: "//x".to_owned()
        })
    );
}

fn check(src: &str) -> Result<(), String> {
    let doc: Document = parse(src);
    let mut want = Vec::new();
    tree_atoms(doc.root(), &mut want);
    let mut want: Atoms = want.into_iter().map(|(n, s)| (n, Some(s))).collect();
    let mut got = view_atoms(&doc.screenplay());
    let key = |a: &(bool, Option<Span>)| (a.0, a.1.map(|s| (s.start, s.end)));
    want.sort_by_key(key);
    got.sort_by_key(key);
    if want == got {
        Ok(())
    } else {
        Err(format!("{src:?}: tree {want:?}, view {got:?}"))
    }
}

fn check_all(inputs: impl Iterator<Item = String>) {
    let mut bad = Vec::new();
    for src in inputs {
        if let Err(e) = check(&src) {
            bad.push(e);
        }
    }
    assert!(
        bad.is_empty(),
        "{} failures, first: {:?}",
        bad.len(),
        &bad[..bad.len().min(8)]
    );
}

#[test]
fn off_19_every_tree_atom_appears_once_in_the_view_for_short_inputs() {
    const A: [char; 19] = [
        '=', '#', '[', ']', '/', '*', '\n', ' ', 'A', '1', '.', '!', '@', '^', '(', ')', '~', '>',
        '<',
    ];
    let mut inputs = vec![String::new()];
    let mut layer = vec![String::new()];
    for _ in 0..4 {
        layer = layer
            .iter()
            .flat_map(|s| A.iter().map(move |c| format!("{s}{c}")))
            .collect();
        inputs.extend(layer.iter().cloned());
    }
    check_all(inputs.into_iter());
}

#[test]
fn off_19_every_tree_atom_appears_once_in_the_view_for_hand_picked_inputs() {
    let picked = [
        "=== [[note]]\n",
        "/* x */===\n",
        "/* x */=== [[n]]\n",
        "a\n=== [[n]]\nb",
        "a\n/* x */===\nb",
        "INT. A #1[[n]]#\n",
        "INT. A #1/* x */#\n",
        "INT. A [[a]] #1[[n]]#\n",
        "MARA\n=== [[n]]\nHello\n",
        "MARA\nHello\n=== [[n]]\n\nTOM ^\nHi\n",
        "MARA\nHello\n\n=== [[n]]\n\nTOM ^\nHi\n",
        "MARA\nHello\n\n=== [[n]] /* b */\n\nTOM ^\nHi\n",
        "# A\n=== [[n]]\nINT. X\n",
        "= syn\n=== [[n]]\nINT. X\n",
        "INT. X\n=== [[n]]\n= syn\n",
        "Title: a\n  [[n]]\n",
        "Title: a\n=== [[n]]\n",
        "[[n]]Title: a\n",
        "Title: [[n]]a\nAuthor: /* b */\n",
        "~ la [[n]]\n=== [[n]]\n~ la\n",
        "> centered [[n]] <\n",
        "MARA (V.O. [[n]])\nHi\n",
        "@MARA [[n]]\n(paren [[n]])\nHi *a [[n]] b*\n",
        "TOM ^\n=== [[n]]\n",
        "A\n\n=== [[n]]\n\n=== /* b */ [[c]]\n",
    ];
    check_all(picked.iter().map(|s| (*s).to_owned()));
}

/// Deterministic pseudo-random strings built from the fragments that make hidden spans and
/// structure, to reach inputs the short enumeration cannot.
#[test]
fn off_19_every_tree_atom_appears_once_in_the_view_for_generated_inputs() {
    const F: [&str; 22] = [
        "[[n]]", "[[", "]]", "/* b */", "/*", "*/", "===", "=", "#", "##", "\n", "\n\n", " ", "A",
        "1", "INT. ", "MARA", "^", "~", "(", ")", "*",
    ];
    let mut state: u64 = 0x2545_f491_4f6c_dd1d;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        usize::try_from(state % 1_000_003).unwrap_or(0)
    };
    let inputs = (0..30_000).map(|_| {
        let len = 1 + next() % 12;
        (0..len).map(|_| F[next() % F.len()]).collect::<String>()
    });
    check_all(inputs);
}
