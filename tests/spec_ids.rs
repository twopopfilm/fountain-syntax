//! Guards the rule that every ID in `SPEC.md` §12.1 has a test that names it: each ID must appear in some
//! `tests/*.rs` file, as a test name or in a comment, with `-` or `_` at each hyphen.

use std::path::Path;

/// IDs still without a test, kept only until their tests land (issue #79).
const ALLOWLIST: [&str; 7] = [
    "EL-action-4",
    "EL-emphasis-1",
    "EL-emphasis-2",
    "EXT-contd-1",
    "EXT-open-keys-2",
    "HASH2-31",
    "OFF-14b",
];

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
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

    let mut tests = String::new();
    for entry in std::fs::read_dir(root.join("tests")).expect("read tests/") {
        let path = entry.expect("read tests/ entry").path();
        let is_guard = path.file_name().is_some_and(|n| n == "spec_ids.rs");
        if path.extension().is_some_and(|e| e == "rs") && !is_guard {
            tests.push_str(&read(&path).to_lowercase());
            tests.push('\n');
        }
    }

    let missing: Vec<&str> = ids
        .iter()
        .copied()
        .filter(|id| !ALLOWLIST.contains(id) && !names(&tests, &id.to_lowercase()))
        .collect();
    assert!(
        missing.is_empty(),
        "{} SPEC.md ID(s) are named by no tests/*.rs file: {}",
        missing.len(),
        missing.join(", ")
    );
}

#[test]
fn names_respects_boundaries_and_underscores() {
    assert!(names("fn el_action_1() {}", "el-action-1"));
    assert!(names("// off-1a:", "off-1a"));
    assert!(!names("fn hash2_31()", "hash2-3"));
    assert!(!names("fn off_11a()", "off-1a"));
    assert!(!names("xel-action-1", "el-action-1"));
}
