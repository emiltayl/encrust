#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

mod generator;
mod parser;

use proc_macro::TokenStream;
use syn::parse_macro_input;

use crate::generator::{
    BytesFileReader, CStringFileReader, StringFileReader, ToEncrustedTokenStream,
};
use crate::parser::{FilePath, LiteralNode, ToHashBytes, ToHashString};

/// Encrust a literal value before it is included in the binary.
///
/// Currently integers, strings and arrays of integers are accepted. Arrays can be
/// nested arbitrarily deep.
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
/// assert_eq!("This is a string", &*string.decrust());
/// let mut array = encrust!([1i32, 2i32, 3i32]);
/// assert_eq!(&[1i32, 2i32, 3i32], &*array.decrust());
/// ```
#[proc_macro]
pub fn encrust(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as LiteralNode)
        .kind
        .generate_output_tokens()
}

/// Read the contents of a file into a string and encrust it before it is included in the binary.
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

/// Read the contents of a file into a `CString` and encrust it before it is included in the binary.
///
/// Unless an absolute path is given, the file is read relative to the `CARGO_MANIFEST_DIR`
/// environment variable, which is set to the directory containing the crate's `Cargo.toml` file.
/// *Note* that this is not identical to `include_str!`'s behavior, which reads relative to the file
/// using the macro.
///
/// # Example
/// ```
/// # extern crate encrust_core as encrust;
/// # use encrust_macros::encrust_file_cstring;
/// let mut cargo_toml = encrust_file_cstring!("Cargo.toml");
/// ```
#[proc_macro]
pub fn encrust_file_cstring(input: TokenStream) -> TokenStream {
    CStringFileReader::from(parse_macro_input!(input as FilePath)).generate_output_tokens()
}

/// Read the contents of a file into a `u8` array and encrust it before it is included in the
/// binary.
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
