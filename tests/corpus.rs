//! The conformance corpus (`SPEC.md` §12): every file has a recorded source and licence, each `off/`
//! script keeps its first block ordinary, and each script fires the diagnostic its `OFF` row names.

mod common;

use std::collections::BTreeSet;

use common::corpus_files;
use fountain_syntax::{Element, parse};

/// The `off/` scripts whose case is the first block, so they cannot start with an ordinary one.
const START_OF_DOCUMENT: [&str; 11] = [
    "off/off-6e.fountain",
    "off/off-15f.fountain",
    "off/off-17a.fountain",
    "off/off-17a-heading.fountain",
    "off/off-17a-bare-key.fountain",
    "off/off-17a-prose.fountain",
    "off/off-17b.fountain",
    "off/off-17c.fountain",
    "off/off-17d.fountain",
    "off/off-17e.fountain",
    "off/off-17f.fountain",
];

const OFF_GLOB: &str = "off/off-*.fountain";

fn read(name: &str) -> String {
    std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(name))
        .unwrap_or_else(|e| panic!("read {name}: {e}"))
}

#[test]
fn corpus_every_file_has_source_and_licence() {
    let readme = read("tests/corpus/README.md");
    let listed: Vec<&str> = readme
        .lines()
        .filter(|l| l.starts_with('|'))
        .filter_map(|l| l.split('|').nth(1))
        .map(|c| c.trim().trim_matches('`'))
        .collect();
    for (path, _) in corpus_files() {
        let by_glob = path.starts_with("off/off-") && path.ends_with(".fountain");
        assert!(
            listed.contains(&path.as_str()) || (by_glob && listed.contains(&OFF_GLOB)),
            "{path} has no source and licence row in tests/corpus/README.md"
        );
    }
}

/// The lines of the first block: after any leading blank lines (and a BOM), up to the next blank line.
fn first_block(src: &str) -> Vec<String> {
    // CRLF, CR and LF each count as one line break.
    let src = src
        .trim_start_matches('\u{FEFF}')
        .replace("\r\n", "\n")
        .replace('\r', "\n");
    src.lines()
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn corpus_off_scripts_keep_their_first_block_ordinary() {
    let scripts = corpus_files()
        .into_iter()
        .filter(|(p, _)| p.starts_with("off/") && !START_OF_DOCUMENT.contains(&p.as_str()));
    for (path, src) in scripts {
        let sp = parse(&src).screenplay();
        assert!(
            sp.title_page.is_none(),
            "{path}: first block is a title page"
        );
        let first = sp
            .preamble
            .elements
            .iter()
            .find(|e| !matches!(e, Element::Note(_) | Element::Boneyard(_)));
        assert!(
            matches!(first, Some(Element::Action(_))),
            "{path}: first visible element is {first:?}, not Action"
        );
        let block = first_block(&src);
        // A sigil, or a note or boneyard opener, would hide or change the first block.
        let lead = block.first().and_then(|l| l.trim_start().chars().next());
        assert!(
            lead.is_some_and(|c| !"!@~.>#=[/".contains(c)),
            "{path}: first block starts with a sigil or an opener: {block:?}"
        );
        assert!(
            block.iter().all(|l| !l.contains(':')),
            "{path}: first block has a colon: {block:?}"
        );
    }
}

/// The lowercase row ID of a §6.2 row line (`| OFF-<digits><letter> | ...`). The §6.3 lines read
/// `| OFF-7: ...` and give none.
fn row_id(line: &str) -> Option<String> {
    let id = line.strip_prefix("| ")?.split(' ').next()?;
    let rest = id.strip_prefix("OFF-")?;
    let digits = rest.strip_suffix(|c: char| c.is_ascii_lowercase())?;
    let ok = !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit());
    ok.then(|| id.to_lowercase())
}

/// (lowercase row ID, diagnostic code) for each §6.2 row whose Diagnostic cell names a code.
fn rows_with_diagnostics(spec: &str) -> Vec<(String, String)> {
    let rows: Vec<(String, String)> = spec
        .lines()
        .filter_map(|l| {
            let id = row_id(l)?;
            let cell = l
                .trim_end()
                .trim_end_matches('|')
                .rsplit('|')
                .next()?
                .trim();
            if cell == "none" {
                return None;
            }
            let code = cell
                .strip_prefix('`')
                .and_then(|c| c.strip_suffix('`'))
                .filter(|c| !c.is_empty() && !c.contains('`'))
                .unwrap_or_else(|| {
                    panic!("{id}: Diagnostic cell is not `none` or one backticked code: {cell:?}")
                });
            Some((id, code.to_owned()))
        })
        .collect();
    assert!(
        !rows.is_empty(),
        "no OFF rows with a diagnostic found in SPEC.md"
    );
    rows
}

#[test]
fn corpus_off_scripts_fire_their_row_diagnostic() {
    let files = corpus_files();
    for (id, code) in rows_with_diagnostics(&read("SPEC.md")) {
        let name = format!("off/{id}.fountain");
        // A missing script is reported by `corpus_every_off_row_has_a_script`.
        let Some((_, src)) = files.iter().find(|(p, _)| *p == name) else {
            continue;
        };
        let doc = parse(src);
        let codes: Vec<&str> = doc.diagnostics().iter().map(|d| d.code.as_str()).collect();
        assert!(
            codes.contains(&code.as_str()),
            "{name}: expected diagnostic {code}, got {codes:?}"
        );
    }
}

#[test]
fn corpus_off_23e_uses_one_line_ending() {
    // OFF-23e is about a carriage return as trailing whitespace; mixed endings are `off-2a`'s case.
    let doc = parse(&read("tests/corpus/off/off-23e.fountain"));
    let codes: Vec<&str> = doc.diagnostics().iter().map(|d| d.code.as_str()).collect();
    assert!(
        !codes.contains(&"mixed-line-endings"),
        "off-23e mixes line endings: {codes:?}"
    );
}

#[test]
fn corpus_every_off_row_has_a_script() {
    let rows: BTreeSet<String> = read("SPEC.md").lines().filter_map(row_id).collect();
    assert!(!rows.is_empty(), "no OFF rows found in SPEC.md");
    let scripts: BTreeSet<String> = corpus_files()
        .into_iter()
        .filter_map(|(p, _)| {
            let stem = p.strip_prefix("off/")?.strip_suffix(".fountain")?;
            // `off-<row>-<case>` is an extra script of row `off-<row>`.
            Some(stem.splitn(3, '-').take(2).collect::<Vec<_>>().join("-"))
        })
        .collect();
    let missing: Vec<_> = rows.difference(&scripts).collect();
    let extra: Vec<_> = scripts.difference(&rows).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "SPEC.md §6.2 rows without a script: {missing:?}; scripts without a row: {extra:?}"
    );
}
