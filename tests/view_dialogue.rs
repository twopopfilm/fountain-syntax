//! The typed view's Dialogue blocks and dual dialogue (SPEC 4.4-4.8, 9), read through `parse`.

use fountain_syntax::{
    DialogueBlock, DialogueItem, DualDialogue, Element, Run, Screenplay, Span, Text, parse,
};

fn view(src: &str) -> Screenplay {
    parse(src).screenplay()
}

/// The elements of the preamble.
fn pre(src: &str) -> Vec<Element> {
    view(src).preamble.elements
}

fn block(e: &Element) -> &DialogueBlock {
    match e {
        Element::Dialogue(b) => b,
        other => panic!("not a Dialogue block: {other:?}"),
    }
}

fn dual(e: &Element) -> &DualDialogue {
    match e {
        Element::DualDialogue(d) => d,
        other => panic!("not a DualDialogue: {other:?}"),
    }
}

/// The only element of the preamble, as a Dialogue block.
fn only_block(src: &str) -> DialogueBlock {
    let els = pre(src);
    assert_eq!(els.len(), 1, "{src:?}");
    block(&els[0]).clone()
}

/// The kind and plain text of each item: `L` Line, `P` Parenthetical, `I` Interjection.
fn items(b: &DialogueBlock) -> Vec<(char, String)> {
    b.items
        .iter()
        .map(|i| match i {
            DialogueItem::Line(t) => ('L', t.plain()),
            DialogueItem::Parenthetical(t) => ('P', t.plain()),
            DialogueItem::Interjection(_) => ('I', String::new()),
        })
        .collect()
}

fn line(kind: char, text: &str) -> (char, String) {
    (kind, text.to_owned())
}

fn interjection(i: &DialogueItem) -> &Element {
    match i {
        DialogueItem::Interjection(e) => e,
        other => panic!("not an Interjection: {other:?}"),
    }
}

/// Plain text of each preamble element of a script made of single-line blocks, for sequences.
fn shape(els: &[Element]) -> Vec<&'static str> {
    els.iter()
        .map(|e| match e {
            Element::Dialogue(b) if b.unpaired_caret => "unpaired",
            Element::Dialogue(_) => "dialogue",
            Element::DualDialogue(_) => "dual",
            Element::Action(_) => "action",
            _ => "other",
        })
        .collect()
}

#[test]
fn el_character_1_view() {
    let b = only_block("NELL (V.O.)\nHello.");
    assert_eq!(b.cue.name.plain(), "NELL");
    assert_eq!(b.cue.name.span, Some(Span::new(0, 4)));
    assert_eq!(b.cue.extensions.len(), 1);
    assert_eq!(b.cue.extensions[0].text.plain(), "V.O.");
    assert!(!b.cue.forced);
    assert_eq!(b.cue.span, Some(Span::new(0, 11)));
    assert!(!b.unpaired_caret);
    assert_eq!(items(&b), [line('L', "Hello.")]);
}

#[test]
fn el_character_2_view_forced_keeps_case() {
    let b = only_block("@McCLANE\nYippee.");
    assert_eq!(b.cue.name.plain(), "McCLANE");
    assert_eq!(b.cue.name.span, Some(Span::new(1, 8)));
    assert!(b.cue.forced);
    // A forced cue stands with nothing after it.
    let b = only_block("@MARA\n");
    assert!(b.cue.forced);
    assert_eq!(b.cue.name.plain(), "MARA");
    assert!(b.items.is_empty());
}

#[test]
fn el_character_3_view_last_line_is_action() {
    assert_eq!(shape(&pre("a\n\nMARA")), ["action", "action"]);
}

#[test]
fn el_dialogue_1_view_lines_kept() {
    let b = only_block("MARA\nline one\n  line two ");
    assert_eq!(items(&b), [line('L', "line one"), line('L', "line two")]);
    let DialogueItem::Line(second) = &b.items[1] else {
        panic!("a Line")
    };
    assert_eq!(second.span, Some(Span::new(16, 24)));
}

#[test]
fn el_dialogue_2_view_two_space_line() {
    let b = only_block("MARA\none\n  \ntwo");
    assert_eq!(
        items(&b),
        [line('L', "one"), line('L', ""), line('L', "two")]
    );
}

#[test]
fn el_dialogue_3_view_blank_ends_block() {
    let els = pre("MARA\nhi\n\nAfter.");
    assert_eq!(shape(&els), ["dialogue", "action"]);
    assert_eq!(items(block(&els[0])), [line('L', "hi")]);
}

#[test]
fn el_parenthetical_1_view_keeps_parentheses() {
    let b = only_block("MARA\n(quietly)\nhi\n(beat)\nbye");
    assert_eq!(
        items(&b),
        [
            line('P', "(quietly)"),
            line('L', "hi"),
            line('P', "(beat)"),
            line('L', "bye")
        ]
    );
}

#[test]
fn el_lyrics_2_view_lyric_among_dialogue() {
    let b = only_block("MARA\nhi\n~la la\n~la\nbye");
    assert_eq!(
        items(&b),
        [line('L', "hi"), line('I', ""), line('L', "bye")]
    );
    let Element::Lyrics(l) = interjection(&b.items[1]) else {
        panic!("Lyrics")
    };
    let lines: Vec<String> = l.lines.iter().map(Text::plain).collect();
    assert_eq!(lines, ["la la", "la"]);
}

#[test]
fn el_indent_1_view() {
    for ind in ["\t", "            ", "        ", "    ", ""] {
        let b = only_block(&format!("{ind}MARA\n{ind}(beat)\n{ind}Hello."));
        assert_eq!(b.cue.name.plain(), "MARA", "{ind:?}");
        assert_eq!(
            items(&b),
            [line('P', "(beat)"), line('L', "Hello.")],
            "{ind:?}"
        );
    }
}

#[test]
fn el_dual_1_pair() {
    let els = pre("MARA\nhi\n\nOTTO ^\nyo");
    assert_eq!(els.len(), 1);
    let d = dual(&els[0]);
    assert_eq!(d.left.cue.name.plain(), "MARA");
    assert_eq!(d.right.cue.name.plain(), "OTTO");
    assert_eq!(items(&d.left), [line('L', "hi")]);
    assert_eq!(items(&d.right), [line('L', "yo")]);
    assert!(!d.left.unpaired_caret && !d.right.unpaired_caret);
    // Directly after a block, with no blank line, the caret cue is Dialogue of that block.
    assert_eq!(shape(&pre("MARA\nhi\nOTTO ^\nyo")), ["dialogue"]);
}

#[test]
fn el_dual_2_caret_left_out_of_name_view() {
    for (src, name) in [
        ("OTTO ^\nyo", "OTTO"),
        ("OTTO^\nyo", "OTTO"),
        ("OTTO ^ \nyo", "OTTO"),
        ("@Otto ^\nyo", "Otto"),
    ] {
        let b = only_block(src);
        assert_eq!(b.cue.name.plain(), name, "{src:?}");
        assert!(b.unpaired_caret, "{src:?}");
    }
    let b = only_block("MARA (V.O.) ^\nyo");
    assert_eq!(b.cue.name.plain(), "MARA");
    assert_eq!(b.cue.extensions[0].text.plain(), "V.O.");
    assert!(b.unpaired_caret);
    // OFF-11c: a caret that is not last is part of the name and does not pair.
    let b = only_block("NAME ^ (V.O.)\nyo");
    assert_eq!(b.cue.name.plain(), "NAME ^");
    assert!(!b.unpaired_caret);
    let els = pre("MARA\nhi\n\nNAME ^ (V.O.)\nyo");
    assert_eq!(shape(&els), ["dialogue", "dialogue"]);
}

#[test]
fn off_11a_lone_caret_unpaired() {
    let els = pre("OTTO ^\nyo");
    let b = block(&els[0]);
    assert!(b.unpaired_caret);
    assert_eq!(b.cue.name.plain(), "OTTO");
    assert_eq!(items(b), [line('L', "yo")]);
    // A heading or an Action between the two blocks breaks the pairing.
    let v = view("MARA\nhi\n\nINT. X\n\nOTTO ^\nyo");
    assert_eq!(shape(&v.preamble.elements), ["dialogue"]);
    assert_eq!(shape(&v.scenes[0].elements), ["unpaired"]);
    let els = pre("MARA\nhi\n\nThe door opens.\n\nOTTO ^\nyo");
    assert_eq!(shape(&els), ["dialogue", "action", "unpaired"]);
    // So do a Lyrics run, a Transition and centered text.
    for between in ["~la", "CUT TO:", "> end <"] {
        let els = pre(&format!("MARA\nhi\n\n{between}\n\nOTTO ^\nyo"));
        assert_eq!(els.len(), 3, "{between:?}");
        assert!(block(&els[2]).unpaired_caret, "{between:?}");
    }
}

#[test]
fn off_11d_third_caret_unpaired() {
    let els = pre("MARA\nhi\n\nOTTO ^\nyo\n\nNELL ^\nno");
    assert_eq!(shape(&els), ["dual", "unpaired"]);
    assert_eq!(dual(&els[0]).right.cue.name.plain(), "OTTO");
    assert_eq!(block(&els[1]).cue.name.plain(), "NELL");
    // The caret block after a pair is unpaired even when only transparent lines come between.
    let els = pre("MARA\nhi\n\nOTTO ^\nyo\n\n# s\n\nNELL ^\nno");
    assert_eq!(shape(&els), ["dual", "other", "unpaired"]);
    // A caret block that is not a right half can still take the next caret block as its partner.
    let els = pre("MARA\nhi\n\nOTTO ^\nyo\n\nNELL ^\nno\n\nIVAN ^\nup");
    assert_eq!(shape(&els), ["dual", "dual"]);
    assert_eq!(dual(&els[1]).left.cue.name.plain(), "NELL");
    assert!(!dual(&els[1]).left.unpaired_caret);
}

#[test]
fn off_11e_partner_skips_transparent_lines() {
    let src = "MARA\nhi\n\n# Act\n\n= syn\n\n===\n\n[[n]]\n\n/* b */\n\nOTTO ^\nyo";
    let els = pre(src);
    assert_eq!(els.len(), 1);
    let d = dual(&els[0]);
    assert_eq!(items(&d.right), [line('L', "yo")]);
    assert_eq!(d.left.items.len(), 6);
    assert_eq!(items(&d.left)[0], line('L', "hi"));
    let kinds: Vec<&Element> = d.left.items[1..].iter().map(interjection).collect();
    assert!(matches!(kinds[0], Element::Section(s) if s.text.plain() == "Act" && s.depth == 1));
    assert!(matches!(kinds[1], Element::Synopsis(s) if s.text.plain() == "syn"));
    assert!(matches!(kinds[2], Element::PageBreak(_)));
    assert!(matches!(kinds[3], Element::Note(n) if n.text == "n"));
    assert!(matches!(kinds[4], Element::Boneyard(b) if b.text == " b "));
    // The pair is one element of the scene and the right half has no interjections.
    assert!(
        d.right
            .items
            .iter()
            .all(|i| !matches!(i, DialogueItem::Interjection(_)))
    );
}

#[test]
fn off_11e_pair_inside_a_scene() {
    let v = view("INT. X\n\nMARA\nhi\n\n= syn\n\nOTTO ^\nyo\n\nCUT TO:\n");
    let els = &v.scenes[0].elements;
    assert_eq!(els.len(), 2);
    assert_eq!(dual(&els[0]).left.items.len(), 2);
    assert!(matches!(els[1], Element::Transition(_)));
}

#[test]
fn off_3e_view_contd_kept() {
    let b = only_block("MARA (CONT'D)\nhi");
    assert_eq!(b.cue.extensions.len(), 1);
    assert_eq!(b.cue.extensions[0].text.plain(), "CONT'D");
    assert!(b.cue.extensions[0].is_contd());
    let b = only_block("MARA (V.O.) (cont\u{2019}d)\nhi");
    let texts: Vec<String> = b.cue.extensions.iter().map(|e| e.text.plain()).collect();
    assert_eq!(texts, ["V.O.", "cont\u{2019}d"]);
    assert!(!b.cue.extensions[0].is_contd());
    assert!(b.cue.extensions[1].is_contd());
}

#[test]
fn off_1b_view_two_space_line() {
    let b = only_block("MARA\nhi\n  \nyo");
    let DialogueItem::Line(empty) = &b.items[1] else {
        panic!("a Line")
    };
    assert!(empty.runs.is_empty());
    assert_eq!(empty.span, None);
    // One space is blank and ends the block.
    let els = pre("MARA\nhi\n \nyo");
    assert_eq!(shape(&els), ["dialogue", "action"]);
}

#[test]
fn off_12_view_parenthetical() {
    let b = only_block("MARA\n  (beat) \nhi");
    assert_eq!(items(&b), [line('P', "(beat)"), line('L', "hi")]);
    let DialogueItem::Parenthetical(p) = &b.items[0] else {
        panic!("a Parenthetical")
    };
    assert_eq!(p.span, Some(Span::new(7, 13)));
    // OFF-12b: a line that is only partly in parentheses is Dialogue.
    let b = only_block("MARA\n(whispering) and then speaks\n(not closed\nparen)");
    assert_eq!(
        items(&b),
        [
            line('L', "(whispering) and then speaks"),
            line('L', "(not closed"),
            line('L', "paren)")
        ]
    );
    // OFF-12c: no cue above, so Action.
    assert_eq!(shape(&pre("a\n\n(beat)\nhi")), ["action", "action"]);
}

#[test]
fn off_21a_view_outline_in_dialogue() {
    let b = only_block("MARA\nhi\n# s\n= syn\n===\nyo");
    assert_eq!(
        items(&b),
        [
            line('L', "hi"),
            line('I', ""),
            line('I', ""),
            line('I', ""),
            line('L', "yo")
        ]
    );
    assert!(matches!(interjection(&b.items[1]), Element::Section(s) if s.text.plain() == "s"));
    assert!(matches!(interjection(&b.items[2]), Element::Synopsis(s) if s.text.plain() == "syn"));
    assert!(matches!(interjection(&b.items[3]), Element::PageBreak(_)));
    // An outline line after the block's last member is top-level.
    let els = pre("MARA\nhi\n# s\n\nb");
    assert_eq!(els.len(), 3);
    assert_eq!(block(&els[0]).items.len(), 1);
    // Transparent: the cue looks past it for its text line.
    let b = only_block("MARA\n# s\nhi");
    assert_eq!(items(&b), [line('I', ""), line('L', "hi")]);
}

#[test]
fn off_24d_view_hidden_in_dialogue() {
    let b = only_block("MARA\nhi\n[[n]]\nyo");
    assert_eq!(items(&b), [line('L', "hi"), line('I', ""), line('L', "yo")]);
    assert!(matches!(interjection(&b.items[1]), Element::Note(n) if n.text == "n"));
    let b = only_block("MARA\nhi\n/* b */\nyo");
    assert!(matches!(interjection(&b.items[1]), Element::Boneyard(n) if n.text == " b "));
    // An inline Boneyard stays a run of its line.
    let b = only_block("MARA\nhi /* x */ there");
    let DialogueItem::Line(t) = &b.items[0] else {
        panic!("a Line")
    };
    assert_eq!(t.plain(), "hi  there");
    assert!(
        matches!(&t.runs[..], [Run::Plain(a), Run::Boneyard(_), Run::Plain(c)]
        if a == "hi " && c == " there")
    );
    // A trailing Note joins the line, the parenthetical and the cue it sits on.
    let b = only_block("MARA [[c]]\nhi [[m]]\n(soft) [[p]]");
    assert!(
        matches!(&b.cue.name.runs[..], [Run::Plain(a), Run::Note(n)] if a == "MARA" && n.text == "c")
    );
    let DialogueItem::Line(t) = &b.items[0] else {
        panic!("a Line")
    };
    assert!(matches!(&t.runs[..], [Run::Plain(a), Run::Note(n)] if a == "hi " && n.text == "m"));
    let DialogueItem::Parenthetical(t) = &b.items[1] else {
        panic!("a Parenthetical")
    };
    assert!(
        matches!(&t.runs[..], [Run::Plain(a), Run::Note(n)] if a == "(soft) " && n.text == "p")
    );
    // A hidden span inside the name splits its text but not the name.
    let b = only_block("MA/*x*/RA\nhi");
    assert_eq!(b.cue.name.plain(), "MARA");
    assert_eq!(b.cue.name.runs.len(), 3);
}

#[test]
fn dialogue_view_never_panics_on_short_dialogue_input() {
    let alphabet = [
        "\n", " ", "@", "^", "(", ")", "~", "=", "#", "[", "]", "A", "a",
    ];
    let mut inputs = vec![String::new()];
    let mut last = vec![String::new()];
    for _ in 0..4 {
        let next: Vec<String> = last
            .iter()
            .flat_map(|s| alphabet.iter().map(move |c| format!("{s}{c}")))
            .collect();
        inputs.extend(next.iter().cloned());
        last = next;
    }
    for src in &inputs {
        let _ = parse(src).screenplay();
    }
}

#[test]
fn caret_pairing_is_linear_on_long_runs() {
    let n = 20_000;
    let src = "MARA\nhi\n\n".to_owned() + &"OTTO ^\nyo\n\n".repeat(n);
    let els = pre(&src);
    // Blocks 0 and 1 pair, then 2 and 3, and so on: the caret block after a pair is unpaired
    // (OFF-11d) but takes the next one as its partner.
    assert_eq!(els.len(), (n + 1).div_ceil(2));
    // Many lines to skip between one pair.
    let src = "MARA\nhi\n\n".to_owned() + &"# s\n\n".repeat(n) + "OTTO ^\nyo\n";
    let els = pre(&src);
    assert_eq!(els.len(), 1);
    assert_eq!(dual(&els[0]).left.items.len(), n + 1);
    // Many caret blocks with a heading before each: none pairs.
    let src = "INT. A\n\nMARA\nhi\n\nINT. B\n\nOTTO ^\nyo\n\n".repeat(n);
    let v = view(&src);
    assert_eq!(v.scenes.len(), 2 * n);
}
