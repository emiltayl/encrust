#![cfg_attr(docsrs, feature(doc_cfg))]

//! Crate implementing macros for `encrust`. See the main crate for documentation.

mod derive;
mod generator;
mod parser;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::{
    generator::{BytesFileReader, StringFileReader, ToEncrustedTokenStream},
    parser::{FilePath, Literal, LiteralVec, ToHashBytes, ToHashString},
};

/// Encrust a literal value so the actual data is obfuscated before being included in the binary.
/// Currently integers, strings and arrays of (arrays of) integers and strings are accepted.
///
/// Integers require their data type suffixed (`-1i8`, `127u16` etc).
///
/// # Examples
/// ```
/// # extern crate encrust_core as encrust;
/// # use encrust_macros::encrust;
/// let mut num = encrust!(0u8);
/// assert_eq!(0u8, *num.decrust());
/// let mut string = encrust!("This is a string");
/// assert_eq!("This is a string", string.decrust().as_str());
/// let mut array = encrust!([1i32, 2i32, 3i32]);
/// assert_eq!(&[1i32, 2i32, 3i32], array.decrust().as_slice());
/// ```
#[proc_macro]
pub fn encrust(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as Literal).generate_output_tokens()
}

/// Encrust a vec of literals. This works similarly to [`encrust!`] and supports the same data
/// types, but puts the data in a `vec`.
///
/// # Example
/// ```
/// # extern crate encrust_core as encrust;
/// # use encrust_macros::encrust_vec;
/// let mut a_vec = encrust_vec![1i32, 2i32, 3i32];
/// assert_eq!(
///     vec![1i32, 2i32, 3i32].as_slice(),
///     a_vec.decrust().as_slice()
/// );
/// ```
#[proc_macro]
pub fn encrust_vec(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as LiteralVec).generate_output_tokens()
}

/// Read the contents of a file into a string and encrust it so the actual file contents is
/// obfuscated before being included in the binary.
///
/// Unless an absolute path is given, the file is read relative to the `CARGO_MANIFEST_DIR`
/// environment variable, which is set to the directory containing the crate's `Cargo.toml` file.
/// *Note* that this is not identical to `include_str!`'s behavior, which reads relative to the file
/// using the macro.
///
/// # Example
/// ```
/// # extern crate encrust_core as encrust;
/// # use encrust_macros::encrust_file_string;
/// let mut cargo_toml = encrust_file_string!("Cargo.toml");
/// ```
#[proc_macro]
pub fn encrust_file_string(input: TokenStream) -> TokenStream {
    StringFileReader::from(parse_macro_input!(input as FilePath)).generate_output_tokens()
}

/// Read the contents of a file into a `u8` array and encrust it so the actual file contents is
/// obfuscated before being included in the binary.
///
/// Unless an absolute path is given, the file is read relative to the `CARGO_MANIFEST_DIR`
/// environment variable, which is set to the directory containing the crate's `Cargo.toml` file.
/// *Note* that this is not identical to `include_bytes!`'s behavior, which reads relative to the
/// file using the macro.
///
/// # Example
/// ```
/// # extern crate encrust_core as encrust;
/// # use encrust_macros::encrust_file_bytes;
/// let mut cargo_toml = encrust_file_bytes!("Cargo.toml");
/// ```
#[proc_macro]
pub fn encrust_file_bytes(input: TokenStream) -> TokenStream {
    BytesFileReader::from(parse_macro_input!(input as FilePath)).generate_output_tokens()
}

/// Hash a string so that it can be searched for in the resulting executable without including the
/// actual string. This macro creates a case sensitive `encrust::Hashstring`.
///
/// # Example
/// ```
/// # extern crate encrust_core as encrust;
/// # use encrust_macros::hashstring;
/// let look_for_me = hashstring!("Find me!");
/// assert!(look_for_me == "Find me!");
/// assert!(look_for_me != "fInD Me!");
/// ```
#[proc_macro]
#[cfg(feature = "hashstrings")]
pub fn hashstring(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as ToHashString).generate_output_tokens_case_sensitive()
}

/// Similar to the [`hashstring!`] macro, but with a case insensitive `encrust::Hashstring`.
///
/// # Example
/// ```
/// # extern crate encrust_core as encrust;
/// # use encrust_macros::hashstring_ci;
/// let look_for_me = hashstring_ci!("Find me!");
/// assert!(look_for_me == "Find me!");
/// assert!(look_for_me == "fInD Me!");
/// ```
#[proc_macro]
#[cfg(feature = "hashstrings")]
pub fn hashstring_ci(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as ToHashString).generate_output_tokens_case_insensitive()
}

/// Hash an array of bytes so that the byte pattern can be searched for without including the bytes
/// themselves in the executable.
///
/// # Example
/// ```
/// # extern crate encrust_core as encrust;
/// # use encrust_macros::hashbytes;
/// let look_for_me = hashbytes!([0, 1, 2, 3]);
/// assert!(look_for_me == &[0, 1, 2, 3]);
/// ```
#[proc_macro]
#[cfg(feature = "hashstrings")]
pub fn hashbytes(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as ToHashBytes).generate_output_tokens()
}

/// Derive macro to allow custom `struct`s and `enum`s to be encrusted.
///
/// This requires that all fields are `Encrustable`. Currently, no other options are available.
#[proc_macro_derive(Encrustable)]
pub fn derive_encrustable_macro(input: TokenStream) -> TokenStream {
    derive::derive_encrustable(parse_macro_input!(input as syn::DeriveInput))
}
