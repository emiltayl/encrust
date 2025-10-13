//! Functions to search for strings or bytes at run-time without having to include the strings
//! or byte patterns themselves in the binary.
//! Macros are used to make it possible to ensure that the plain text is not present in the
//! executable, see the documentation for [`encrust`] for examples of macro usage.

use rapidhash::v3::{RapidSecrets, rapidhash_v3_seeded};
use zeroize::Zeroize;

/// Used to specify whether a [`Hashstring`] should ignore case when comparing strings.
#[cfg_attr(docsrs, doc(cfg(feature = "hashstrings")))]
pub enum Sensitivity {
    /// Ignore case when comparing strings.
    CaseInsensitive,
    /// Do *NOT* ignore case when comparing strings.
    CaseSensitive,
}

/// The hash of a string.
/// Can be used to search for strings without storing the string itself in memory.
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
#[cfg_attr(docsrs, doc(cfg(feature = "hashstrings")))]
pub struct Hashstring {
    value: u64,
    seed: u64,
    sensitivity: Sensitivity,
}

impl Hashstring {
    /// Create a new [`Hashstring`] using the provided string and random seed.
    ///
    /// Note that if `Sensitivity::CaseInsensitive` is used, a new `String` is allocated with the
    /// provided `s` converted to lowercase. The newly allocated string is overwritten using
    /// `Zeroize` after calculating the hash.
    ///
    /// This function does not zeroize the original string. To avoid ever having the string in
    /// memory, it is recommended to use the `hashstring!` macro.
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
    #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
    #[cfg(feature = "macros")]
    pub fn get_raw_value(&self) -> u64 {
        self.value
    }

    /// Used by the macros to create `Hashstring` from raw data.
    /// Should not be used outside of the provided macros.
    #[doc(hidden)]
    #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
    #[cfg(feature = "macros")]
    pub fn new_from_raw_value(value: u64, seed: u64, sensitivity: Sensitivity) -> Self {
        Self {
            value,
            seed,
            sensitivity,
        }
    }
}

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

/// The hash of a slice of u8's.
/// Can be used to search for data without storing the data itself in memory.
///
/// # Example
/// ```
/// use encrust_core::Hashbytes;
///
/// let hashbytes = Hashbytes::new(&[1, 2, 3], 0xc0ffee);
/// assert!(hashbytes == &[1, 2, 3]);
/// assert!(hashbytes != &[4, 5, 6]);
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "hashstrings")))]
pub struct Hashbytes {
    value: u64,
    seed: u64,
}

impl Hashbytes {
    /// Create a new [`Hashbytes`] using the provided `u8` slice and random seed.
    ///
    /// This function does not zeroize the original data. To avoid ever having the data in memory,
    /// it is recommended to use the `hashbytes` macro.
    pub fn new(bytes: &[u8], seed: u64) -> Self {
        let rapid_secrets = RapidSecrets::seed_cpp(seed);
        let value = rapidhash_v3_seeded(bytes, &rapid_secrets);

        Self { value, seed }
    }

    /// Used by the macros to get the hash value to create `Hashbytes` from raw data.
    /// Should not be used outside of the provided macros.
    #[doc(hidden)]
    #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
    #[cfg(feature = "macros")]
    pub fn get_raw_value(&self) -> u64 {
        self.value
    }

    /// Used by the macros to create `Hashbytes` from raw data.
    /// Should not be used outside of the provided macros.
    #[doc(hidden)]
    #[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
    #[cfg(feature = "macros")]
    pub fn new_from_raw_value(value: u64, seed: u64) -> Self {
        Self { value, seed }
    }
}

impl PartialEq<&[u8]> for Hashbytes {
    fn eq(&self, other: &&[u8]) -> bool {
        let rapid_secrets = RapidSecrets::seed_cpp(self.seed);
        let other_value = rapidhash_v3_seeded(other, &rapid_secrets);

        self.value == other_value
    }
}

#[cfg(test)]
mod tests {
    use rand::RngCore;

    use super::*;

    const A_STRING: &str = "A string😶";
    const A_LOWERCASE_STRING: &str = "a string😶";
    const A_STRING_BYTES: &[u8] = A_STRING.as_bytes();
    const A_LOWERCASE_STRING_BYTES: &[u8] = A_LOWERCASE_STRING.as_bytes();

    #[test]
    fn test_hashstrings() {
        let case_sensitive_hashstring =
            Hashstring::new(A_STRING, rand::rng().next_u64(), Sensitivity::CaseSensitive);
        let case_insensitive_hashstring = Hashstring::new(
            A_STRING,
            rand::rng().next_u64(),
            Sensitivity::CaseInsensitive,
        );

        assert!(case_sensitive_hashstring == A_STRING);
        assert!(case_sensitive_hashstring != A_LOWERCASE_STRING);
        assert!(case_insensitive_hashstring == A_STRING);
        assert!(case_insensitive_hashstring == A_LOWERCASE_STRING);
    }

    #[test]
    fn test_hashbytes() {
        let hashbytes = Hashbytes::new(A_STRING_BYTES, rand::rng().next_u64());

        assert!(hashbytes == A_STRING_BYTES);
        assert!(hashbytes != A_LOWERCASE_STRING_BYTES);
    }

    /// Test to make sure that a previously encrusted object can be decrusted with the current
    /// version of `encrust`.
    #[test]
    fn ensure_hashstring_bytes_has_not_changed() {
        // Created from `A_LOWERCASE_STRING`
        #[allow(
            clippy::unreadable_literal,
            reason = "Created using a random seed, has no special meaning outside of this crate."
        )]
        let value = 10002744355855325072u64;

        #[allow(
            clippy::unreadable_literal,
            reason = "A random seed, has no special meaning outside of this crate."
        )]
        let seed = 15748439925883409278u64;

        let hashed_string_lower =
            Hashstring::new_from_raw_value(value, seed, Sensitivity::CaseSensitive);

        let hashed_string_lower_ci =
            Hashstring::new_from_raw_value(value, seed, Sensitivity::CaseInsensitive);

        let hashed_bytes = Hashbytes::new_from_raw_value(value, seed);

        assert!(hashed_string_lower != A_STRING);
        assert!(hashed_string_lower == A_LOWERCASE_STRING);
        assert!(hashed_string_lower_ci == A_STRING);
        assert!(hashed_string_lower_ci == A_LOWERCASE_STRING);
        assert!(hashed_bytes != A_STRING_BYTES);
        assert!(hashed_bytes == A_LOWERCASE_STRING_BYTES);
    }
}
