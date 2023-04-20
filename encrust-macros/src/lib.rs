#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! Crate implementing macros for `encrust`. See the main crate for
//! documentation.

mod derive;
mod generator;
mod parser;

use crate::{
    generator::{BytesFileReader, StringFileReader, ToTokenStream},
    parser::{FilePath, Literal, LiteralVec},
};

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Encrust a literal value so the actual data is encrypted before being
/// included in the binary.
///
/// Currently integers, strings and arrays of (arrays of) integers and strings
/// are accepted.
///
/// Integers require their data type suffixed (`-1i8`, `127u16` etc).
///
/// # Examples
/// ```
/// # use encrust_macros::encrust;
/// let mut num = encrust!(0u8);
/// assert_eq!(0u8, *num.decrust());
/// let mut string = encrust!("This is a string");
/// assert_eq!("This is a string", string.decrust().as_str());
/// let mut array = encrust!([1i32, 2i32, 3i32]);
/// assert_eq!(&[1i32, 2i32, 3i32], array.decrust().as_slice());
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[proc_macro]
pub fn encrust(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as Literal).generate_output_tokens()
}

/// Encrust a vec of literals. This works similarly to [`encrust!`] and supports
/// the same data types, but puts the data in a `vec`.
///
/// # Example
/// ```
/// # use encrust_macros::encrust_vec;
/// let mut a_vec = encrust_vec![1i32, 2i32, 3i32];
/// assert_eq!(vec![1i32,2i32,3i32].as_slice(), a_vec.decrust().as_slice());
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[proc_macro]
pub fn encrust_vec(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as LiteralVec).generate_output_tokens()
}

/// Read the contents of a file into a String and encrust it so the actual file
/// contents is encrypted before being included in the binary.
///
/// Unless an absolute path is given, the file is read relative to the
/// `CARGO_MANIFEST_DIR` variable, which is set to the directory containing the
/// crate's `Cargo.toml` file. *Note* that this is not identical to
/// `include_str!`'s behavior, which reads relative to the file using the macro.
///
/// # Example
/// ```
/// # use encrust_macros::encrust_file_string;
/// let mut cargo_toml = encrust_file_string!("Cargo.toml");
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[proc_macro]
pub fn encrust_file_string(input: TokenStream) -> TokenStream {
    StringFileReader::from(parse_macro_input!(input as FilePath)).generate_output_tokens()
}

/// Read the contents of a file into a `u8` array and encrust it so the actual
/// file contents is encrypted before being included in the binary.
///
/// Unless an absolute path is given, the file is read relative to the
/// `CARGO_MANIFEST_DIR` variable, which is set to the directory containing the
/// crate's `Cargo.toml` file. *Note* that this is not identical to
/// `include_bytes!`'s behavior, which reads relative to the file using the
/// macro.
///
/// # Example
/// ```
/// # use encrust_macros::encrust_file_bytes;
/// let mut cargo_toml = encrust_file_bytes!("Cargo.toml");
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[proc_macro]
pub fn encrust_file_bytes(input: TokenStream) -> TokenStream {
    BytesFileReader::from(parse_macro_input!(input as FilePath)).generate_output_tokens()
}

/// Derive macro to allow custom `struct`s and `enum`s to be encrusted.
///
/// This requires that all fields are `Encrustable`. Currently, no other options
/// are available.
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[proc_macro_derive(Encrustable)]
pub fn derive_encrustable_macro(input: TokenStream) -> TokenStream {
    derive::derive_encrustable(parse_macro_input!(input as syn::DeriveInput))
}
