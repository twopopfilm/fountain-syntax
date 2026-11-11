#![cfg(not(any(feature = "shot", feature = "locked-scene-numbers")))]
//! Diffs the typed view with screenplay-tools' reading of the corpus (issue #16).

mod common;

use fountain_syntax::{DialogueBlock, DialogueItem, Element, Run, Scene, Screenplay, Text, parse};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::path::Path;

/// U+23CE stands for an embedded line break, as in `scripts/screenplay-tools-diff.py`.
const BREAK: char = '⏎';
const REFRESH: &str = "uv run --script scripts/screenplay-tools-diff.py";
const SNAPSHOT: &str = "tests/corpus/screenplay-tools.json";
const RECORD: &str = "docs/conformance/screenplay-tools.md";

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn load_snapshot() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SNAPSHOT);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{SNAPSHOT}: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{SNAPSHOT}: {e}"))
}

/// Ok when the snapshot holds exactly the corpus files, each with the hash of its bytes.
fn check_fresh(snapshot: &Value, files: &[(String, String)]) -> Result<(), String> {
    let held = &snapshot["files"];
    for (name, src) in files {
        if held[name.as_str()]["sha256"].as_str() != Some(&sha256_hex(src.as_bytes())) {
            return Err(format!("{SNAPSHOT} is stale for {name}: run `{REFRESH}`"));
        }
    }
    let count = held.as_object().map_or(0, |o| o.len());
    if count != files.len() {
        return Err(format!(
            "{SNAPSHOT} has {count} files, the corpus {}: run `{REFRESH}`",
            files.len()
        ));
    }
    Ok(())
}

// ---- the line form ----

/// The text with emphasis and escapes written back as markers, and Notes and Boneyards left out.
fn marked(runs: &[Run], out: &mut String) {
    for run in runs {
        match run {
            Run::Plain(t) => out.push_str(t),
            Run::Escaped(c) => {
                out.push('\\');
                out.push(*c);
            }
            Run::Emphasis {
                bold,
                italic,
                underline,
                runs,
            } => {
                let stars = match (bold, italic) {
                    (true, true) => "***",
                    (true, false) => "**",
                    (false, true) => "*",
                    (false, false) => "",
                };
                let under = if *underline { "_" } else { "" };
                out.push_str(under);
                out.push_str(stars);
                marked(runs, out);
                out.push_str(stars);
                out.push_str(under);
            }
            Run::Note(_) | Run::Boneyard(_) => {}
        }
    }
}

/// The marked text, every line break written `\n`.
fn raw(text: &Text) -> String {
    let mut out = String::new();
    marked(&text.runs, &mut out);
    out.replace("\r\n", "\n").replace('\r', "\n")
}

/// The text on one line: only its ends are stripped, and each line break is the break mark.
fn shown(text: &Text) -> String {
    flat(&raw(text))
}

fn flat(s: &str) -> String {
    s.trim().replace('\n', &BREAK.to_string())
}

/// An Action's marked text, not stripped, with each tab four spaces (`Action::display`, OFF-18c).
fn action_raw(text: &Text) -> String {
    raw(text).replace('\t', "    ")
}

/// The importer's `HASH_V1` of a scene: the first 12 hex digits of the SHA-256 of its scheme 1 text.
fn hash_line(scene: &Scene) -> String {
    let hash = sha256_hex(scene.normalized_text_v1().as_bytes());
    format!("HASH_V1 {}", &hash[..12])
}

#[derive(Default)]
struct Out {
    lines: Vec<String>,
    /// Consecutive non-centered Actions, merged into one line when something else comes.
    actions: Vec<String>,
}

impl Out {
    /// A line `<prefix> <text>`, dropped when the text is empty.
    fn push(&mut self, prefix: &str, text: &Text) {
        self.push_shown(prefix, shown(text));
    }

    fn push_shown(&mut self, prefix: &str, t: String) {
        if !t.is_empty() {
            self.lines.push(format!("{prefix} {t}"));
        }
    }

    fn flush(&mut self) {
        // Like screenplay-tools, strip the ends of the merged text once, not of each paragraph.
        let merged = flat(&self.actions.join("\n\n"));
        self.actions.clear();
        self.push_shown("ACTION", merged);
    }

    fn element(&mut self, el: &Element) {
        match el {
            // A whole-line Note or Boneyard is left out and does not end a run of Actions.
            Element::Note(_) | Element::Boneyard(_) => {}
            Element::Action(a) if !a.centered => {
                // An empty Action stays in the run; the line is dropped only if the whole run is empty.
                self.actions.push(action_raw(&a.text));
                a.interjections.iter().for_each(|i| self.element(i));
            }
            other => {
                self.flush();
                match other {
                    Element::Action(a) => {
                        self.push_shown("CENTERED", flat(&action_raw(&a.text)));
                        a.interjections.iter().for_each(|i| self.element(i));
                    }
                    Element::Dialogue(b) => self.block(b, false),
                    Element::DualDialogue(d) => {
                        self.block(&d.left, false);
                        self.block(&d.right, true);
                    }
                    Element::Transition(t) => self.push("TRANSITION", &t.text),
                    Element::Lyrics(l) => l.lines.iter().for_each(|t| self.push("LYRIC", t)),
                    Element::Section(x) => self.push(&format!("SECTION {}", x.depth), &x.text),
                    Element::Synopsis(x) => self.push("SYNOPSIS", &x.text),
                    Element::PageBreak(_) => self.lines.push("PAGEBREAK".to_owned()),
                    _ => panic!("the flattener has no arm for this Element variant: {other:?}"),
                }
            }
        }
    }

    fn block(&mut self, b: &DialogueBlock, right: bool) {
        let mut cue = shown(&b.cue.name);
        for ext in b.cue.extensions.iter().filter(|e| !e.is_contd()) {
            cue.push_str(&format!(" ({})", shown(&ext.text)));
        }
        if right {
            cue.push_str(" ^");
        }
        self.lines.push(format!("CHARACTER {cue}"));
        let mut run = Vec::new();
        for item in &b.items {
            match item {
                DialogueItem::Line(t) => run.push(raw(t)),
                DialogueItem::Parenthetical(t) => {
                    self.dialogue(&mut run);
                    // The library keeps the parentheses; screenplay-tools gives the text inside.
                    let p = shown(t);
                    let inner = p.strip_prefix('(').and_then(|x| x.strip_suffix(')'));
                    self.push_shown("PAREN", inner.unwrap_or(&p).trim().to_owned());
                }
                DialogueItem::Interjection(e) => {
                    if !matches!(**e, Element::Note(_) | Element::Boneyard(_)) {
                        self.dialogue(&mut run);
                    }
                    self.element(e);
                }
            }
        }
        self.dialogue(&mut run);
    }

    /// The Dialogue lines gathered so far, as one line.
    fn dialogue(&mut self, run: &mut Vec<String>) {
        // An empty Line is a continuation line (OFF-1b) and stays; only the ends are stripped.
        self.push_shown("DIALOGUE", flat(&run.join("\n")));
        run.clear();
    }

    fn scene(&mut self, sc: &Scene) {
        let at = sc.heading.span.map_or(0, |s| s.start);
        let start = |t: &Text| t.span.map_or(0, |s| s.start);
        let mut attached: Vec<(usize, String, &Text)> = sc
            .sections
            .iter()
            .map(|x| (start(&x.text), format!("SECTION {}", x.depth), &x.text))
            .chain(
                sc.synopses
                    .iter()
                    .map(|x| (start(&x.text), "SYNOPSIS".to_owned(), &x.text)),
            )
            .collect();
        attached.sort_by_key(|a| a.0);
        let (before, after): (Vec<_>, Vec<_>) = attached.into_iter().partition(|a| a.0 < at);
        before.iter().for_each(|(_, p, t)| self.push(p, t));
        let number = sc
            .number
            .as_ref()
            .map_or(String::new(), |n| format!(" #{}", n.text));
        self.lines
            .push(format!("HEADING {}{number}", shown(&sc.heading.text)));
        after.iter().for_each(|(_, p, t)| self.push(p, t));
        sc.elements.iter().for_each(|e| self.element(e));
        self.flush();
        self.lines.push(hash_line(sc));
    }
}

/// The typed view in the line form of `scripts/screenplay-tools-diff.py`.
fn lines(sp: &Screenplay) -> Vec<String> {
    let mut out = Out::default();
    let sep = BREAK.to_string();
    for entry in sp.title_page.iter().flat_map(|t| &t.entries) {
        // Only a TITLE value is stripped line by line, with the empty lines dropped.
        let value: Vec<String> = entry
            .values
            .iter()
            .flat_map(|v| {
                raw(v)
                    .trim()
                    .split('\n')
                    .map(|l| l.trim().to_owned())
                    .collect::<Vec<_>>()
            })
            .filter(|l| !l.is_empty())
            .collect();
        out.lines
            .push(format!("TITLE {}: {}", shown(&entry.key), value.join(&sep)));
    }
    sp.preamble.elements.iter().for_each(|e| out.element(e));
    out.flush();
    sp.scenes.iter().for_each(|sc| out.scene(sc));
    out.lines
}

// ---- the diff and the record ----

/// One run of changed lines: ours (`-`) against theirs (`+`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Hunk {
    ours: Vec<String>,
    theirs: Vec<String>,
}

/// A longest-common-subsequence line diff; adjacent changes form one hunk.
fn diff(ours: &[String], theirs: &[String]) -> Vec<Hunk> {
    let (n, m) = (ours.len(), theirs.len());
    // lcs[i][j] is the common length of ours[i..] and theirs[j..].
    let mut lcs = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if ours[i] == theirs[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }
    let (mut hunks, mut cur) = (Vec::new(), Hunk::default());
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && ours[i] == theirs[j] {
            if cur != Hunk::default() {
                hunks.push(std::mem::take(&mut cur));
            }
            (i, j) = (i + 1, j + 1);
        } else if j == m || (i < n && lcs[i + 1][j] >= lcs[i][j + 1]) {
            cur.ours.push(ours[i].clone());
            i += 1;
        } else {
            cur.theirs.push(theirs[j].clone());
            j += 1;
        }
    }
    if cur != Hunk::default() {
        hunks.push(cur);
    }
    hunks
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Class {
    TheirsWrong,
    OursWrong,
    DocumentedChoice,
    Todo,
}

impl Class {
    fn parse(name: &str) -> Class {
        match name {
            "theirs-wrong" => Class::TheirsWrong,
            "ours-wrong" => Class::OursWrong,
            "documented-choice" => Class::DocumentedChoice,
            "todo" => Class::Todo,
            _ => panic!("unknown class {name:?} in {RECORD}"),
        }
    }
}

#[derive(Debug)]
struct Entry {
    file: String,
    class: Class,
    hunk: Hunk,
}

/// The entry text for the record, class `todo`, ready to paste.
fn render(file: &str, h: &Hunk) -> String {
    let mut out = format!(
        "### {file}\n\n**Class:** todo · **Ruling:** ? · **Reference parser:** ?\n\nUnclassified.\n\n````diff\n"
    );
    for l in &h.ours {
        out.push_str(&format!("-{l}\n"));
    }
    for l in &h.theirs {
        out.push_str(&format!("+{l}\n"));
    }
    out.push_str("````\n\n");
    out
}

/// The entries of the record: a `### <file>` line, a `**Class:**` line, then a `diff` block.
fn parse_record(md: &str) -> Vec<Entry> {
    let (mut entries, mut file, mut class) = (Vec::new(), None, None);
    let mut rows = md.lines();
    while let Some(row) = rows.next() {
        if let Some(f) = row.strip_prefix("### ") {
            (file, class) = (Some(f.trim().to_owned()), None);
        } else if let Some(rest) = row.strip_prefix("**Class:** ") {
            class = Some(Class::parse(rest.split(" · ").next().unwrap_or_default()));
        } else if row == "````diff" {
            let (Some(file), Some(class)) = (file.clone(), class.take()) else {
                panic!("a diff block without a heading and a class in {RECORD}");
            };
            let mut hunk = Hunk::default();
            for row in rows.by_ref().take_while(|r| *r != "````") {
                match (row.strip_prefix('-'), row.strip_prefix('+')) {
                    (Some(l), _) => hunk.ours.push(l.to_owned()),
                    (_, Some(l)) => hunk.theirs.push(l.to_owned()),
                    _ => panic!("bad diff line {row:?} in {RECORD}"),
                }
            }
            entries.push(Entry { file, class, hunk });
        }
    }
    entries
}

fn s(items: &[&str]) -> Vec<String> {
    items.iter().map(|x| (*x).to_owned()).collect()
}

#[test]
fn diff_groups_adjacent_changes() {
    let hunks = diff(&s(&["A", "B", "C"]), &s(&["A", "X", "C"]));
    assert_eq!(hunks.len(), 1);
    assert_eq!((&hunks[0].ours, &hunks[0].theirs), (&s(&["B"]), &s(&["X"])));
}

#[test]
fn diff_of_equal_lists_is_empty() {
    assert!(diff(&s(&["A", "B"]), &s(&["A", "B"])).is_empty());
}

#[test]
fn record_round_trips_a_hunk_with_backticks() {
    let hunk = Hunk {
        ours: s(&["ACTION a ``` b | c", "HEADING X"]),
        theirs: s(&["ACTION a b"]),
    };
    let md = format!(
        "# Record\n\nprose\n\n{}",
        render("off/off-1a.fountain", &hunk)
    );
    let entries = parse_record(&md);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file, "off/off-1a.fountain");
    assert_eq!(entries[0].class, Class::Todo);
    assert_eq!(entries[0].hunk, hunk);
}

#[test]
#[should_panic(expected = "unknown class")]
fn record_rejects_an_unknown_class() {
    parse_record(
        "### off/x.fountain\n\n**Class:** maybe · **Ruling:** x · **Reference parser:** y\n\nWhy.\n\n````diff\n-A\n+B\n````\n",
    );
}

#[test]
fn stale_snapshot_names_the_command() {
    let files = vec![("off/a.fountain".to_owned(), "x\n".to_owned())];
    let command = "uv run --script scripts/screenplay-tools-diff.py";
    let wrong = json!({"files": {"off/a.fountain": {"sha256": "00", "lines": []}}});
    assert!(check_fresh(&wrong, &files).unwrap_err().contains(command));
    let missing = json!({"files": {}});
    assert!(check_fresh(&missing, &files).unwrap_err().contains(command));
    let right = json!({"files": {"off/a.fountain": {"sha256": sha256_hex(b"x\n"), "lines": []}}});
    assert_eq!(check_fresh(&right, &files), Ok(()));
}

#[test]
fn lines_flatten_a_small_script() {
    let src = "Title: Demo\n\nINT. ROOM - DAY\n\nThe kettle is **hot** and *old*, \\*not\\* _bare_.\n\n\
               [[a note]]\n\nStill hot.\n\nMARA (V.O.) (CONT'D)\n(quietly)\nHello.\nThere.\n\n\
               BEN\nHi.\n\nNORA ^\nYo.\n";
    let got = lines(&parse(src).screenplay());
    let (hash, body) = got.split_last().expect("lines");
    assert!(
        hash.starts_with("HASH_V1 ") && hash.len() == "HASH_V1 ".len() + 12,
        "{hash}"
    );
    assert_eq!(
        body,
        s(&[
            "TITLE Title: Demo",
            "HEADING INT. ROOM - DAY",
            "ACTION The kettle is **hot** and *old*, \\*not\\* _bare_.⏎⏎Still hot.",
            "CHARACTER MARA (V.O.)",
            "PAREN quietly",
            "DIALOGUE Hello.⏎There.",
            "CHARACTER BEN",
            "DIALOGUE Hi.",
            "CHARACTER NORA ^",
            "DIALOGUE Yo.",
        ])
    );
}

#[test]
fn lines_keep_indentation_and_expand_action_tabs() {
    let src = "INT. ROOM - DAY\n\n  He stands.\n\tShe  sits.\n\n   Later.\n\n\tTabbed.\n";
    let got = lines(&parse(src).screenplay());
    assert_eq!(
        got[1],
        "ACTION He stands.⏎    She  sits.⏎⏎   Later.⏎⏎    Tabbed."
    );
}

#[test]
fn lines_keep_an_empty_forced_action_in_a_run() {
    let got = lines(&parse("INT. ROOM - DAY\n\nHe waits.\n\n!\n\nShe goes.\n").screenplay());
    assert_eq!(got[1], "ACTION He waits.⏎⏎⏎⏎She goes.");
}

#[test]
fn lines_keep_a_two_space_continuation_line_in_dialogue() {
    let got = lines(&parse("INT. ROOM - DAY\n\nMARA\nWait.\n  \nGo on.\n").screenplay());
    assert_eq!(got[2], "DIALOGUE Wait.⏎⏎Go on.");
}

#[test]
fn snapshot_covers_the_line_ending_rows() {
    let snapshot = load_snapshot();
    for f in [
        "off/off-2a.fountain",
        "off/off-2b.fountain",
        "off/off-23e.fountain",
    ] {
        assert!(snapshot["files"][f]["lines"].is_array(), "{f}");
    }
}

#[test]
fn screenplay_tools_every_difference_is_classified() {
    let files = common::corpus_files();
    let snapshot = load_snapshot();
    if let Err(e) = check_fresh(&snapshot, &files) {
        panic!("{e}");
    }
    let record = Path::new(env!("CARGO_MANIFEST_DIR")).join(RECORD);
    let md = std::fs::read_to_string(&record).unwrap_or_else(|e| panic!("{RECORD}: {e}"));
    let entries = parse_record(&md);
    let mut used = vec![false; entries.len()];
    let (mut unrecorded, mut hash_hunks, mut hunks) = (Vec::new(), Vec::new(), 0);
    for (name, src) in &files {
        let theirs: Vec<String> = snapshot["files"][name.as_str()]["lines"]
            .as_array()
            .unwrap_or_else(|| panic!("{name}: no lines in {SNAPSHOT}"))
            .iter()
            .map(|v| {
                v.as_str()
                    .unwrap_or_else(|| panic!("{name}: {v}"))
                    .to_owned()
            })
            .collect();
        for h in diff(&lines(&parse(src).screenplay()), &theirs) {
            hunks += 1;
            let is_hash = |l: &String| l.starts_with("HASH_V1 ");
            if name.starts_with("twopop/") && h.ours.iter().chain(&h.theirs).any(is_hash) {
                hash_hunks.push(format!("{name}: {h:?}"));
            }
            let found = (0..entries.len())
                .find(|&k| !used[k] && entries[k].file == *name && entries[k].hunk == h);
            match found {
                Some(k) => used[k] = true,
                None => unrecorded.push(render(name, &h)),
            }
        }
    }
    let stale: Vec<String> = (0..entries.len())
        .filter(|&k| !used[k])
        .map(|k| format!("{} {:?}", entries[k].file, entries[k].hunk))
        .collect();
    let open: Vec<String> = entries
        .iter()
        .filter(|e| matches!(e.class, Class::Todo | Class::OursWrong))
        .map(|e| format!("{} [{:?}] {:?}", e.file, e.class, e.hunk))
        .collect();
    let mut problems = String::new();
    for (title, items) in [
        (
            "hunks of twopop/ files with a HASH_V1 line (HASH1-22)",
            &hash_hunks,
        ),
        ("stale entries, no hunk matches", &stale),
        ("entries still todo or ours-wrong", &open),
    ] {
        if !items.is_empty() {
            problems.push_str(&format!(
                "\n== {title}: {}\n{}\n",
                items.len(),
                items.join("\n")
            ));
        }
    }
    if !unrecorded.is_empty() {
        problems.push_str(&format!(
            "\n== unrecorded hunks: {}\n{}",
            unrecorded.len(),
            unrecorded.concat()
        ));
    }
    assert!(
        problems.is_empty(),
        "{hunks} hunks in {} files; classify them in {RECORD}:\n{problems}",
        files.len()
    );
}
