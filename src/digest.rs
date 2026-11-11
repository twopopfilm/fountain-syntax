//! Digests and their text form (`SPEC.md` 11.1). These types need no dependency.

use core::fmt;
use core::str::FromStr;

/// The hash scheme a [`Digest`] was made by.
///
/// Scheme 2 is the default; scheme 1 is the frozen compatibility scheme. `Scene::hash_v1` (with the
/// `hash` feature) makes a `V1` digest; parsing and printing one is supported in every build.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HashScheme {
    /// Scheme 1, the compatibility scheme.
    V1,
    /// Scheme 2, the scheme that new code should use.
    V2,
}

impl HashScheme {
    /// The scheme number: 1 or 2.
    pub const fn number(self) -> u32 {
        match self {
            HashScheme::V1 => 1,
            HashScheme::V2 => 2,
        }
    }
}

/// The scheme that new code should use.
pub const HASH_SCHEME_CURRENT: u32 = 2;

/// A 32-byte hash and the scheme that made it.
///
/// Equality compares the scheme as well as the bytes. The text form is `fs2:` plus 64 lowercase
/// hexadecimal digits for scheme 2, and the bare 64 digits for scheme 1.
///
/// `HASH2-47`: the methods that compute a digest exist only with the `hash` feature. This example
/// fails to compile without it and runs with it.
///
#[cfg_attr(not(feature = "hash"), doc = "```compile_fail")]
#[cfg_attr(feature = "hash", doc = "```")]
/// let s = fountain_syntax::parse("INT. X\n").screenplay();
/// let _ = s.hash();
/// let _ = s.scenes[0].hash_v1();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Digest {
    /// The scheme that made `bytes`.
    pub scheme: HashScheme,
    /// The hash.
    pub bytes: [u8; 32],
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.scheme == HashScheme::V2 {
            f.write_str("fs2:")?;
        }
        self.bytes.iter().try_for_each(|b| write!(f, "{b:02x}"))
    }
}

/// Why a string is not a [`Digest`] text form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseDigestError {
    reason: &'static str,
}

impl fmt::Display for ParseDigestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a digest: {}", self.reason)
    }
}

impl std::error::Error for ParseDigestError {}

impl FromStr for Digest {
    type Err = ParseDigestError;

    /// Accepts 64 lowercase hex digits (scheme 1) or `fs2:` plus 64 (scheme 2), and nothing else.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (scheme, hex) = match text.strip_prefix("fs2:") {
            Some(hex) => (HashScheme::V2, hex),
            None => (HashScheme::V1, text),
        };
        let hex = hex.as_bytes();
        if hex.len() != 64 {
            return Err(ParseDigestError {
                reason: "expected 64 hexadecimal digits",
            });
        }
        let nibble = |c: u8| match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            _ => Err(ParseDigestError {
                reason: "expected lowercase hexadecimal digits",
            }),
        };
        let mut bytes = [0u8; 32];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = (nibble(hex[2 * i])? << 4) | nibble(hex[2 * i + 1])?;
        }
        Ok(Digest { scheme, bytes })
    }
}
