use std::ffi::CString;

use encrust_core::{Encrust, Hashbytes, Hashstring, Sensitivity};
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::parser::{ArrayElement, FilePath, LiteralKind, LiteralNode, ToHashBytes, ToHashString};

macro_rules! number_to_token_stream {
    ($ty:ty, $num:ident, $encruster:ident) => {{
        let mut n = $num;
        <$ty as Encrust>::toggle_encrust(&mut n, $encruster);
        EncrustedTokenStream {
            type_stream: quote! {$ty},
            value_stream: quote! {#n},
        }
    }};
}

/// Convert an array of number literals to a `Vec` of the underlying numbers and encrust it. This
/// relies on arrays and `Vec`s both using the same slice-based encrusting.
macro_rules! array_to_token_stream {
    ($ty:ty, $variant:ident, $arr:ident, $encruster:ident) => {{
        let mut vec: Vec<$ty> = $arr
            .iter()
            .map(|elem| {
                if let ArrayElement::$variant(v) = elem {
                    *v
                } else {
                    // The parser should reject any heterogenous array.
                    unreachable!()
                }
            })
            .collect();
        <Vec<$ty> as Encrust>::toggle_encrust(&mut vec, $encruster);
        let len = vec.len();
        EncrustedTokenStream {
            type_stream: quote! { [$ty; #len] },
            value_stream: quote! { [#(#vec),*] },
        }
    }};
}

#[derive(Debug)]
pub struct TokenStreamError {
    msg: String,
    span: Span,
}

impl std::error::Error for TokenStreamError {}

impl std::fmt::Display for TokenStreamError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EncrustedTokenStream {
    pub(crate) type_stream: proc_macro2::TokenStream,
    pub(crate) value_stream: proc_macro2::TokenStream,
}

impl EncrustedTokenStream {
    fn into_output_tokens(self, seed: u64) -> proc_macro::TokenStream {
        let EncrustedTokenStream {
            type_stream,
            value_stream,
        } = self;
        quote! {
            unsafe {
                ::encrust::Encrusted::<#type_stream>::from_encrusted_data(
                    #value_stream,
                    #seed
                )
            }
        }
        .into()
    }

    fn from_array_element_vec(arr: Vec<ArrayElement>, encruster: &mut impl Rng) -> Self {
        let first = arr
            .first()
            .expect("Parser should guarantee non-empty arrays.");

        match first {
            ArrayElement::NestedArray(_) => {
                let encrusted_items: Vec<_> = arr
                    .into_iter()
                    .map(|elem| {
                        if let ArrayElement::NestedArray(nested) = elem {
                            EncrustedTokenStream::from_array_element_vec(nested, encruster)
                        } else {
                            unreachable!("Parser should guarantee a homogenous array.")
                        }
                    })
                    .collect();

                let inner_type_stream = encrusted_items.first().unwrap().type_stream.clone();
                let encrusted_values: Vec<_> = encrusted_items
                    .into_iter()
                    .map(|ets| ets.value_stream)
                    .collect();
                let len = encrusted_values.len();

                EncrustedTokenStream {
                    type_stream: quote! { [#inner_type_stream; #len] },
                    value_stream: quote! { [#(#encrusted_values),*] },
                }
            }
            ArrayElement::U8(_) => array_to_token_stream!(u8, U8, arr, encruster),
            ArrayElement::U16(_) => array_to_token_stream!(u16, U16, arr, encruster),
            ArrayElement::U32(_) => array_to_token_stream!(u32, U32, arr, encruster),
            ArrayElement::U64(_) => array_to_token_stream!(u64, U64, arr, encruster),
            ArrayElement::U128(_) => array_to_token_stream!(u128, U128, arr, encruster),
            ArrayElement::Usize(_) => array_to_token_stream!(usize, Usize, arr, encruster),
            ArrayElement::I8(_) => array_to_token_stream!(i8, I8, arr, encruster),
            ArrayElement::I16(_) => array_to_token_stream!(i16, I16, arr, encruster),
            ArrayElement::I32(_) => array_to_token_stream!(i32, I32, arr, encruster),
            ArrayElement::I64(_) => array_to_token_stream!(i64, I64, arr, encruster),
            ArrayElement::I128(_) => array_to_token_stream!(i128, I128, arr, encruster),
            ArrayElement::Isize(_) => array_to_token_stream!(isize, Isize, arr, encruster),
        }
    }
}

pub(crate) trait ToEncrustedTokenStream {
    fn to_token_stream(
        self,
        encruster: &mut impl Rng,
    ) -> Result<EncrustedTokenStream, TokenStreamError>;

    fn generate_output_tokens(self) -> proc_macro::TokenStream
    where
        Self: Sized,
    {
        let seed = rand::rng().next_u64();
        let mut encruster = SmallRng::seed_from_u64(seed);

        match self.to_token_stream(&mut encruster) {
            Ok(encrusted_stream) => encrusted_stream.into_output_tokens(seed),
            Err(error) => {
                let error_message = format!("{error}");
                quote_spanned! {error.span=>
                    compile_error!(#error_message)
                }
                .into()
            }
        }
    }
}

impl ToEncrustedTokenStream for LiteralKind {
    fn to_token_stream(
        self,
        encruster: &mut impl Rng,
    ) -> Result<EncrustedTokenStream, TokenStreamError> {
        Ok(match self {
            LiteralKind::U8(n) => number_to_token_stream!(u8, n, encruster),
            LiteralKind::U16(n) => number_to_token_stream!(u16, n, encruster),
            LiteralKind::U32(n) => number_to_token_stream!(u32, n, encruster),
            LiteralKind::U64(n) => number_to_token_stream!(u64, n, encruster),
            LiteralKind::U128(n) => number_to_token_stream!(u128, n, encruster),
            LiteralKind::Usize(n) => number_to_token_stream!(usize, n, encruster),
            LiteralKind::I8(n) => number_to_token_stream!(i8, n, encruster),
            LiteralKind::I16(n) => number_to_token_stream!(i16, n, encruster),
            LiteralKind::I32(n) => number_to_token_stream!(i32, n, encruster),
            LiteralKind::I64(n) => number_to_token_stream!(i64, n, encruster),
            LiteralKind::I128(n) => number_to_token_stream!(i128, n, encruster),
            LiteralKind::Isize(n) => number_to_token_stream!(isize, n, encruster),
            LiteralKind::String(s) => {
                let mut bytes = s.as_bytes().to_vec();
                <String as Encrust>::toggle_encrust(&mut bytes, encruster);
                EncrustedTokenStream {
                    type_stream: quote! { ::encrust::__private::String },
                    value_stream: quote! { ::encrust::__private::vec![#(#bytes),*] },
                }
            }
            LiteralKind::BString(bstring) => {
                let mut bytes = bstring.clone();
                let len = bytes.len();
                <Vec<u8> as Encrust>::toggle_encrust(&mut bytes, encruster);
                EncrustedTokenStream {
                    type_stream: quote! { [u8; #len] },
                    value_stream: quote! { [#(#bytes),*] },
                }
            }
            LiteralKind::CString(cstr) => {
                let mut bytes = cstr.as_bytes_with_nul().to_vec();
                <::encrust_core::__private::CString as Encrust>::toggle_encrust(
                    &mut bytes, encruster,
                );
                EncrustedTokenStream {
                    type_stream: quote! { ::encrust::__private::CString },
                    value_stream: quote! { ::encrust::__private::vec![#(#bytes),*] },
                }
            }
            LiteralKind::Array(arr) => EncrustedTokenStream::from_array_element_vec(arr, encruster),
        })
    }
}

pub struct StringFileReader(FilePath);

impl From<FilePath> for StringFileReader {
    fn from(path: FilePath) -> Self {
        Self(path)
    }
}

impl ToEncrustedTokenStream for StringFileReader {
    fn to_token_stream(
        self,
        encruster: &mut impl Rng,
    ) -> Result<EncrustedTokenStream, TokenStreamError> {
        match std::fs::read_to_string(&self.0.path) {
            Ok(s) => LiteralNode {
                kind: LiteralKind::String(s),
                span: Span::call_site(),
            }
            .kind
            .to_token_stream(encruster),
            Err(error) => Err(TokenStreamError {
                msg: format!(
                    "Error when attempting to read `{}` to a `String`: {}",
                    self.0.path.display(),
                    error
                ),
                span: self.0.span,
            }),
        }
    }
}

pub struct CStringFileReader(FilePath);

impl From<FilePath> for CStringFileReader {
    fn from(path: FilePath) -> Self {
        Self(path)
    }
}

impl ToEncrustedTokenStream for CStringFileReader {
    fn to_token_stream(
        self,
        encruster: &mut impl Rng,
    ) -> Result<EncrustedTokenStream, TokenStreamError> {
        match std::fs::read_to_string(&self.0.path) {
            Ok(s) => {
                let cstr = CString::new(s).map_err(|_| TokenStreamError {
                    msg: format!(
                        "Error when attempting to file contents in `{}` to a `CString`.",
                        self.0.path.display(),
                    ),
                    span: self.0.span,
                })?;

                LiteralNode {
                    kind: LiteralKind::CString(cstr),
                    span: Span::call_site(),
                }
                .kind
                .to_token_stream(encruster)
            }
            Err(error) => Err(TokenStreamError {
                msg: format!(
                    "Error when attempting to read `{}` to a `CString`: {}",
                    self.0.path.display(),
                    error
                ),
                span: self.0.span,
            }),
        }
    }
}

pub struct BytesFileReader(FilePath);

impl From<FilePath> for BytesFileReader {
    fn from(path: FilePath) -> Self {
        Self(path)
    }
}

impl ToEncrustedTokenStream for BytesFileReader {
    fn to_token_stream(
        self,
        encruster: &mut impl Rng,
    ) -> Result<EncrustedTokenStream, TokenStreamError> {
        match std::fs::read(&self.0.path) {
            Ok(bytes) => LiteralNode {
                kind: LiteralKind::Array(bytes.into_iter().map(ArrayElement::U8).collect()),
                span: Span::call_site(),
            }
            .kind
            .to_token_stream(encruster),
            Err(error) => Err(TokenStreamError {
                msg: format!(
                    "Error when attempting to read `{}` to a byte array: {}",
                    self.0.path.display(),
                    error
                ),
                span: self.0.span,
            }),
        }
    }
}

#[cfg(feature = "hashstrings")]
impl ToHashString {
    pub fn generate_output_tokens_case_sensitive(&self) -> proc_macro::TokenStream {
        let seed = rand::rng().next_u64();
        let hashstring = Hashstring::new(&self.0, seed, Sensitivity::CaseSensitive);
        let value = hashstring.get_raw_value();

        quote! {
            ::encrust::Hashstring::new_from_raw_value(
                #value,
                #seed,
                ::encrust::Sensitivity::CaseSensitive
            )
        }
        .into()
    }

    pub fn generate_output_tokens_case_insensitive(&self) -> proc_macro::TokenStream {
        let seed = rand::rng().next_u64();
        let hashstring = Hashstring::new(&self.0, seed, Sensitivity::CaseInsensitive);
        let value = hashstring.get_raw_value();

        quote! {
            ::encrust::Hashstring::new_from_raw_value(
                #value,
                #seed,
                ::encrust::Sensitivity::CaseInsensitive
            )
        }
        .into()
    }
}

#[cfg(feature = "hashstrings")]
impl ToHashBytes {
    pub fn generate_output_tokens(&self) -> proc_macro::TokenStream {
        let seed = rand::rng().next_u64();
        let hashbytes = Hashbytes::new(&self.0, seed);
        let value = hashbytes.get_raw_value();

        quote! {
            ::encrust::Hashbytes::new_from_raw_value(#value, #seed)
        }
        .into()
    }
}
