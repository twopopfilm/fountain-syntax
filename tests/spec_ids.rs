//! Guards the rule that every ID in `SPEC.md` §12.1 has a test that names it: each ID must appear in some
//! `tests/*.rs` file, in a function name or a `///` / `//!` line (not a plain comment or a string), with `-` or
//! `_` at each hyphen.

use std::path::Path;

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

/// The lines of one test file that can name an ID: doc lines (`///`, `//!`) and `fn` lines, lower-cased and
/// joined by `\n`. A `//` comment and a string in a table are dropped, and so is a trailing comment on any line that
/// does not start with `fn ` / `pub fn `; such a line is kept whole. It does not tell a test from a helper: any `fn`
/// line or doc line counts.
// shortcut: a `fn` line is kept whole, so a trailing comment on it counts; upgrade if one ever hides an ID.
fn naming_lines(source: &str) -> String {
    let kept: Vec<&str> = source
        .lines()
        .filter(|line| {
            let line = line.trim_start();
            ["///", "//!", "fn ", "pub fn "]
                .iter()
                .any(|marker| line.starts_with(marker))
        })
        .collect();
    kept.join("\n").to_lowercase()
}

/// The IDs listed in the ID-INDEX block of `spec`, checked against its Total line.
fn spec_ids(spec: &str) -> Vec<&str> {
    let index = spec
        .split("<!-- ID-INDEX:start -->")
        .nth(1)
        .and_then(|rest| rest.split("<!-- ID-INDEX:end -->").next())
        .expect("SPEC.md has no ID-INDEX block");
    let ids: Vec<&str> = index.split('`').skip(1).step_by(2).collect();

    let total: usize = index
        .split("**Total: ")
        .nth(1)
        .and_then(|rest| rest.split(" IDs.**").next())
        .and_then(|n| n.parse().ok())
        .expect("the ID-INDEX block has no `**Total: N IDs.**` line");
    assert!(!ids.is_empty(), "the ID-INDEX block lists no IDs");
    assert_eq!(
        ids.len(),
        total,
        "the ID-INDEX block lists a different number of IDs than its Total line"
    );
    ids
}

/// The first `(shorter, longer)` pair where the lower-cased `longer` is `shorter` followed by `-`.
fn shared_prefix_pair<'a>(ids: &[&'a str]) -> Option<(&'a str, &'a str)> {
    ids.iter().find_map(|&short| {
        let prefix = format!("{}-", short.to_lowercase());
        ids.iter()
            .find(|long| long.to_lowercase().starts_with(&prefix))
            .map(|&long| (short, long))
    })
}

/// True when `id` (lower case) occurs in `text` (lower case), with `-` or `_` at each hyphen and no ASCII
/// alphanumeric on either side, so `hash2-3` does not match `hash2_31`.
fn names(text: &str, id: &str) -> bool {
    let (text, id) = (text.as_bytes(), id.as_bytes());
    (0..text.len()).any(|start| {
        let Some(end) = start.checked_add(id.len()).filter(|&e| e <= text.len()) else {
            return false;
        };
        let fits = text[start..end]
            .iter()
            .zip(id)
            .all(|(&t, &i)| t == i || (i == b'-' && t == b'_'));
        let before = start == 0 || !text[start - 1].is_ascii_alphanumeric();
        let after = end == text.len() || !text[end].is_ascii_alphanumeric();
        fits && before && after
    })
}

#[test]
fn every_spec_id_is_named_by_a_test() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let spec = read(&root.join("SPEC.md"));
    let ids = spec_ids(&spec);
    assert_eq!(
        shared_prefix_pair(&ids),
        None,
        "an ID is a hyphen-prefix of another, and `names` treats `-` and `_` as boundaries, so the longer would \
         count for the shorter"
    );

    let mut tests = String::new();
    for entry in std::fs::read_dir(root.join("tests")).expect("read tests/") {
        let path = entry.expect("read tests/ entry").path();
        let is_guard = path.file_name().is_some_and(|n| n == "spec_ids.rs");
        if path.extension().is_some_and(|e| e == "rs") && !is_guard {
            tests.push_str(&naming_lines(&read(&path)));
            tests.push('\n');
        }
    }

    let missing: Vec<&str> = ids
        .iter()
        .copied()
        .filter(|id| !names(&tests, &id.to_lowercase()))
        .collect();
    assert!(
        missing.is_empty(),
        "{} SPEC.md ID(s) are named by no function name or `///` / `//!` line in tests/*.rs: {}",
        missing.len(),
        missing.join(", ")
    );
}

#[test]
fn names_respects_boundaries_and_underscores() {
    assert!(names("fn el_action_1() {}", "el-action-1"));
    assert!(names("/// off-1a: x", "off-1a"));
    assert!(!names("fn hash2_31()", "hash2-3"));
    assert!(!names("fn off_11a()", "off-1a"));
    assert!(!names("xel-action-1", "el-action-1"));
    assert!(names("off-1a", "off-1a"));
    assert!(names("off-1a x", "off-1a"));
    assert!(names("x off-1a", "off-1a"));
    assert!(!names("off-1ab", "off-1a"));
    assert!(!names("off-1a2", "off-1a"));
    assert!(!names("aoff-1a", "off-1a"));
    assert!(!names("fn off_1ab()", "off-1a"));
}

#[test]
fn naming_lines_keeps_doc_and_fn_lines_only() {
    let kept = naming_lines("/// Off-1A\n//! OFF-2b\n    fn Off_3c() {}\npub fn OFF_4d() {}");
    assert_eq!(
        kept,
        "/// off-1a\n//! off-2b\n    fn off_3c() {}\npub fn off_4d() {}"
    );

    let dropped =
        "// off-1a plain comment\n    (\"OFF-2b\", \"x\", &[]),\nlet x = 1; // see /// off-3c\n";
    assert_eq!(naming_lines(dropped), "");
}

#[test]
fn spec_ids_returns_the_listed_ids() {
    let spec = "intro\n<!-- ID-INDEX:start -->\n`A-1`, `b-2`\n**Total: 2 IDs.**\n<!-- ID-INDEX:end -->\n`x`";
    assert_eq!(spec_ids(spec), ["A-1", "b-2"]);
}

#[test]
#[should_panic(expected = "no ID-INDEX block")]
fn spec_ids_needs_the_markers() {
    spec_ids("`a-1`\n**Total: 1 IDs.**");
}

#[test]
#[should_panic(expected = "no `**Total: N IDs.**` line")]
fn spec_ids_needs_a_total_line() {
    spec_ids("<!-- ID-INDEX:start -->\n`a-1`\n<!-- ID-INDEX:end -->");
}

#[test]
#[should_panic(expected = "lists no IDs")]
fn spec_ids_needs_some_ids() {
    spec_ids("<!-- ID-INDEX:start -->\n**Total: 0 IDs.**\n<!-- ID-INDEX:end -->");
}

#[test]
#[should_panic(expected = "different number of IDs")]
fn spec_ids_total_must_match() {
    spec_ids("<!-- ID-INDEX:start -->\n`a-1`\n**Total: 2 IDs.**\n<!-- ID-INDEX:end -->");
}

#[test]
fn shared_prefix_pair_finds_hyphen_prefixes_only() {
    assert_eq!(
        shared_prefix_pair(&["a-1", "a-1-2"]),
        Some(("a-1", "a-1-2"))
    );
    assert_eq!(shared_prefix_pair(&["a-1", "a-12", "a-2"]), None);
}
