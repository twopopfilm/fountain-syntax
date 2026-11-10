//! `SPEC.md` 11.1: digests and their text form. Builds without the `hash` feature.

use fountain_syntax::{Digest, HASH_SCHEME_CURRENT, HashScheme};

fn counting_bytes() -> [u8; 32] {
    std::array::from_fn(|i| i as u8)
}

const HEX: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

#[test]
fn hash2_38_text_form() {
    let v2 = Digest {
        scheme: HashScheme::V2,
        bytes: counting_bytes(),
    };
    let v1 = Digest {
        scheme: HashScheme::V1,
        bytes: counting_bytes(),
    };
    assert_eq!(v2.to_string(), format!("fs2:{HEX}"));
    assert_eq!(v1.to_string(), HEX);
    assert_eq!(v2.to_string().parse::<Digest>(), Ok(v2));
    assert_eq!(v1.to_string().parse::<Digest>(), Ok(v1));
    assert_eq!(HEX.parse::<Digest>().unwrap().scheme, HashScheme::V1);

    let upper = HEX.to_uppercase();
    let bad = [
        upper.clone(),
        format!("fs2:{upper}"),
        HEX[..63].to_string(),
        format!("{HEX}0"),
        format!("fs2:{}", &HEX[..63]),
        format!("fs2:{HEX}0"),
        format!("fs1:{HEX}"),
        format!("fs3:{HEX}"),
        format!(" {HEX}"),
        format!("{HEX} "),
        format!("fs2:{HEX}\n"),
        format!("fs2: {HEX}"),
        format!("FS2:{HEX}"),
        "fs2:".to_string(),
        "g".repeat(64),
        // Non-ASCII text of 64 bytes must be an error, not a panic.
        "é".repeat(32),
        String::new(),
    ];
    for text in bad {
        let err = text.parse::<Digest>().expect_err(&text);
        assert!(!err.to_string().is_empty());
    }
}

#[test]
fn hash2_39_scheme_numbers() {
    assert_eq!(HASH_SCHEME_CURRENT, 2);
    assert_eq!(HashScheme::V2.number(), HASH_SCHEME_CURRENT);
    assert_eq!(HashScheme::V1.number(), 1);
    let v1 = Digest {
        scheme: HashScheme::V1,
        bytes: counting_bytes(),
    };
    let v2 = Digest {
        scheme: HashScheme::V2,
        bytes: counting_bytes(),
    };
    assert_ne!(v1, v2);
}
