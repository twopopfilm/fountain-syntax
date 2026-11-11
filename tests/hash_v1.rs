//! Scheme 1 lines and normalised text (`SPEC.md` 11.7 and the HASH1 table of 11.9). No feature is
//! needed: the three `_v1` line methods exist in every build. Two scenes hash the same exactly
//! when their `normalized_text_v1` is the same, so the tests compare that text; the digest half
//! of the HASH1 tests is in the `hash`-gated files.

use fountain_syntax::{
    Action, Boneyard, Element, Heading, Note, PageBreak, Run, Scene, Section, Synopsis, Text, parse,
};

const H: &str = "INT. KITCHEN - DAY";

fn scenes(src: &str) -> Vec<Scene> {
    parse(src).screenplay().scenes
}

fn scene(src: &str) -> Scene {
    scenes(src).remove(0)
}

/// The scheme 1 text of the first scene of `src`.
fn text(src: &str) -> String {
    scene(src).normalized_text_v1()
}

/// The source of one scene whose body is `b`.
fn body(b: &str) -> String {
    format!("{H}\n\n{b}\n")
}

/// The scheme 1 text of the scene `H` with body `b`.
fn t(b: &str) -> String {
    text(&body(b))
}

/// The text of `H` and the given element lines.
fn want(lines: &[&str]) -> String {
    format!("{H}\n{}", lines.join("\n"))
}

/// The two bodies give the same text, so the same hash.
fn same(a: &str, b: &str) {
    assert_eq!(t(a), t(b), "{a:?} against {b:?}");
}

/// The two bodies give different texts, so different hashes.
fn differ(a: &str, b: &str) {
    assert_ne!(t(a), t(b), "{a:?} against {b:?}");
}

fn plain(s: &str) -> Text {
    Text {
        span: None,
        runs: vec![Run::Plain(s.to_string())],
    }
}

fn heading(s: &str) -> Heading {
    Heading {
        text: plain(s),
        forced: false,
        span: None,
    }
}

fn action(s: &str) -> Element {
    Element::Action(Action {
        text: plain(s),
        centered: false,
        forced: false,
        interjections: Vec::new(),
    })
}

// ---- HASH1-1 to HASH1-9: what does not change the text ----

#[test]
fn hash1_1_spaces_and_tabs() {
    assert_eq!(t("Mara waits."), want(&["Mara waits."]));
    same("Mara waits.", "Mara \t\t  waits.   ");
    same("Mara waits.", "Mara\u{a0}waits.");
    same("MARA\nHello there.", "MARA\nHello \t there.   ");
    same("MARA\n(quietly)\nHi", "MARA\n(  quietly \t)\nHi");
    same("~la la\n~lo lo", "~la \t la\n~  lo lo  ");
    same("CUT TO:", "CUT TO: \t");
    same("= a beat", "=   a \t beat  ");
    // A leading tab or run of spaces on a line of an Action paragraph.
    same("Mara waits.\nAt the door.", "Mara waits.\n\t  At the door.");
}

#[test]
fn hash1_2_line_endings() {
    let src = format!(
        "{H}\n\nMara waits,\nand waits.\n\nMARA\nHello\nthere.\n\n~la\n~lo\n\nCUT TO:\n\n= beat\n"
    );
    let lf = text(&src);
    assert_eq!(
        lf,
        want(&[
            "Mara waits, and waits.",
            "MARA",
            "Hello there.",
            "la",
            "lo",
            "CUT TO:",
            "beat"
        ])
    );
    assert_eq!(text(&src.replace('\n', "\r\n")), lf);
    assert_eq!(text(&src.replace('\n', "\r")), lf);
}

#[test]
fn hash1_3_rewrapping() {
    same("Mara waits\nby the door.", "Mara waits by the door.");
    same("Mara waits\nby\nthe door.", "Mara waits by the door.");
    same("MARA\nHello\nthere.", "MARA\nHello there.");
    assert_eq!(t("MARA\nHello\nthere."), want(&["MARA", "Hello there."]));
}

#[test]
fn hash1_4_extra_blank_lines() {
    let one = t("Mara waits.\n\nOTTO\nHi.\n\n~la\n\nCUT TO:");
    let many = t("Mara waits.\n\n\n\nOTTO\nHi.\n\n\n\n~la\n\n\n\n\nCUT TO:");
    assert_eq!(one, many);
    assert_eq!(one, want(&["Mara waits.", "OTTO", "Hi.", "la", "CUT TO:"]));
    // Blank lines before the heading and after the last element change nothing either.
    assert_eq!(
        text(&format!("\n\n\n{H}\n\n\n\nMara waits.\n\n\n\n")),
        want(&["Mara waits."])
    );
}

#[test]
fn hash1_5_hidden_and_unhashed_elements() {
    let plain_text = t("Mara waits.\n\nOTTO\nHi.");
    // Inline hidden spans.
    same(
        "Mara [[a note]]waits.\n\nOTTO\nHi.",
        "Mara waits.\n\nOTTO\nHi.",
    );
    same(
        "Mara /* gone */waits.\n\nOTTO\nHi.",
        "Mara waits.\n\nOTTO\nHi.",
    );
    same(
        "Mara waits.\n\nOTTO\nHi [[n]]there.",
        "Mara waits.\n\nOTTO\nHi there.",
    );
    // Whole-line Note, Boneyard, Section, Page Break between non-Action elements and at the ends.
    for extra in ["[[a note]]", "/* gone */", "# Act One", "===", "## Beat"] {
        assert_eq!(
            t(&format!("Mara waits.\n\n{extra}\n\nOTTO\nHi.")),
            plain_text,
            "{extra}"
        );
        assert_eq!(
            t(&format!("Mara waits.\n\nOTTO\nHi.\n\n{extra}")),
            plain_text,
            "{extra}"
        );
        assert_eq!(
            t(&format!("{extra}\n\nMara waits.\n\nOTTO\nHi.")),
            plain_text,
            "{extra}"
        );
    }
    // Inside a Dialogue block: the run continues across it.
    for extra in ["[[a note]]", "/* gone */", "# Act One", "==="] {
        assert_eq!(
            t(&format!("OTTO\nHi\n{extra}\nthere.")),
            want(&["OTTO", "Hi there."]),
            "{extra}"
        );
    }
    // Inside an Action paragraph: a Note, Boneyard, Section or Page Break does not end the group.
    for extra in ["[[a note]]", "/* gone */", "# Act One", "==="] {
        assert_eq!(
            t(&format!("Mara waits\n{extra}\nat the door.")),
            want(&["Mara waits at the door."]),
            "{extra}"
        );
    }
}

#[test]
fn hash1_6_scene_numbers() {
    let plain_scene = text(&format!("{H}\n\nMara waits.\n"));
    assert_eq!(plain_scene, want(&["Mara waits."]));
    assert_eq!(text(&format!("{H} #12A#\n\nMara waits.\n")), plain_scene);
    assert_eq!(text(&format!("{H} #1#\n\nMara waits.\n")), plain_scene);
    assert_eq!(text(&format!("{H} #7#\n\nMara waits.\n")), plain_scene);
}

#[cfg(feature = "locked-scene-numbers")]
#[test]
fn hash1_6_locked_scene_numbers() {
    let sc = scene(&format!("{H} ##12A#\n\nMara waits.\n"));
    assert!(sc.number.as_ref().is_some_and(|n| n.locked));
    assert_eq!(sc.normalized_text_v1(), want(&["Mara waits."]));
}

#[test]
fn hash1_7_case_of_heading_and_cue() {
    assert_eq!(text("int. kitchen - day\n\nHi.\n"), want(&["Hi."]));
    assert_eq!(text("Int. Kitchen - Day\n\nHi.\n"), want(&["Hi."]));
    assert_eq!(text("INT. KITCHEN - DAY\n\nHi.\n"), want(&["Hi."]));
    // A forced cue with any case gives the same cue line.
    assert_eq!(t("@Mara\nHi."), want(&["MARA", "Hi."]));
    assert_eq!(t("@mara\nHi."), want(&["MARA", "Hi."]));
    assert_eq!(t("MARA\nHi."), want(&["MARA", "Hi."]));
    // The full upper-case mapping, with no Unicode normalisation after it.
    assert_eq!(
        text("int. stra\u{df}e - day\n\nHi.\n"),
        "INT. STRASSE - DAY\nHi."
    );
}

#[test]
fn hash1_8_cue_extensions_and_caret() {
    let base = t("MARA\nHi.");
    assert_eq!(base, want(&["MARA", "Hi."]));
    same("MARA\nHi.", "MARA (CONT'D)\nHi.");
    same("MARA\nHi.", "MARA (V.O.)\nHi.");
    same("MARA\nHi.", "MARA (V.O.) (CONT'D)\nHi.");
    same("MARA (V.O.)\nHi.", "MARA (O.S.)\nHi.");
    same("MARA (V.O.)\nHi.", "MARA (CONT'D)\nHi.");
    // The dual-dialogue caret: the pair and the two separate blocks give the same lines.
    same("MARA\nHi.\n\nOTTO ^\nYo.", "MARA\nHi.\n\nOTTO\nYo.");
    same(
        "MARA\nHi.\n\nOTTO ^\nYo.",
        "MARA\nHi.\n\nOTTO (V.O.) ^\nYo.",
    );
    assert_eq!(
        t("MARA (V.O.) (CONT'D)\nHi.\n\nOTTO (O.S.) ^\nYo."),
        want(&["MARA", "Hi.", "OTTO", "Yo."])
    );
}

#[test]
fn hash1_9_forced_heading() {
    assert_eq!(text(".INT. KITCHEN - DAY\n\nHi.\n"), want(&["Hi."]));
    assert_eq!(text("INT. KITCHEN - DAY\n\nHi.\n"), want(&["Hi."]));
    assert_eq!(text(".KITCHEN\n\nHi.\n"), "KITCHEN\nHi.");
}

// ---- HASH1-10 to HASH1-14: what changes the text, and the known limit ----

#[test]
fn hash1_10_a_changed_word() {
    differ("Mara waits.", "Mara hides.");
    differ("OTTO\nHi there.", "OTTO\nHi, you.");
    differ("CUT TO:", "SMASH CUT TO:");
    differ("= a beat", "= another beat");
    differ("~la la", "~la lo");
}

#[test]
fn hash1_11_case_of_action_and_dialogue() {
    differ("Mara waits.", "MARA waits.");
    differ("Mara waits.", "mara waits.");
    differ("OTTO\nHello.", "OTTO\nhello.");
    differ("= A beat", "= a beat");
}

#[test]
fn hash1_12_emphasis_marks() {
    differ("Mara *waits*.", "Mara waits.");
    differ("Mara **waits**.", "Mara *waits*.");
    differ("Mara _waits_.", "Mara waits.");
    differ("Mara ***waits***.", "Mara **waits**.");
    differ("OTTO\nHello *there*.", "OTTO\nHello there.");
    assert_eq!(t("Mara *waits*."), want(&["Mara *waits*."]));
    assert_eq!(t("Mara _a **b** c_."), want(&["Mara _a **b** c_."]));
    // An escape is kept as typed.
    assert_eq!(t("Mara \\*waits\\*."), want(&["Mara \\*waits\\*."]));
    differ("Mara \\*waits\\*.", "Mara waits.");
}

#[test]
fn hash1_13_kind_change_with_the_same_text() {
    // The known limit G3. The Action has no Action neighbour: a neighbour would merge with it.
    assert_eq!(t("FADE OUT"), want(&["FADE OUT"]));
    same("FADE OUT", "> FADE OUT");
    same("FADE OUT", "> FADE OUT <");
    same("FADE OUT", "~FADE OUT");
    same("FADE OUT", "= FADE OUT");
    same("= CUT TO:", "> CUT TO:");
    same("~la la", "la la");
    // A cue that gains or loses an extension is the same limit.
    same("MARA\nHi.", "MARA (V.O.)\nHi.");
}

#[test]
fn hash1_14_swapped_lines() {
    differ("MARA\nHi.\n\nOTTO\nYo.", "OTTO\nYo.\n\nMARA\nHi.");
    differ(
        "MARA\nOne.\nTwo.\n\nOTTO\nYo.",
        "MARA\nOne.\nTwo.\n\nOTTO\nYo.\n\n= a",
    );
    differ("~la\n~lo", "~lo\n~la");
    differ("Mara waits.\n\nCUT TO:", "CUT TO:\n\nMara waits.");
    differ("A.\n\nB.", "B.\n\nA.");
    differ("= one\n\n= two", "= two\n\n= one");
}

// ---- HASH1-15: the boundary ----

#[test]
fn hash1_15_the_boundary() {
    // A blank line inside an Action paragraph: the paragraphs merge again.
    same("Mara waits.\nBy the door.", "Mara waits.\n\nBy the door.");
    assert_eq!(
        t("Mara waits.\n\nBy the door."),
        want(&["Mara waits. By the door."])
    );
    // A blank line inside a run of Dialogue lines splits it into Dialogue and Action.
    differ("MARA\nHello\nthere.", "MARA\nHello\n\nthere.");
    assert_eq!(
        t("MARA\nHello\n\nthere."),
        want(&["MARA", "Hello", "there."])
    );
    // A blank line under a bare cue: BEN and Hello become the one line BEN Hello.
    assert_eq!(t("BEN\nHello"), want(&["BEN", "Hello"]));
    assert_eq!(t("BEN\n\nHello"), want(&["BEN Hello"]));
    differ("BEN\nHello", "BEN\n\nHello");
    // Under a cue with an extension, the extension is in the line.
    assert_eq!(t("BEN (V.O.)\nHello"), want(&["BEN", "Hello"]));
    assert_eq!(t("BEN (V.O.)\n\nHello"), want(&["BEN (V.O.) Hello"]));
    differ("BEN (V.O.)\nHello", "BEN (V.O.)\n\nHello");
}

// ---- HASH1-16 to HASH1-18: scope and shape ----

#[test]
fn hash1_16_synopsis_scope() {
    // A Synopsis after the last element and before the next heading is in the earlier scene.
    let src = "INT. A - DAY\n\nMara waits.\n\n= beat for B\n\nINT. B - DAY\n\n= right after B\n\nOtto waits.\n";
    let sc = scenes(src);
    assert_eq!(
        sc[0].normalized_text_v1(),
        "INT. A - DAY\nMara waits.\nbeat for B"
    );
    assert_eq!(
        sc[1].normalized_text_v1(),
        "INT. B - DAY\nright after B\nOtto waits."
    );

    // A Section between does not move it: the Section is not hashed.
    let src =
        "INT. A - DAY\n\nMara waits.\n\n# Act Two\n\n= beat for B\n\nINT. B - DAY\n\nOtto waits.\n";
    let sc = scenes(src);
    assert_eq!(
        sc[0].normalized_text_v1(),
        "INT. A - DAY\nMara waits.\nbeat for B"
    );
    assert_eq!(sc[1].normalized_text_v1(), "INT. B - DAY\nOtto waits.");

    // Several trailing Synopses keep their order.
    let src = "INT. A - DAY\n\nMara waits.\n\n= one\n\n= two\n\nINT. B - DAY\n\nOtto waits.\n";
    let sc = scenes(src);
    assert_eq!(
        sc[0].normalized_text_v1(),
        "INT. A - DAY\nMara waits.\none\ntwo"
    );
    assert_eq!(sc[1].normalized_text_v1(), "INT. B - DAY\nOtto waits.");

    // A Section is not hashed, a Synopsis is.
    assert_eq!(t("# Act One\n\nMara waits."), want(&["Mara waits."]));
    assert_eq!(
        t("= a beat\n\nMara waits."),
        want(&["a beat", "Mara waits."])
    );

    // Moving a Synopsis across a heading moves it between the two scopes.
    let before = "INT. A - DAY\n\nMara waits.\n\n= beat\n\nINT. B - DAY\n\nOtto waits.\n";
    let after = "INT. A - DAY\n\nMara waits.\n\nINT. B - DAY\n\n= beat\n\nOtto waits.\n";
    assert_ne!(
        scenes(before)[0].normalized_text_v1(),
        scenes(after)[0].normalized_text_v1()
    );
    assert_ne!(
        scenes(before)[1].normalized_text_v1(),
        scenes(after)[1].normalized_text_v1()
    );
}

#[test]
fn hash1_16_a_synopsis_that_is_the_whole_body_of_a_scene() {
    let sc = scenes("INT. A - DAY\n\n= s\n\nINT. B - DAY\n\nOtto waits.\n");
    assert_eq!(sc.len(), 2);
    assert_eq!(sc[0].normalized_text_v1(), "INT. A - DAY\ns");
    assert_eq!(sc[1].normalized_text_v1(), "INT. B - DAY\nOtto waits.");
}

#[test]
fn hash1_16_the_synopsis_after_a_heading_is_in_its_own_scene() {
    let sc =
        scenes("INT. A - DAY\n\n= first\n\nMara waits.\n\nINT. B - DAY\n\n= second\n\nOtto.\n");
    assert_eq!(
        sc[0].normalized_text_v1(),
        "INT. A - DAY\nfirst\nMara waits."
    );
    assert_eq!(sc[1].normalized_text_v1(), "INT. B - DAY\nsecond\nOtto.");
}

#[test]
fn hash1_17_nothing_before_the_first_heading_is_hashed() {
    let src = "Title: Big Fish\nAuthor: Someone\n\nOpening action.\n\n= before first\n\nINT. A - DAY\n\nMara waits.\n";
    let sp = parse(src).screenplay();
    assert!(sp.title_page.is_some());
    assert!(!sp.preamble.elements.is_empty());
    assert_eq!(sp.scenes.len(), 1);
    assert_eq!(
        sp.scenes[0].normalized_text_v1(),
        "INT. A - DAY\nMara waits."
    );

    // The same without the preamble and the title page.
    assert_eq!(
        text("INT. A - DAY\n\nMara waits.\n"),
        "INT. A - DAY\nMara waits."
    );
    // A Synopsis directly before the first heading is in no scene.
    let sc = scenes("= before first\n\nINT. A - DAY\n\nMara waits.\n\nINT. B - DAY\n\nOtto.\n");
    assert_eq!(sc[0].normalized_text_v1(), "INT. A - DAY\nMara waits.");
    assert_eq!(sc[1].normalized_text_v1(), "INT. B - DAY\nOtto.");
}

#[test]
fn hash1_18_heading_only() {
    assert_eq!(text("INT. A - DAY\n"), "INT. A - DAY\n");
    assert_eq!(text(".HEADING ONLY\n"), "HEADING ONLY\n");
    assert_eq!(text(".HEADING ONLY\n").len(), 13);
    let sc = scene(".HEADING ONLY\n");
    assert_eq!(sc.heading_line_v1(), "HEADING ONLY");
    assert!(sc.element_lines_v1().is_empty());
    // A scene built in memory.
    assert_eq!(
        Scene::new(heading("heading only")).normalized_text_v1(),
        "HEADING ONLY\n"
    );
    // Two scenes with equal text: the number, the position and the span do not matter.
    let sc = scenes("INT. A - DAY\n\nMara waits.\n\nINT. A - DAY #2#\n\nMara waits.\n");
    assert_eq!(sc.len(), 2);
    assert_eq!(sc[0].normalized_text_v1(), sc[1].normalized_text_v1());
    assert_eq!(sc[0].normalized_text_v1(), "INT. A - DAY\nMara waits.");
    // A scene built in memory with the same content gives the same text.
    let mut built = Scene::new(heading("INT. A - DAY"));
    built.elements.push(action("Mara waits."));
    assert_eq!(built.normalized_text_v1(), sc[0].normalized_text_v1());
}

#[test]
fn hash1_18_a_scene_built_in_memory_uses_its_own_synopses_and_elements() {
    let mut built = Scene::new(heading("INT. A - DAY"));
    built.synopses.push(Synopsis {
        text: plain("a beat"),
    });
    built.elements.push(action("Mara waits."));
    assert_eq!(
        built.normalized_text_v1(),
        "INT. A - DAY\na beat\nMara waits."
    );
}

// ---- HASH1-19, HASH1-23 to HASH1-25: line shapes ----

#[test]
fn hash1_19_dual_dialogue() {
    assert_eq!(
        t("MARA\nHi.\n\nOTTO ^\nYo."),
        want(&["MARA", "Hi.", "OTTO", "Yo."])
    );
    assert_eq!(
        t("MARA\n(softly)\nHi\nthere.\n\nOTTO (V.O.) ^\nYo."),
        want(&["MARA", "softly", "Hi there.", "OTTO", "Yo."])
    );
    // A Synopsis between the two halves follows the left block's last line.
    assert_eq!(
        t("MARA\nHi.\n\n= beat\n\nOTTO ^\nYo."),
        want(&["MARA", "Hi.", "beat", "OTTO", "Yo."])
    );
}

#[test]
fn hash1_23_parenthetical() {
    assert_eq!(t("MARA\n(quietly)\nHi."), want(&["MARA", "quietly", "Hi."]));
    same("MARA\n(quietly)\nHi.", "MARA\n( quietly )\nHi.");
    same("MARA\n(quietly)\nHi.", "MARA\n(\tquietly\t)\nHi.");
    assert_eq!(
        t("MARA\n( very  quietly )\nHi."),
        want(&["MARA", "very quietly", "Hi."])
    );
    // A Parenthetical ends the run of Dialogue lines.
    assert_eq!(
        t("MARA\nOne\n(beat)\ntwo\nthree."),
        want(&["MARA", "One", "beat", "two three."])
    );
}

#[test]
fn hash1_24_lyrics_one_line_per_tilde_line() {
    assert_eq!(t("~la la\n~lo lo"), want(&["la la", "lo lo"]));
    assert_eq!(t("~la\n~ \n~lo"), want(&["la", "lo"]));
    assert_eq!(t("~  la \t la  "), want(&["la la"]));
    differ("~la\n~lo", "~la lo");
}

#[test]
fn hash1_25_line_shapes() {
    // A cue line is the name only, upper-cased: no extension, no caret.
    assert_eq!(
        t("Mara\n\nMARA (V.O.) (CONT'D)\nHi."),
        want(&["Mara", "MARA", "Hi."])
    );
    assert_eq!(
        t("@Mara (O.S.)\nHi.\n\nOTTO ^\nYo."),
        want(&["MARA", "Hi.", "OTTO", "Yo."])
    );
    // A centered Action has no marks.
    assert_eq!(t("> THE END <"), want(&["THE END"]));
    assert_eq!(t(">THE END<"), want(&["THE END"]));
    // A Transition, a Synopsis, Lyrics: no sigil.
    assert_eq!(t("> FADE OUT"), want(&["FADE OUT"]));
    assert_eq!(t("CUT TO:"), want(&["CUT TO:"]));
    assert_eq!(t("= a beat"), want(&["a beat"]));
    assert_eq!(t("~la"), want(&["la"]));
    // A forced Action has no `!`.
    assert_eq!(t("!MARA waits."), want(&["MARA waits."]));
    // No line has a kind prefix, and tag-like text stays in the text.
    assert_eq!(
        t("Mara pockets the #prop-coin."),
        want(&["Mara pockets the #prop-coin."])
    );
    assert_eq!(
        t("OTTO\nSee #prop-coin."),
        want(&["OTTO", "See #prop-coin."])
    );
}

#[cfg(feature = "shot")]
#[test]
fn hash1_25_shot_line_has_no_marks() {
    assert_eq!(t("!!Close on the coin."), want(&["Close on the coin."]));
    assert_eq!(
        t("!!  Close \t on the coin.  "),
        want(&["Close on the coin."])
    );
    // A Shot ends an Action group.
    assert_eq!(
        t("Mara waits.\n\n!!Close on the coin.\n\nShe leaves."),
        want(&["Mara waits.", "Close on the coin.", "She leaves."])
    );
}

// ---- HASH1-26: the line methods, in every build ----

#[test]
fn hash1_26_line_methods_agree() {
    let sc = scene(&body("Mara waits.\n\nOTTO\nHi.\n\n= a beat"));
    assert_eq!(sc.heading_line_v1(), H);
    assert_eq!(
        sc.element_lines_v1(),
        ["Mara waits.", "OTTO", "Hi.", "a beat"]
    );
    assert_eq!(
        sc.normalized_text_v1(),
        format!(
            "{}\n{}",
            sc.heading_line_v1(),
            sc.element_lines_v1().join("\n")
        )
    );
    // The heading line is collapsed and upper-cased, without the number.
    let sc = scene("int.   kitchen\t- day #4#\n\nHi.\n");
    assert_eq!(sc.heading_line_v1(), "INT. KITCHEN - DAY");
    // The methods are pure.
    assert_eq!(sc.normalized_text_v1(), sc.normalized_text_v1());
}

#[test]
fn hash2_47_v1_line_methods_without_hash() {
    // This file has no feature gate, so a plain `cargo test` builds and runs it with no `hash`.
    let sc = scene(&body("Mara waits."));
    let heading: String = sc.heading_line_v1();
    let lines: Vec<String> = sc.element_lines_v1();
    let text: String = sc.normalized_text_v1();
    assert_eq!(heading, H);
    assert_eq!(lines, ["Mara waits."]);
    assert_eq!(text, "INT. KITCHEN - DAY\nMara waits.");
}

// ---- HASH1-27: the Action merge ----

#[test]
fn hash1_27_consecutive_actions_give_one_line() {
    assert_eq!(t("A one.\n\nA two."), want(&["A one. A two."]));
    assert_eq!(
        t("A one.\n\n\n\nA two.\n\nA three."),
        want(&["A one. A two. A three."])
    );
    assert_eq!(t("A one.\nA two."), want(&["A one. A two."]));
    same("A one.\n\nA two.", "A one.\nA two.");
    same("A one.\n\nA two.", "A one.\n\n\n\nA two.");
    assert_eq!(t("  A  one.\n\n\tA   two.  "), want(&["A one. A two."]));
    // The emphasis marks of both are kept.
    assert_eq!(t("A *one*.\n\nA _two_."), want(&["A *one*. A _two_."]));
}

#[test]
fn hash1_27_a_whole_line_note_or_boneyard_does_not_end_the_group() {
    assert_eq!(
        t("A one.\n\n[[a note]]\n\nA two."),
        want(&["A one. A two."])
    );
    assert_eq!(
        t("A one.\n\n/* gone */\n\nA two."),
        want(&["A one. A two."])
    );
    assert_eq!(
        t("A one.\n\n[[a note]]\n\n/* gone */\n\nA two."),
        want(&["A one. A two."])
    );
    same("A one.\n\n[[a note]]\n\nA two.", "A one.\n\nA two.");
}

#[test]
fn hash1_27_a_note_or_boneyard_between_an_action_and_something_else() {
    // Skipped, so the group is still open, and then the Dialogue ends it.
    assert_eq!(
        t("A one.\n\n[[a note]]\n\nMARA\nHi."),
        want(&["A one.", "MARA", "Hi."])
    );
    assert_eq!(
        t("A one.\n\n/* gone */\n\nMARA\nHi.\n\nA two."),
        want(&["A one.", "MARA", "Hi.", "A two."])
    );
    // A centered Action after a Note: the group ends at the centered Action.
    assert_eq!(
        t("A one.\n\n[[a note]]\n\n> CENTRE <\n\nA two."),
        want(&["A one.", "CENTRE", "A two."])
    );
    assert_eq!(
        t("A one.\n\n/* gone */\n\n> CENTRE <"),
        want(&["A one.", "CENTRE"])
    );
    // A Note before the first Action and after the last one give nothing.
    assert_eq!(t("[[a note]]\n\nA one.\n\n[[a note]]"), want(&["A one."]));
}

#[test]
fn hash1_27_a_centered_action_ends_the_group() {
    assert_eq!(
        t("A one.\n\n> CENTRE <\n\nA two."),
        want(&["A one.", "CENTRE", "A two."])
    );
    // Two centered Actions are never merged.
    assert_eq!(t("> ONE <\n\n> TWO <"), want(&["ONE", "TWO"]));
    differ(
        "A one.\n\n> CENTRE <\n\nA two.",
        "A one.\n\nCENTRE\n\nA two.",
    );
}

#[test]
fn hash1_27_other_elements_end_the_group() {
    // A Transition.
    assert_eq!(
        t("A one.\n\nCUT TO:\n\nA two."),
        want(&["A one.", "CUT TO:", "A two."])
    );
    // Lyrics.
    assert_eq!(
        t("A one.\n\n~la\n~lo\n\nA two."),
        want(&["A one.", "la", "lo", "A two."])
    );
    // A Dialogue block.
    assert_eq!(
        t("A one.\n\nMARA\nHi.\n\nA two."),
        want(&["A one.", "MARA", "Hi.", "A two."])
    );
    // A dual pair.
    assert_eq!(
        t("A one.\n\nMARA\nHi.\n\nOTTO ^\nYo.\n\nA two."),
        want(&["A one.", "MARA", "Hi.", "OTTO", "Yo.", "A two."])
    );
    // A Section, a Page Break and a Synopsis between paragraphs.
    assert_eq!(t("A one.\n\n# Act\n\nA two."), want(&["A one.", "A two."]));
    assert_eq!(t("A one.\n\n===\n\nA two."), want(&["A one.", "A two."]));
    assert_eq!(
        t("A one.\n\n= a beat\n\nA two."),
        want(&["A one.", "a beat", "A two."])
    );
    // Adding or removing such an element changes the hash.
    differ("A one.\n\nA two.", "A one.\n\n# Act\n\nA two.");
    differ("A one.\n\nA two.", "A one.\n\n===\n\nA two.");
    differ("A one.\n\nA two.", "A one.\n\n= a beat\n\nA two.");
    differ("A one.\n\nA two.", "A one.\n\nCUT TO:\n\nA two.");
    differ("A one.\n\nA two.", "A one.\n\n> CENTRE <\n\nA two.");
    differ("A one.\n\nA two.", "A one.\n\n~la\n\nA two.");
    differ("A one.\n\nA two.", "A one.\n\nMARA\nHi.\n\nA two.");
}

#[cfg(feature = "shot")]
#[test]
fn hash1_27_a_shot_ends_the_group() {
    assert_eq!(
        t("A one.\n\n!!Close up.\n\nA two."),
        want(&["A one.", "Close up.", "A two."])
    );
}

#[test]
fn hash1_27_a_section_or_page_break_inside_a_paragraph_does_not_end_the_group() {
    assert_eq!(t("A one.\n# Act\nA two."), want(&["A one. A two."]));
    assert_eq!(t("A one.\n===\nA two."), want(&["A one. A two."]));
    same("A one.\n# Act\nA two.", "A one.\nA two.");
    same("A one.\n===\nA two.", "A one.\nA two.");
    // A paragraph that follows it still joins the group.
    assert_eq!(
        t("A one.\n# Act\nA two.\n\nA three."),
        want(&["A one. A two. A three."])
    );
    // Blank lines around it turn the interjection into an element between paragraphs (G1).
    assert_eq!(t("A one.\n\n# Act\n\nA two."), want(&["A one.", "A two."]));
    assert_eq!(t("A one.\n\n===\n\nA two."), want(&["A one.", "A two."]));
    differ("A one.\n# Act\nA two.", "A one.\n\n# Act\n\nA two.");
    differ("A one.\n===\nA two.", "A one.\n\n===\n\nA two.");
}

#[test]
fn hash1_27_a_synopsis_inside_a_paragraph_ends_the_group() {
    // The group's line first, then the Synopsis line.
    assert_eq!(
        t("A one.\n= a beat\nA two."),
        want(&["A one. A two.", "a beat"])
    );
    // Then a new group starts.
    assert_eq!(
        t("A one.\n= a beat\nA two.\n\nA three."),
        want(&["A one. A two.", "a beat", "A three."])
    );
    // The earlier paragraphs of the group are in the line.
    assert_eq!(
        t("A zero.\n\nA one.\n= a beat\nA two."),
        want(&["A zero. A one. A two.", "a beat"])
    );
    // Blank lines around the Synopsis move its line before the second paragraph (G1).
    assert_eq!(
        t("A one.\n\n= a beat\n\nA two."),
        want(&["A one.", "a beat", "A two."])
    );
    differ("A one.\n= a beat\nA two.", "A one.\n\n= a beat\n\nA two.");
}

#[test]
fn hash1_27_an_action_with_no_text_joins_and_adds_nothing() {
    let mut s = Scene::new(heading(H));
    s.elements = vec![action("A one."), action(" \t "), action("A two.")];
    assert_eq!(s.element_lines_v1(), ["A one. A two."]);

    // It does not end the group, and on its own it gives no line.
    let mut s = Scene::new(heading(H));
    s.elements = vec![action(""), action("  ")];
    assert!(s.element_lines_v1().is_empty());
    assert_eq!(s.normalized_text_v1(), format!("{H}\n"));

    // Also when the text is only a hidden span.
    let mut s = Scene::new(heading(H));
    let hidden = Element::Action(Action {
        text: Text {
            span: None,
            runs: vec![Run::Note(Note {
                text: "n".to_string(),
                span: None,
            })],
        },
        centered: false,
        forced: false,
        interjections: Vec::new(),
    });
    s.elements = vec![action("A one."), hidden, action("A two.")];
    assert_eq!(s.element_lines_v1(), ["A one. A two."]);
}

#[test]
fn hash1_27_whole_line_elements_built_in_memory() {
    let note = Element::Note(Note {
        text: "n".to_string(),
        span: None,
    });
    let boneyard = Element::Boneyard(Boneyard {
        text: "b".to_string(),
        span: None,
    });
    let section = Element::Section(Section {
        depth: 1,
        text: plain("Act"),
    });
    let page_break = Element::PageBreak(PageBreak { span: None });
    let mut s = Scene::new(heading(H));
    s.elements = vec![
        action("A one."),
        note.clone(),
        boneyard.clone(),
        action("A two."),
        section,
        action("A three."),
        page_break,
        action("A four."),
        note,
        boneyard,
    ];
    assert_eq!(
        s.element_lines_v1(),
        ["A one. A two.", "A three.", "A four."]
    );
}

#[test]
fn hash1_27_the_last_group_is_flushed() {
    // The scope ends with an open group.
    assert_eq!(t("A one.\n\nA two."), want(&["A one. A two."]));
    assert_eq!(
        t("MARA\nHi.\n\nA one.\n\nA two."),
        want(&["MARA", "Hi.", "A one. A two."])
    );
    // A trailing Synopsis (the next scene's lead-in) ends it: the group's line, then the Synopsis.
    let sc =
        scenes("INT. A - DAY\n\nA one.\n\nA two.\n\n= next\n\nINT. B - DAY\n\nB one.\n\nB two.\n");
    assert_eq!(
        sc[0].normalized_text_v1(),
        "INT. A - DAY\nA one. A two.\nnext"
    );
    // A group does not leak into the next scene.
    assert_eq!(sc[1].normalized_text_v1(), "INT. B - DAY\nB one. B two.");
    let sc = scenes("INT. A - DAY\n\nA one.\n\nINT. B - DAY\n\nB one.\n");
    assert_eq!(sc[0].normalized_text_v1(), "INT. A - DAY\nA one.");
    assert_eq!(sc[1].normalized_text_v1(), "INT. B - DAY\nB one.");
}

#[test]
fn hash1_27_a_transition_between_actions_that_becomes_action() {
    // G3's exception: the merge changes the lines.
    assert_eq!(
        t("A one.\n\nCUT TO:\n\nA two."),
        want(&["A one.", "CUT TO:", "A two."])
    );
    assert_eq!(
        t("A one.\n\n!CUT TO:\n\nA two."),
        want(&["A one. CUT TO: A two."])
    );
    differ(
        "A one.\n\nCUT TO:\n\nA two.",
        "A one.\n\n!CUT TO:\n\nA two.",
    );
}

// ---- Dialogue runs and interjections (SPEC 11.7, rule 4) ----

#[test]
fn dialogue_interjections_follow_the_run_they_interrupt() {
    // The run continues across a Synopsis; the Synopsis line comes after the run's line.
    assert_eq!(
        t("MARA\nHi\n= a beat\nthere."),
        want(&["MARA", "Hi there.", "a beat"])
    );
    // A Lyrics line inside the block is held the same way.
    assert_eq!(
        t("MARA\nHi\n~la\nthere."),
        want(&["MARA", "Hi there.", "la"])
    );
    // A Parenthetical closes the run, and the held line comes out before the Parenthetical.
    assert_eq!(
        t("MARA\nHi\n= a beat\n(quietly)\nthere."),
        want(&["MARA", "Hi", "a beat", "quietly", "there."])
    );
    // With no open run an interjection comes out at once.
    assert_eq!(t("MARA\n= a beat\nHi."), want(&["MARA", "a beat", "Hi."]));
    // Several held interjections keep their order.
    assert_eq!(
        t("MARA\nHi\n= one\n~la\n= two\nthere."),
        want(&["MARA", "Hi there.", "one", "la", "two"])
    );
    // A Section, a Page Break, a Note and a Boneyard give nothing.
    assert_eq!(
        t("MARA\nHi\n# Act\n===\n[[n]]\n/* x */\nthere."),
        want(&["MARA", "Hi there."])
    );
}

#[test]
fn lines_with_only_hidden_text_are_dropped() {
    // A Dialogue line with only a hidden span gives no run.
    assert_eq!(t("MARA\n[[n]]\nHi."), want(&["MARA", "Hi."]));
    // Lines whose text is only white space or a hidden span are dropped.
    assert_eq!(t("~ [[n]]\n~la"), want(&["la"]));
}
