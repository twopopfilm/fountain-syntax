//! The typed view's elements and title page (SPEC 4, 9), read through `parse`.

mod common;

use fountain_syntax::{
    Action, Element, Lyrics, Note, NoteConvention, Run, Scene, Screenplay, Span, Text, TitleEntry,
    TitlePage, Transition, parse,
};

fn view(src: &str) -> Screenplay {
    parse(src).screenplay()
}

/// The elements of the preamble.
fn pre(src: &str) -> Vec<Element> {
    view(src).preamble.elements
}

fn only_scene(src: &str) -> Scene {
    let mut s = view(src).scenes;
    assert_eq!(s.len(), 1, "{src:?}");
    s.remove(0)
}

fn action(e: &Element) -> &Action {
    match e {
        Element::Action(a) => a,
        other => panic!("not Action: {other:?}"),
    }
}

fn transition(e: &Element) -> &Transition {
    match e {
        Element::Transition(t) => t,
        other => panic!("not Transition: {other:?}"),
    }
}

fn lyrics(e: &Element) -> &Lyrics {
    match e {
        Element::Lyrics(l) => l,
        other => panic!("not Lyrics: {other:?}"),
    }
}

fn title_page(src: &str) -> TitlePage {
    view(src).title_page.expect("a title page")
}

fn plains(texts: &[Text]) -> Vec<String> {
    texts.iter().map(Text::plain).collect()
}

fn note_run(text: &str, start: usize, end: usize) -> Run {
    Run::Note(fountain_syntax::Note {
        text: text.to_owned(),
        span: Some(Span::new(start, end)),
    })
}

/// The runs of the preamble's first Action.
fn runs(src: &str) -> Vec<Run> {
    action(&pre(src)[0]).text.runs.clone()
}

fn plain_run(s: &str) -> Run {
    Run::Plain(s.to_owned())
}

fn em(bold: bool, italic: bool, underline: bool, runs: Vec<Run>) -> Run {
    Run::Emphasis {
        bold,
        italic,
        underline,
        runs,
    }
}

#[test]
fn el_heading_1_each_prefix_view() {
    for prefix in ["INT.", "EXT.", "EST.", "INT./EXT.", "INT/EXT.", "I/E."] {
        for p in [prefix.to_owned(), prefix.to_lowercase()] {
            let src = format!("{p} HOUSE - DAY\n");
            let s = only_scene(&src);
            assert_eq!(s.heading.text.plain(), format!("{p} HOUSE - DAY"));
            assert!(!s.heading.forced);
            assert_eq!(s.heading.span, Some(Span::new(0, src.len() - 1)));
            assert!(s.number.is_none());
        }
    }
}

#[test]
fn el_heading_2_forced_heading_view() {
    let s = only_scene("x\n.LAMP ROOM\n");
    assert_eq!(s.heading.text.plain(), "LAMP ROOM");
    assert_eq!(s.heading.text.span, Some(Span::new(3, 12)));
    assert!(s.heading.forced);
}

#[test]
fn el_heading_3_scene_number_view() {
    let src = "INT. A - DAY #12A#\n";
    let s = only_scene(src);
    assert_eq!(s.heading.text.plain(), "INT. A - DAY");
    let n = s.number.expect("a number");
    assert_eq!(n.text, "12A");
    assert_eq!(n.span, Some(Span::new(13, 18)));
}

#[test]
fn off_8f_view_number_apart_from_text() {
    let src = ".TITLE CARD #3#  \n";
    let doc = parse(src);
    let s = doc.screenplay().scenes.remove(0);
    assert_eq!(s.heading.text.plain(), "TITLE CARD");
    assert_eq!(s.number.map(|n| n.text).as_deref(), Some("3"));
    assert_eq!(doc.source(), src, "the tree keeps both");
}

#[test]
fn el_action_1_paragraph_keeps_lines() {
    let e = pre("He waits.\nShe goes.\r\nEnd.\n");
    assert_eq!(e.len(), 1);
    let a = action(&e[0]);
    assert_eq!(a.text.plain(), "He waits.\nShe goes.\r\nEnd.");
    assert_eq!(a.text.span, Some(Span::new(0, 25)));
    assert!(!a.centered && !a.forced && a.interjections.is_empty());
}

#[test]
fn el_action_2_indentation_kept() {
    let e = pre("  He waits. \n\tShe goes.\t\n");
    assert_eq!(action(&e[0]).text.plain(), "  He waits.\n\tShe goes.");
}

#[test]
fn el_action_3_forced_and_joined_lines() {
    let e = pre("!FORCED\nnext line\n");
    let a = action(&e[0]);
    assert!(a.forced);
    assert_eq!(a.text.plain(), "FORCED\nnext line");
}

#[test]
fn el_action_3_only_the_first_line_sigil_forces() {
    let forced_and_text = |src: &str| {
        let e = pre(src);
        let a = action(&e[0]);
        (a.forced, a.text.plain())
    };
    assert_eq!(forced_and_text("a\n!b\n"), (false, "a\nb".to_owned()));
    assert_eq!(forced_and_text("!a\n!b\n"), (true, "a\nb".to_owned()));
    assert_eq!(forced_and_text("!a\nb\n"), (true, "a\nb".to_owned()));
}

#[test]
fn off_22a_indented_or_hidden_before_sigil_still_forces() {
    let forced = |src: &str| action(&pre(src)[0]).forced;
    for src in ["  !x\n", "\t!x\n", "[[n]] !x\n", "  !a\n  b\n", "[[n]]!x\n"] {
        assert!(forced(src), "{src:?}");
    }
    assert!(!forced("a\n  !b\n"));
}

/// OFF-22a: the indentation before a forcing `!` stays in the Action text; only the `!` leaves it.
#[test]
fn off_22a_indent_before_sigil_kept_in_text() {
    let a = |src: &str| {
        let e = pre(src);
        let act = action(&e[0]);
        (act.forced, act.text.plain())
    };
    assert_eq!(
        a("  !indented action\n"),
        (true, "  indented action".to_owned())
    );
    assert_eq!(
        a("\t!tabbed action\n"),
        (true, "\ttabbed action".to_owned())
    );
}

#[test]
fn action_lines_of_only_hidden_spans_add_no_text() {
    // A forced one-line Action holding only a Note: its whitespace goes, the Note stays.
    let e = pre("! [[n]] \n");
    let a = action(&e[0]);
    assert!(a.forced);
    assert_eq!(a.text.plain(), "");
    assert_eq!(a.text.runs, [note_run("n", 2, 7)]);
    // Two hidden-only lines in a row leave no blank line.
    let e = pre("a\n[[n]]\n[[m]]\nb\n");
    assert_eq!(action(&e[0]).text.plain(), "a\nb");
}

#[test]
fn off_18b_indent_kept_only_in_action() {
    let src =
        "Title:  x  \n\n  INT. A  \n\n  > CUT  \n\n  # Sec  \n\n  = syn  \n\n  ~ la  \n\n  Act  \n";
    let v = view(src);
    assert_eq!(v.title_page.unwrap().entries[0].values[0].plain(), "x");
    let s = &v.scenes[0];
    assert_eq!(s.heading.text.plain(), "INT. A");
    let e = &s.elements;
    assert_eq!(transition(&e[0]).text.plain(), "CUT");
    let Element::Section(sec) = &e[1] else {
        panic!("{e:?}")
    };
    assert_eq!(sec.text.plain(), "Sec");
    let Element::Synopsis(syn) = &e[2] else {
        panic!("{e:?}")
    };
    assert_eq!(syn.text.plain(), "syn");
    assert_eq!(plains(&lyrics(&e[3]).lines), ["la"]);
    assert_eq!(action(&e[4]).text.plain(), "  Act");
}

#[test]
fn el_centered_1_marks_flush() {
    let e = pre(">THE END<\n");
    let a = action(&e[0]);
    assert!(a.centered && !a.forced);
    assert_eq!(a.text.plain(), "THE END");
}

#[test]
fn el_centered_2_inner_spaces_and_trailing_whitespace() {
    let e = pre(">  THE END  < \n");
    let a = action(&e[0]);
    assert!(a.centered);
    assert_eq!(a.text.plain(), "THE END");
    assert_eq!(a.text.span, Some(Span::new(3, 10)));
}

#[test]
fn off_10_view_centered_trimmed() {
    let e = pre(">> FIN <<\n\n> FIN\n\n>FIN< and more\n");
    assert!(action(&e[0]).centered);
    assert_eq!(action(&e[0]).text.plain(), "> FIN <");
    assert_eq!(transition(&e[1]).text.plain(), "FIN");
    assert_eq!(transition(&e[2]).text.plain(), "FIN< and more");
}

#[test]
fn el_transition_1_automatic() {
    let v = view("INT. A\n\nCUT TO:\n\nINT. B\n");
    assert_eq!(v.scenes.len(), 2);
    let t = transition(&v.scenes[0].elements[0]);
    assert_eq!(t.text.plain(), "CUT TO:");
    assert!(!t.forced);
}

#[test]
fn el_transition_2_forced() {
    let e = pre(">fade away\n");
    let t = transition(&e[0]);
    assert!(t.forced);
    assert_eq!(t.text.plain(), "fade away");
}

#[test]
fn el_lyrics_1_run_of_lines() {
    let src = "~la la\n ~ lo lo \n[[n]]\n~\n";
    let e = pre(src);
    assert_eq!(e.len(), 1);
    let l = lyrics(&e[0]);
    assert_eq!(plains(&l.lines), ["la la", "lo lo", ""]);
    // The own-line Note joins the line before it, and makes no line of its own.
    assert_eq!(
        l.lines[1].runs,
        [Run::Plain("lo lo".to_owned()), note_run("n", 17, 22)]
    );
}

#[test]
fn el_section_1_depths() {
    let e = pre("# A\n\n## B\n\n### C\n\n####D\n");
    let got: Vec<(usize, String)> = e
        .iter()
        .map(|e| match e {
            Element::Section(s) => (s.depth, s.text.plain()),
            other => panic!("{other:?}"),
        })
        .collect();
    let want = [(1, "A"), (2, "B"), (3, "C"), (4, "D")].map(|(d, t)| (d, t.to_owned()));
    assert_eq!(got, want);
}

#[test]
fn el_synopsis_1_after_heading() {
    let s = only_scene("INT. A\n\n= the set-up\n\nx\n");
    assert_eq!(plains(&[s.synopses[0].text.clone()]), ["the set-up"]);
    assert_eq!(s.synopses.len(), 1);
    assert_eq!(action(&s.elements[0]).text.plain(), "x");
}

#[test]
fn el_page_break_1_three_and_more() {
    let e = pre("===\n\n=====\n");
    let spans: Vec<Option<Span>> = e
        .iter()
        .map(|e| match e {
            Element::PageBreak(p) => p.span,
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(spans, [Some(Span::new(0, 3)), Some(Span::new(5, 10))]);
}

#[test]
fn el_note_1_inline_and_own_line() {
    let src = "She [[a note]] goes.\n\n[[own\nline]]\n";
    let e = pre(src);
    let a = action(&e[0]);
    assert_eq!(a.text.plain(), "She  goes.");
    assert_eq!(
        a.text.runs,
        [
            Run::Plain("She ".to_owned()),
            note_run("a note", 4, 14),
            Run::Plain(" goes.".to_owned())
        ]
    );
    let Element::Note(n) = &e[1] else {
        panic!("{e:?}")
    };
    assert_eq!(n.text, "own\nline");
    assert_eq!(n.span, Some(Span::new(22, 34)));
}

#[test]
fn el_note_1_own_line_note_inside_action_adds_no_line() {
    let e = pre("x [[a]]\n[[b]]\ny\n");
    let a = action(&e[0]);
    // The space before the inline Note stays, as in OFF-16a; the Note line adds no blank line.
    assert_eq!(a.text.plain(), "x \ny");
    assert_eq!(
        a.text.runs,
        [
            Run::Plain("x ".to_owned()),
            note_run("a", 2, 7),
            Run::Plain("\n".to_owned()),
            note_run("b", 8, 13),
            Run::Plain("y".to_owned()),
        ]
    );
}

#[test]
fn el_boneyard_1_hides_a_whole_scene() {
    let src = "INT. A\n\n/* INT. B\n\nhid */\n\nINT. C\n";
    let v = view(src);
    let heads: Vec<String> = v.scenes.iter().map(|s| s.heading.text.plain()).collect();
    assert_eq!(heads, ["INT. A", "INT. C"]);
    let Element::Boneyard(b) = &v.scenes[0].elements[0] else {
        panic!("{v:?}")
    };
    assert_eq!(b.text, " INT. B\n\nhid ");
    assert_eq!(b.span, Some(Span::new(8, 25)));
    assert!(
        parse(src).root().text().contains("INT. B"),
        "kept in the tree"
    );
}

#[test]
fn off_16a_view_plain_text() {
    let e = pre("Mara /* aside */ waits.\n");
    let a = action(&e[0]);
    assert_eq!(a.text.plain(), "Mara  waits.");
    assert!(matches!(&a.text.runs[1], Run::Boneyard(b) if b.text == " aside "));
}

/// EL-action-4
#[test]
fn el_action_4_tab_expansion_is_display_only() {
    let src = "\tHe waits.\tAlone.\n";
    let doc = parse(src);
    let v = doc.screenplay();
    let a = action(&v.preamble.elements[0]);
    assert_eq!(a.display(), "    He waits.    Alone.");
    assert_eq!(a.text.plain(), "\tHe waits.\tAlone.");
    assert_eq!(doc.write(), src);
}

/// EL-emphasis-1
#[test]
fn el_emphasis_1_each_style_and_nesting() {
    assert_eq!(runs("*a*"), [em(false, true, false, vec![plain_run("a")])]);
    assert_eq!(
        runs("**a**"),
        [em(true, false, false, vec![plain_run("a")])]
    );
    assert_eq!(
        runs("***a***"),
        [em(true, true, false, vec![plain_run("a")])]
    );
    assert_eq!(runs("_a_"), [em(false, false, true, vec![plain_run("a")])]);
    assert_eq!(
        runs("*a **b** c*"),
        [em(
            false,
            true,
            false,
            vec![
                plain_run("a "),
                em(true, false, false, vec![plain_run("b")]),
                plain_run(" c"),
            ]
        )]
    );
    // Underline around italic.
    assert_eq!(
        runs("_*x*_"),
        [em(
            false,
            false,
            true,
            vec![em(false, true, false, vec![plain_run("x")])]
        )]
    );
    // Italic inside bold.
    assert_eq!(
        runs("**a *b* c**"),
        [em(
            true,
            false,
            false,
            vec![
                plain_run("a "),
                em(false, true, false, vec![plain_run("b")]),
                plain_run(" c"),
            ]
        )]
    );
}

/// EL-emphasis-2
#[test]
fn el_emphasis_2_escapes_and_unmatched_literal() {
    assert_eq!(
        runs("\\*not italic*"),
        [Run::Escaped('*'), plain_run("not italic*")]
    );
    assert_eq!(runs("*open"), [plain_run("*open")]);
    assert_eq!(runs("****a****"), [plain_run("****a****")]);
    assert_eq!(runs("__a__"), [plain_run("__a__")]);
}

/// OFF-14b
#[test]
fn off_14b_no_span_across_line_break() {
    assert_eq!(runs("*across\nlines*"), [plain_run("*across\nlines*")]);
    assert_eq!(
        runs("*a*\n*b*"),
        [
            em(false, true, false, vec![plain_run("a")]),
            plain_run("\n"),
            em(false, true, false, vec![plain_run("b")]),
        ]
    );
}

#[test]
fn off_14f_bold_heading_is_action() {
    let v = view("\n**INT. X**\n\n");
    assert!(v.scenes.is_empty());
    let a = action(&v.preamble.elements[0]);
    assert_eq!(
        a.text.runs,
        [Run::Emphasis {
            bold: true,
            italic: false,
            underline: false,
            runs: vec![Run::Plain("INT. X".to_owned())],
        }]
    );
}

#[test]
fn off_21b_view_interjections_in_action() {
    let e = pre("a\n  # sec\nb\n= syn\n===\nc\n");
    assert_eq!(e.len(), 1);
    let a = action(&e[0]);
    assert_eq!(a.text.plain(), "a\nb\nc");
    match a.interjections.as_slice() {
        [
            Element::Section(s),
            Element::Synopsis(y),
            Element::PageBreak(p),
        ] => {
            assert_eq!((s.depth, s.text.plain()), (1, "sec".to_owned()));
            assert_eq!(y.text.plain(), "syn");
            assert_eq!(p.span, Some(Span::new(18, 21)));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn off_22d_view_sigils_left_out() {
    let v = view(".HEAD\n\n!act\n\n~lyr\n\n>tr\n\n>ce<\n\n@Name\nhi\n");
    assert_eq!(v.scenes[0].heading.text.plain(), "HEAD");
    let e = &v.scenes[0].elements;
    assert_eq!(action(&e[0]).text.plain(), "act");
    assert_eq!(plains(&lyrics(&e[1]).lines), ["lyr"]);
    assert_eq!(transition(&e[2]).text.plain(), "tr");
    assert_eq!(action(&e[3]).text.plain(), "ce");
    let Element::Dialogue(d) = &e[4] else {
        panic!("{e:?}")
    };
    assert_eq!(d.cue.name.plain(), "Name");
    assert!(d.cue.forced);
}

#[test]
fn el_punctuation_1_passes_through() {
    let line = "Wait... what?! -- \"Yes\" (no); 50% & more: done. 'Q' / \\ [x] {y} <z> ~ # = @ ^";
    let e = pre(&format!("{line}\n"));
    assert_eq!(action(&e[0]).text.plain(), line);
}

#[test]
fn el_title_page_1_inline_values() {
    let src = "Title: Big Fish\nCredit: Written by\nAuthor: Me\n\nINT. A\n";
    let p = title_page(src);
    let got: Vec<(String, Vec<String>)> = p
        .entries
        .iter()
        .map(|e| (e.key.plain(), plains(&e.values)))
        .collect();
    let want = [
        ("Title", "Big Fish"),
        ("Credit", "Written by"),
        ("Author", "Me"),
    ]
    .map(|(k, v)| (k.to_owned(), vec![v.to_owned()]));
    assert_eq!(got, want);
    assert_eq!(p.span, Some(Span::new(0, 45)));
    assert_eq!(p.entries[0].span, Some(Span::new(0, 15)));
    assert!(p.stray_lines.is_empty());
}

#[test]
fn el_title_page_2_indented_multi_line_values() {
    let p = title_page("Title: Big\n   Fish\nContact:\n\tAddr 1\n    Addr 2\n");
    assert_eq!(plains(&p.entries[0].values), ["Big", "Fish"]);
    assert_eq!(plains(&p.entries[1].values), ["Addr 1", "Addr 2"]);
}

#[test]
fn el_title_page_3_typed_access_through_parse() {
    let p = title_page("TITLE: a\nRevision: r\ntitle: b\n");
    let titles: Vec<String> = p.get("Title").map(|e| e.values[0].plain()).collect();
    assert_eq!(titles, ["a", "b"]);
    assert_eq!(p.title().map(|e| e.key.plain()).as_deref(), Some("TITLE"));
    assert_eq!(p.get("revision").count(), 1);
}

/// EXT-open-keys-2
#[test]
fn ext_open_keys_2_typed_accessors_ignore_case() {
    let p = title_page(
        "AUTHORS: A and B\ndraft DATE: 1 May\nnotes: n\nCOPYRIGHT: c\ndate: d\n\nINT. A\n",
    );
    let first = |e: Option<&TitleEntry>| e.map(|e| e.values[0].plain());
    assert_eq!(first(p.author()).as_deref(), Some("A and B"));
    assert_eq!(first(p.draft_date()).as_deref(), Some("1 May"));
    assert_eq!(first(p.notes()).as_deref(), Some("n"));
    assert_eq!(first(p.copyright()).as_deref(), Some("c"));
    assert_eq!(first(p.date()).as_deref(), Some("d"));
    let p = title_page("Author: Me\n\nINT. A\n");
    assert_eq!(first(p.author()).as_deref(), Some("Me"));
}

#[test]
fn el_title_page_4_body_after_first_blank_line() {
    let v = view("Title: x\n\nAuthor: y\n\nINT. A\n");
    assert_eq!(v.title_page.unwrap().entries.len(), 1);
    assert_eq!(action(&v.preamble.elements[0]).text.plain(), "Author: y");
    assert_eq!(v.scenes.len(), 1);
}

#[test]
fn off_17d_view_stray_lines() {
    let p = title_page("Title: x\n  stray one\n  stray [[n]] two\nAuthor: y\n");
    assert_eq!(plains(&p.stray_lines), ["stray one", "stray  two"]);
    assert_eq!(p.entries.len(), 2);
    assert_eq!(plains(&p.entries[1].values), ["y"]);
}

#[test]
fn title_page_hidden_spans_join_the_text_before_them() {
    let p = title_page("Title: [[n]] x [[m]]\n[[own]]\nAuthor: y\n");
    let t = &p.entries[0];
    assert!(matches!(t.key.runs.as_slice(), [Run::Plain(_), Run::Note(n)] if n.text == "n"));
    assert!(
        matches!(t.values[0].runs.as_slice(), [Run::Plain(_), Run::Note(m), Run::Note(o)]
            if m.text == "m" && o.text == "own")
    );
    // A Note before the key joins the key.
    let p = title_page("[[k]] Title: x\n");
    assert!(
        matches!(p.entries[0].key.runs.as_slice(), [Run::Note(_), Run::Plain(k)] if k == "Title")
    );
}

#[test]
fn ext_open_keys_1_view_keeps_keys_in_order() {
    let p = title_page("TC: a\ntitle: b\nTitle: c\nMy Key: d\nTC: e\n");
    let keys: Vec<String> = p.entries.iter().map(|e| e.key.plain()).collect();
    assert_eq!(keys, ["TC", "title", "Title", "My Key", "TC"]);
}

#[test]
fn bom_is_skipped() {
    let v = view("\u{feff}Title: x\n\nINT. A\n");
    assert_eq!(v.title_page.unwrap().entries[0].key.plain(), "Title");
    assert_eq!(v.scenes.len(), 1);
    assert_eq!(view(""), view("\n\n"));
    assert!(view("").title_page.is_none());
}

#[cfg(feature = "shot")]
#[test]
fn ext_shot_1_view() {
    let e = pre("!!close on hands\n");
    let Element::Shot(s) = &e[0] else {
        panic!("{e:?}")
    };
    assert_eq!(s.text.plain(), "close on hands");
}

#[cfg(feature = "locked-scene-numbers")]
#[test]
fn ext_locked_scene_numbers_1_view() {
    let v = view("INT. A - DAY ##12A#\n\nINT. B #3#\n");
    let s = &v.scenes;
    assert_eq!(s[0].heading.text.plain(), "INT. A - DAY");
    let n = s[0].number.as_ref().expect("locked number");
    assert_eq!((n.text.as_str(), n.locked), ("12A", true));
    let n = s[1].number.as_ref().expect("number");
    assert_eq!((n.text.as_str(), n.locked), ("3", false));
}

#[test]
fn screenplay_never_panics_on_short_input() {
    let alphabet = [
        "\n", "\r", " ", "\t", "[", "]", "/", "*", "_", "#", "=", "~", "!", "@", ".", ">", "<",
        "^", "(", ")", ":", "A", "a",
    ];
    let mut inputs = vec![String::new()];
    let mut last = vec![String::new()];
    for _ in 0..3 {
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
fn screenplay_is_linear_on_long_runs() {
    let n = 50_000;
    let src = "# s\n\n".repeat(n) + "INT. A\n\n" + &"a\n# x\n".repeat(n) + "b\n";
    let v = view(&src);
    assert_eq!(v.scenes[0].sections.len(), n);
    assert_eq!(action(&v.scenes[0].elements[0]).interjections.len(), n);
}

/// The convention of a note with this text.
fn conv(text: &str) -> Option<NoteConvention> {
    Note {
        text: text.to_owned(),
        span: None,
    }
    .conventions()
}

fn s(x: &str) -> String {
    x.to_owned()
}

#[test]
fn ext_notes_view_1_colour_marker_tag_none() {
    // Colours: a word, `color` and a word, `#` and six hex digits. The word stays as typed.
    assert_eq!(conv("teal"), Some(NoteConvention::Colour(s("teal"))));
    assert_eq!(conv("Teal"), Some(NoteConvention::Colour(s("Teal"))));
    assert_eq!(conv("color teal"), Some(NoteConvention::Colour(s("teal"))));
    assert_eq!(conv("COLOUR Teal"), Some(NoteConvention::Colour(s("Teal"))));
    assert_eq!(
        conv(" #a1b2c3 "),
        Some(NoteConvention::Colour(s("#a1b2c3")))
    );
    assert_eq!(conv("#A1B2C3"), Some(NoteConvention::Colour(s("#A1B2C3"))));
    assert_eq!(conv("été"), Some(NoteConvention::Colour(s("été"))));
    // Not colours: wrong hex length or digit, or more than one word.
    for not in [
        "#12345",
        "#a1b2c3d",
        "#a1b2cg",
        "#",
        "teal green",
        "color a b",
    ] {
        assert_eq!(conv(not), None, "{not:?}");
    }

    // Markers: `marker`, an optional colour word directly before a colon, the text.
    let marker = |colour: Option<&str>, text: &str| NoteConvention::Marker {
        colour: colour.map(s),
        text: s(text),
    };
    assert_eq!(
        conv("marker amber: fix pacing"),
        Some(marker(Some("amber"), "fix pacing"))
    );
    assert_eq!(conv("Marker: fix"), Some(marker(None, "fix")));
    assert_eq!(conv("marker fix pacing"), Some(marker(None, "fix pacing")));
    assert_eq!(
        conv("marker call at 12:30"),
        Some(marker(None, "call at 12:30"))
    );
    assert_eq!(conv("marker"), Some(marker(None, "")));
    assert_eq!(conv("marker teal:"), Some(marker(Some("teal"), "")));
    assert_eq!(conv("markers are fun"), None);

    // Tags: one word and a colon at the start.
    let tag = |t: &str, text: &str| NoteConvention::Tag {
        tag: s(t),
        text: s(text),
    };
    assert_eq!(
        conv("Check: confirm tides"),
        Some(tag("Check", "confirm tides"))
    );
    assert_eq!(conv("TODO:later"), Some(tag("TODO", "later")));
    assert_eq!(conv("À faire: café"), None, "two words before the colon");
    assert_eq!(conv("Vérifier: café"), Some(tag("Vérifier", "café")));
    assert_eq!(conv("Check:"), Some(tag("Check", "")));

    // Neither.
    assert_eq!(conv("remember to call the bank"), None);
    assert_eq!(conv(""), None);
    assert_eq!(conv("   \t\n "), None);
    assert_eq!(conv(":"), None);
    assert_eq!(conv(": text"), None);

    // Through the view.
    let v = view("[[Check: confirm tides]]\n");
    let Element::Note(n) = &v.preamble.elements[0] else {
        panic!("{v:?}")
    };
    assert_eq!(n.conventions(), Some(tag("Check", "confirm tides")));
}

/// Every Note of the elements, whole-line and inline in an Action.
fn notes_of<'a>(elements: &'a [Element], out: &mut Vec<&'a Note>) {
    for e in elements {
        match e {
            Element::Note(n) => out.push(n),
            Element::Action(a) => {
                out.extend(a.text.runs.iter().filter_map(|r| match r {
                    Run::Note(n) => Some(n),
                    _ => None,
                }));
                notes_of(&a.interjections, out);
            }
            _ => {}
        }
    }
}

#[test]
fn ext_notes_view_2_conventions_change_nothing() {
    let src = "[[teal]]\n\nINT. A\n\nHe waits [[Check: tides]] and [[marker amber: fix]] and \
               [[just a thought]].\n\n[[color teal]]\n\n[[]]\n";
    let doc = parse(src);
    let before_view = doc.screenplay();
    let before_tree = common::dump(doc.root());
    let mut notes = Vec::new();
    notes_of(&before_view.preamble.elements, &mut notes);
    for sc in &before_view.scenes {
        notes_of(&sc.elements, &mut notes);
    }
    assert!(notes.len() >= 5, "{notes:?}");
    let found = notes.iter().filter(|n| n.conventions().is_some()).count();
    assert!(found >= 3, "{found}");
    assert_eq!(doc.screenplay(), before_view);
    assert_eq!(common::dump(doc.root()), before_tree);
    assert_eq!(doc.source(), src);
    // The hash half of EXT-notes-view-2 (the same hashes with or without the call) waits for #10.
}

#[test]
fn conventions_never_panic_on_short_text() {
    let alphabet = ["m", "a", "r", "#", ":", " ", "é", "0", "f", "k"];
    let mut inputs = vec![String::new()];
    let mut last = vec![String::new()];
    for _ in 0..3 {
        let next: Vec<String> = last
            .iter()
            .flat_map(|x| alphabet.iter().map(move |c| format!("{x}{c}")))
            .collect();
        inputs.extend(next.iter().cloned());
        last = next;
    }
    for text in &inputs {
        let _ = conv(text);
    }
    let _ = conv("marke");
    let _ = conv(&"é".repeat(100_000));
    let _ = conv(&format!("marker {}:", "é ".repeat(50_000)));
}
