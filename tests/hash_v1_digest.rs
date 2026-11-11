#![cfg(feature = "hash")]
//! The scheme 1 digest (`SPEC.md` 11.7), its scheme tag, and the migration an application runs
//! (11.8): `HASH1-20`, the digest half of `HASH1-26`, the `_v1` clause of `HASH2-39` and
//! `HASH2-46`. The line rules are tested in `hash_v1.rs`.

use fountain_syntax::{Digest, Element, HashScheme, Scene, parse};
use sha2::{Digest as _, Sha256};

fn scenes(src: &str) -> Vec<Scene> {
    parse(src).screenplay().scenes
}

fn scene(src: &str) -> Scene {
    scenes(src).remove(0)
}

/// SHA-256 of `s` as lowercase hex, computed here and not by the library.
fn sha_hex(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}

#[test]
fn hash1_20_known_answer() {
    let want = "f7eb426cff87a0a1af2b2d5390cc9a2477df78d309f88f77f176751c57863e6f";
    let sc = scene(".HEADING ONLY\n");
    assert_eq!(sc.normalized_text_v1(), "HEADING ONLY\n");
    assert_eq!(sc.normalized_text_v1().len(), 13);
    assert_eq!(sc.hash_v1().to_string(), want);
    assert_eq!(sha_hex("HEADING ONLY\n"), want);
}

#[test]
fn hash1_26_digest_scheme_text_form_and_stability() {
    let sc = scene("INT. A - DAY\n\nMara waits.\n\nMARA\nHi.\n");
    let d = sc.hash_v1();
    assert_eq!(d.scheme, HashScheme::V1);
    let text = d.to_string();
    assert_eq!(text.len(), 64);
    assert!(
        text.chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
    );
    assert!(!text.contains(':'), "no prefix: {text}");
    assert_eq!(text, sha_hex(&sc.normalized_text_v1()));
    assert_eq!(text.parse::<Digest>(), Ok(d));
    assert_eq!(sc.hash_v1(), d);
    assert_eq!(
        scene("INT. A - DAY\n\nMara waits.\n\nMARA\nHi.\n").hash_v1(),
        d
    );
}

#[test]
fn hash2_39_hash_v1_gives_v1_and_never_equals_v2() {
    let sc = scene("INT. A - DAY\n\nMara waits.\n");
    let v1 = sc.hash_v1();
    let v2 = sc.hash();
    assert_eq!(v1.scheme, HashScheme::V1);
    assert_eq!(v2.scheme, HashScheme::V2);
    let relabelled = Digest {
        scheme: HashScheme::V2,
        bytes: v1.bytes,
    };
    assert_ne!(relabelled, v1);
    assert_ne!(v1, v2);
    assert!(v2.to_string().starts_with("fs2:"));
}

// A small test application for the migration of 11.8.

/// Compares only digests of one scheme; `None` for a mix (rule 3).
fn same_scheme_eq(a: &Digest, b: &Digest) -> Option<bool> {
    (a.scheme == b.scheme).then(|| a.bytes == b.bytes)
}

/// The current digest of `scene` under `scheme`.
fn digest_in(scene: &Scene, scheme: HashScheme) -> Digest {
    match scheme {
        HashScheme::V1 => scene.hash_v1(),
        HashScheme::V2 => scene.hash(),
        _ => panic!("a scheme this test application does not know"),
    }
}

/// An input hash that embeds a scene hash, built from the digest's text form (rule 4).
fn input_hash(d: &Digest) -> String {
    sha_hex(&format!("breakdown:{d}"))
}

/// What the application stores for a scene: the digest in its text form (rule 1) and the input
/// hash of the dependent breakdown. The record is stale when the input hash no longer matches.
#[derive(Clone)]
struct Record {
    digest: String,
    input: String,
}

impl Record {
    fn fresh(d: &Digest) -> Self {
        Record {
            digest: d.to_string(),
            input: input_hash(d),
        }
    }

    fn stored(&self) -> Digest {
        self.digest.parse().expect("a stored digest in text form")
    }

    fn is_stale(&self) -> bool {
        self.input != input_hash(&self.stored())
    }
}

/// The rebaseline of 11.8 for one record, from the same parse as `scene`.
fn rebaseline(rec: &mut Record, scene: &Scene) {
    let old = rec.stored();
    if old.scheme != HashScheme::V1 {
        return;
    }
    let was_current = !rec.is_stale();
    let unchanged = same_scheme_eq(&old, &scene.hash_v1()).expect("both are scheme 1");
    let new = scene.hash();
    rec.digest = new.to_string();
    // Only a record whose input hash matched the old computation is re-based. An unchanged scene
    // marks nothing stale; a changed one keeps the old input hash, so its record is stale.
    if unchanged && was_current {
        rec.input = input_hash(&new);
    }
}

/// Re-import matching: a stored scheme 1 value is matched on `hash_v1`, a scheme 2 value on `hash`.
fn find_match<'a>(rec: &Record, scenes: &'a [Scene]) -> Option<&'a Scene> {
    let stored = rec.stored();
    scenes.iter().find(|s| {
        same_scheme_eq(&stored, &digest_in(s, stored.scheme))
            .expect("the same scheme by construction")
    })
}

const OLD: &str =
    "INT. A - DAY\n\nMara waits.\n\nINT. B - DAY\n\nOtto leaves.\n\nINT. C - DAY\n\nThey stare.\n";
// Scene B is edited, the others are not.
const NEW: &str =
    "INT. A - DAY\n\nMara waits.\n\nINT. B - DAY\n\nOtto stays.\n\nINT. C - DAY\n\nThey stare.\n";

#[test]
fn hash2_46_migration_rebaseline() {
    let old = scenes(OLD);
    let new = scenes(NEW);
    assert_eq!((old.len(), new.len()), (3, 3));

    // The application stores scheme 1 text forms; scene C's breakdown was already stale.
    let mut recs: Vec<Record> = old.iter().map(|s| Record::fresh(&s.hash_v1())).collect();
    recs[2].input = "outdated".into();
    assert!(recs.iter().all(|r| r.stored().scheme == HashScheme::V1));
    assert!(!recs[0].is_stale() && !recs[1].is_stale() && recs[2].is_stale());

    // Re-import matching while the values are scheme 1.
    let found = find_match(&recs[0], &new).expect("A matches on hash_v1");
    assert_eq!(found.heading_line_v1(), "INT. A - DAY");
    assert!(
        find_match(&recs[1], &new).is_none(),
        "the edited scene B does not match"
    );

    for (rec, s) in recs.iter_mut().zip(&new) {
        rebaseline(rec, s);
    }

    // Everything is stored as scheme 2 text now.
    for (rec, s) in recs.iter().zip(&new) {
        assert!(rec.digest.starts_with("fs2:"), "{}", rec.digest);
        assert_eq!(rec.stored(), s.hash());
    }
    // A is unchanged: nothing stale, and its input hash was re-based in the same step.
    assert!(!recs[0].is_stale());
    assert_eq!(recs[0].input, input_hash(&new[0].hash()));
    // B changed: its dependent record is stale.
    assert!(recs[1].is_stale());
    // C was stale before and stays stale (its input hash is not re-based).
    assert!(recs[2].is_stale());
    assert_eq!(recs[2].input, "outdated");

    // Re-import matching once the values are scheme 2.
    let found = find_match(&recs[0], &new).expect("A matches on hash");
    assert_eq!(found.heading_line_v1(), "INT. A - DAY");
    let found = find_match(&recs[1], &new).expect("the edited B matches its new hash");
    assert_eq!(found.heading_line_v1(), "INT. B - DAY");
    assert!(find_match(&recs[1], &old).is_none());

    // A second rebaseline is a no-op on scheme 2 records.
    let before: Vec<(String, String)> = recs
        .iter()
        .map(|r| (r.digest.clone(), r.input.clone()))
        .collect();
    for (rec, s) in recs.iter_mut().zip(&new) {
        rebaseline(rec, s);
    }
    let after: Vec<(String, String)> = recs
        .iter()
        .map(|r| (r.digest.clone(), r.input.clone()))
        .collect();
    assert_eq!(before, after);

    // Rule 3: the application refuses to compare across schemes.
    let (v1, v2) = (new[0].hash_v1(), new[0].hash());
    assert_eq!(same_scheme_eq(&v1, &v2), None);
    assert_eq!(same_scheme_eq(&v1, &v1), Some(true));
    assert_eq!(same_scheme_eq(&v2, &v2), Some(true));
    // Even bytes that are equal are not equal digests.
    let relabelled = Digest {
        scheme: HashScheme::V2,
        bytes: v1.bytes,
    };
    assert_eq!(same_scheme_eq(&v1, &relabelled), None);
    assert_ne!(v1, relabelled);
}

#[test]
fn hash2_46_blind_spot_kind_change_is_invisible_to_v1_and_seen_by_v2() {
    let before = scene("INT. A - DAY\n\nFADE OUT.\n");
    let after = scene("INT. A - DAY\n\n>FADE OUT.\n");
    // The same words, a different kind; the Action has no Action neighbour (it merges with none).
    assert!(matches!(before.elements.as_slice(), [Element::Action(_)]));
    assert!(matches!(
        after.elements.as_slice(),
        [Element::Transition(_)]
    ));

    let mut rec = Record::fresh(&before.hash_v1());
    assert_eq!(same_scheme_eq(&rec.stored(), &after.hash_v1()), Some(true));
    assert_eq!(
        before.hash_v1(),
        after.hash_v1(),
        "scheme 1 cannot see the change"
    );

    // The rebaseline marks nothing stale: the change slips through once.
    rebaseline(&mut rec, &after);
    assert!(!rec.is_stale());
    assert_eq!(rec.stored(), after.hash());

    // From then on scheme 2 sees the kind change.
    assert_ne!(before.hash(), after.hash());
    assert_eq!(
        same_scheme_eq(&Record::fresh(&before.hash()).stored(), &after.hash()),
        Some(false)
    );
}

const EDGE_CASES: &str = include_str!("corpus/twopop/edge_cases.fountain");
const TWO_SCENE: &str = include_str!("corpus/twopop/two_scene.fountain");
const BLUE_REV2: &str = include_str!("corpus/twopop/blue_rev2.fountain");

/// Each scene's heading and `hash_v1`, in order.
fn headings_and_hashes(src: &str) -> Vec<(String, String)> {
    scenes(src)
        .iter()
        .map(|s| (s.heading.text.plain(), s.hash_v1().to_string()))
        .collect()
}

fn expect(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
    pairs
        .iter()
        .map(|(h, d)| (h.to_string(), d.to_string()))
        .collect()
}

#[test]
fn hash1_21_parity_edge_cases() {
    // The importer's stored `text_sha` values (Twopop Studio `tests/oracle/parse/edge_cases.json`).
    assert_eq!(
        headings_and_hashes(EDGE_CASES),
        expect(&[
            (
                "THE ROOF AT DAWN",
                "6601466ae39394c1f0dfed5d957f82226c7609b2ad63a9aaa17728e28c10e625"
            ),
            (
                "INT. LAUNDERETTE - DAY",
                "7f243b3424fe00de4b1dc7bc2ae3e735aa7eee24297afa2f8b0df943b1443508"
            ),
        ])
    );
    // The title page and the opening Action are outside every scene, yet the first scene still
    // gets the importer's hash.
    let sp = parse(EDGE_CASES).screenplay();
    assert!(sp.title_page.is_some());
    assert!(!sp.preamble.elements.is_empty());
}

#[test]
fn hash1_22_parity_two_scene_and_blue_rev2() {
    assert_eq!(
        headings_and_hashes(TWO_SCENE),
        expect(&[
            (
                "INT. ARCHIVE VAULT - NIGHT",
                "8436c17623fcfd3e6c813afdcb540d9d52ebd44de24cb9d794ae823e9b46919d"
            ),
            (
                "INT. ARCHIVE VAULT - NIGHT - REVERSE",
                "029202aa944de57e8e6dd8b4a7a652e66b06a9cadd782e2fd5b9c213a1ed87cf"
            ),
        ])
    );
    assert_eq!(
        headings_and_hashes(BLUE_REV2),
        expect(&[
            (
                "INT. ARCHIVE VAULT - NIGHT",
                "0b24c579ece4de4c253589548b6394fb40fe17074f91b6f13ac0d1455bce333d"
            ),
            (
                "INT. ARCHIVE STAIRWELL - NIGHT",
                "9194840a945b5aa208721a8c95833556399bc71f6a23eda650af60a8f0fdd81f"
            ),
        ])
    );
}
