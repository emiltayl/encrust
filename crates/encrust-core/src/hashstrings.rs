//! Hash strings or bytes at run time without storing the original strings or bytes.
//!
//! Macros can calculate the hash at compile time so the original data does not need to appear in
//! the executable. See the `encrust` crate for macro examples.

// Note that items in this crate are behind `#[cfg(feature = "hashstrings")]` to ensure that the
// generated documentation can display that the types require having the "hashstrings" feature
// enabled.

use rapidhash::v3::{RapidSecrets, rapidhash_v3_seeded};
use zeroize::Zeroize;

/// Used to specify whether a [`Hashstring`] should ignore case when comparing strings.
#[cfg(feature = "hashstrings")]
pub enum Sensitivity {
    /// Ignore case when comparing strings.
    CaseInsensitive,
    /// Do *NOT* ignore case when comparing strings.
    CaseSensitive,
}

/// Represents the hash of a string.
///
/// This can be used to compare strings without storing the original string.
///
/// # Example
/// ```
/// use encrust_core::{Hashstring, Sensitivity};
///
/// let hashstring = Hashstring::new("A string", 0xabcdef, Sensitivity::CaseSensitive);
/// assert!(hashstring == "A string");
/// assert!(hashstring != "a string");
///
/// let case_insensitive_hashstring =
///     Hashstring::new("A string", 0xfedcba, Sensitivity::CaseInsensitive);
/// assert!(case_insensitive_hashstring == "A string");
/// assert!(case_insensitive_hashstring == "a string");
/// ```
#[cfg(feature = "hashstrings")]
pub struct Hashstring {
    value: u64,
    seed: u64,
    sensitivity: Sensitivity,
}

#[cfg(feature = "hashstrings")]
impl Hashstring {
    /// Create a new [`Hashstring`] using the provided string and seed.
    ///
    /// Note that if `Sensitivity::CaseInsensitive` is used, a new `String` is allocated with the
    /// provided `s` converted to lowercase. The newly allocated string is overwritten using
    /// `Zeroize` after calculating the hash.
    ///
    /// This function does not zeroize the original string. To avoid storing the string in the
    /// executable, use the `hashstring!` macro.
    pub fn new(s: &str, seed: u64, sensitivity: Sensitivity) -> Self {
        let rapid_secrets = RapidSecrets::seed_cpp(seed);
        let value = match sensitivity {
            Sensitivity::CaseInsensitive => {
                let mut lowercase_string = s.to_lowercase();
                let hash = rapidhash_v3_seeded(lowercase_string.as_bytes(), &rapid_secrets);
                Zeroize::zeroize(&mut lowercase_string);

                hash
            }
            Sensitivity::CaseSensitive => rapidhash_v3_seeded(s.as_bytes(), &rapid_secrets),
        };

        Self {
            value,
            seed,
            sensitivity,
        }
    }

    /// Used by the macros to get the hash value to create `Hashstring` from raw data.
    /// Should not be used outside of the provided macros.
    #[doc(hidden)]
    #[cfg(feature = "macros")]
    pub fn get_raw_value(&self) -> u64 {
        self.value
    }

    /// Used by the macros to create `Hashstring` from raw data.
    /// Should not be used outside of the provided macros.
    #[doc(hidden)]
    #[cfg(feature = "macros")]
    pub fn new_from_raw_value(value: u64, seed: u64, sensitivity: Sensitivity) -> Self {
        Self {
            value,
            seed,
            sensitivity,
        }
    }
}

#[cfg(feature = "hashstrings")]
impl PartialEq<&str> for Hashstring {
    fn eq(&self, other: &&str) -> bool {
        let seed = RapidSecrets::seed_cpp(self.seed);
        let other_value = match self.sensitivity {
            Sensitivity::CaseInsensitive => {
                rapidhash_v3_seeded(other.to_lowercase().as_bytes(), &seed)
            }
            Sensitivity::CaseSensitive => rapidhash_v3_seeded(other.as_bytes(), &seed),
        };

        self.value == other_value
    }
}

/// Represents the hash of a byte slice.
///
/// This can be used to compare byte slices without storing the original bytes.
///
/// # Example
/// ```
/// use encrust_core::Hashbytes;
///
/// let hashbytes = Hashbytes::new(&[1, 2, 3], 0xc0ffee);
/// assert!(hashbytes == &[1, 2, 3]);
/// assert!(hashbytes != &[4, 5, 6]);
/// ```
#[cfg(feature = "hashstrings")]
pub struct Hashbytes {
    value: u64,
    seed: u64,
}

#[cfg(feature = "hashstrings")]
impl Hashbytes {
    /// Create a new [`Hashbytes`] using the provided `u8` slice and seed.
    ///
    /// This function does not zeroize the original data. To avoid storing the bytes in the
    /// executable, use the `hashbytes!` macro.
    pub fn new(bytes: &[u8], seed: u64) -> Self {
        let rapid_secrets = RapidSecrets::seed_cpp(seed);
        let value = rapidhash_v3_seeded(bytes, &rapid_secrets);

        Self { value, seed }
    }

    /// Used by the macros to get the hash value to create `Hashbytes` from raw data.
    /// Should not be used outside of the provided macros.
    #[doc(hidden)]
    #[cfg(feature = "macros")]
    pub fn get_raw_value(&self) -> u64 {
        self.value
    }

    /// Used by the macros to create `Hashbytes` from raw data.
    /// Should not be used outside of the provided macros.
    #[doc(hidden)]
    #[cfg(feature = "macros")]
    pub fn new_from_raw_value(value: u64, seed: u64) -> Self {
        Self { value, seed }
    }
}

#[cfg(feature = "hashstrings")]
impl PartialEq<&[u8]> for Hashbytes {
    fn eq(&self, other: &&[u8]) -> bool {
        let rapid_secrets = RapidSecrets::seed_cpp(self.seed);
        let other_value = rapidhash_v3_seeded(other, &rapid_secrets);

        self.value == other_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A_STRING: &str = "A string😶";
    const A_LOWERCASE_STRING: &str = "a string😶";
    const A_STRING_BYTES: &[u8] = A_STRING.as_bytes();
    const A_LOWERCASE_STRING_BYTES: &[u8] = A_LOWERCASE_STRING.as_bytes();

    const SEED: u64 = 0x2357_bd11_1317_1d1f;

    #[test]
    fn test_hashstrings() {
        let case_sensitive_hashstring = Hashstring::new(A_STRING, SEED, Sensitivity::CaseSensitive);
        let case_insensitive_hashstring =
            Hashstring::new(A_STRING, SEED, Sensitivity::CaseInsensitive);

        assert!(case_sensitive_hashstring == A_STRING);
        assert!(case_sensitive_hashstring != A_LOWERCASE_STRING);
        assert!(case_insensitive_hashstring == A_STRING);
        assert!(case_insensitive_hashstring == A_LOWERCASE_STRING);
    }

    #[test]
    fn test_hashbytes() {
        let hashbytes = Hashbytes::new(A_STRING_BYTES, SEED);

        assert!(hashbytes == A_STRING_BYTES);
        assert!(hashbytes != A_LOWERCASE_STRING_BYTES);
    }

    #[test]
    fn test_empty_hashstrings() {
        let empty = Hashstring::new("", SEED, Sensitivity::CaseSensitive);
        assert!(empty == "");
        assert!(empty != " ");
    }

    #[test]
    fn test_empty_hashbytes() {
        let empty = Hashbytes::new(&[], SEED);
        assert!(empty == &[][..]);
        assert!(empty != &[0][..]);
    }

    /// Test to make sure that a previously hashed values can still be used with current version of
    /// this crate.
    #[test]
    fn ensure_hashstring_bytes_has_not_changed() {
        // Created from `A_LOWERCASE_STRING`
        #[allow(
            clippy::unreadable_literal,
            reason = "Created using a random seed, has no special meaning outside of this crate."
        )]
        let value = 856128386601824369u64;

        let hashed_string_lower =
            Hashstring::new_from_raw_value(value, SEED, Sensitivity::CaseSensitive);

        let hashed_string_lower_ci =
            Hashstring::new_from_raw_value(value, SEED, Sensitivity::CaseInsensitive);

        let hashed_bytes = Hashbytes::new_from_raw_value(value, SEED);

        assert!(hashed_string_lower != A_STRING);
        assert!(hashed_string_lower == A_LOWERCASE_STRING);
        assert!(hashed_string_lower_ci == A_STRING);
        assert!(hashed_string_lower_ci == A_LOWERCASE_STRING);
        assert!(hashed_bytes != A_STRING_BYTES);
        assert!(hashed_bytes == A_LOWERCASE_STRING_BYTES);
    }
}
