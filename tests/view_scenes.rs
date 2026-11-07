//! The typed view's scenes and preamble (SPEC 9: "Section and Synopsis placement").

use fountain_syntax::{Element, Scene, Screenplay, Span, parse};

fn view(src: &str) -> Screenplay {
    parse(src).screenplay()
}

/// A short tag for each element, with its plain text.
fn tags(elements: &[Element]) -> Vec<String> {
    elements
        .iter()
        .map(|e| match e {
            Element::Action(a) => format!("action {}", a.text.plain()),
            Element::Section(s) => format!("section {}", s.text.plain()),
            Element::Synopsis(s) => format!("synopsis {}", s.text.plain()),
            Element::Transition(t) => format!("transition {}", t.text.plain()),
            Element::Dialogue(d) => format!("dialogue {}", d.cue.name.plain()),
            Element::Note(n) => format!("note {}", n.text),
            Element::Boneyard(b) => format!("boneyard {}", b.text),
            Element::PageBreak(_) => "page break".to_owned(),
            Element::Lyrics(_) => "lyrics".to_owned(),
            other => format!("{other:?}"),
        })
        .collect()
}

fn outline(s: &Scene) -> (Vec<String>, Vec<String>) {
    (
        s.sections.iter().map(|x| x.text.plain()).collect(),
        s.synopses.iter().map(|x| x.text.plain()).collect(),
    )
}

#[test]
fn placement_section_and_blank_line_before_heading_go_to_the_scene() {
    let src = "x\n\n# Act\n\nINT. A\n\nhi\n";
    let v = view(src);
    assert_eq!(tags(&v.preamble.elements), ["action x"]);
    let s = &v.scenes[0];
    assert_eq!(outline(s), (vec!["Act".to_owned()], vec![]));
    assert_eq!(tags(&s.elements), ["action hi"]);
    assert_eq!(s.span, Some(Span::new(3, src.len() - 1)));
}

#[test]
fn placement_synopsis_after_heading_goes_to_the_scene() {
    let v = view("INT. A\n\n= one\n\n= two\n\nx\n\n= later\n");
    let s = &v.scenes[0];
    assert_eq!(outline(s).1, ["one", "two"]);
    assert_eq!(tags(&s.elements), ["action x", "synopsis later"]);
}

#[test]
fn placement_action_between_breaks_the_run() {
    let v = view("# Act\n\nx\n\nINT. A\n");
    assert_eq!(tags(&v.preamble.elements), ["section Act", "action x"]);
    assert_eq!(outline(&v.scenes[0]), (vec![], vec![]));
    // After a heading, anything but a Synopsis ends the run of after-heading Synopses.
    let v = view("INT. A\n\n# Sec\n\n= s\n\nx\n");
    let s = &v.scenes[0];
    assert!(s.synopses.is_empty() && s.sections.is_empty());
    assert_eq!(tags(&s.elements), ["section Sec", "synopsis s", "action x"]);
}

#[test]
fn placement_outline_before_first_heading_goes_to_scene_1_not_the_preamble() {
    // The HASH2-25 shape.
    let src = "Title: T\n\n# Act One\n\n= pre\n\nINT. A\n\n= post\n\nx\n\n= before b\n\nINT. B\n";
    let v = view(src);
    assert!(v.preamble.elements.is_empty());
    let a = &v.scenes[0];
    assert_eq!(
        outline(a),
        (
            vec!["Act One".to_owned()],
            vec!["pre".to_owned(), "post".to_owned()]
        )
    );
    assert_eq!(tags(&a.elements), ["action x"]);
    assert_eq!(a.span, Some(Span::new(10, 45)));
    let b = &v.scenes[1];
    assert_eq!(outline(b), (vec![], vec!["before b".to_owned()]));
    assert_eq!(b.span, Some(Span::new(47, src.len() - 1)));
}

#[test]
fn placement_synopsis_between_two_headings_goes_to_the_next_scene() {
    // Directly after A and directly before B: the before-heading rule wins (HASH2-25).
    let src = "INT. A\n\n= s\n\nINT. B\n";
    let v = view(src);
    assert!(v.scenes[0].synopses.is_empty());
    assert_eq!(v.scenes[0].span, Some(Span::new(0, 6)));
    assert_eq!(outline(&v.scenes[1]).1, ["s"]);
    assert_eq!(v.scenes[1].span, Some(Span::new(8, src.len() - 1)));
}

#[test]
fn preamble_keeps_everything_else_in_order() {
    let v = view("[[n]]\n\nx\n\n>CUT\n\n# Act\n");
    assert_eq!(
        tags(&v.preamble.elements),
        ["note n", "action x", "transition CUT", "section Act"]
    );
    assert!(v.scenes.is_empty());
}

#[test]
fn scenes_follow_headings_and_take_the_rest() {
    let v = view("INT. A\n\nMARA\nHi.\n\nCUT TO:\n\nEXT. B\n\n/* gone */\n");
    let heads: Vec<String> = v.scenes.iter().map(|s| s.heading.text.plain()).collect();
    assert_eq!(heads, ["INT. A", "EXT. B"]);
    assert_eq!(
        tags(&v.scenes[0].elements),
        ["dialogue MARA", "transition CUT TO:"]
    );
    assert_eq!(tags(&v.scenes[1].elements), ["boneyard  gone "]);
}
