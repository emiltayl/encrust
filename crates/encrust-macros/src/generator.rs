use std::ffi::CString;

use encrust_core::{Encrust, Hashbytes, Hashstring, Sensitivity};
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};

use crate::parser::{FilePath, Literal, ToHashBytes, ToHashString};

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

pub trait ToEncrustedTokenStream {
    fn to_token_stream(
        &self,
        encruster: &mut impl RngCore,
    ) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), TokenStreamError>;

    fn generate_output_tokens(&self) -> proc_macro::TokenStream {
        let seed = rand::rng().next_u64();
        let mut encruster = SmallRng::seed_from_u64(seed);

        match self.to_token_stream(&mut encruster) {
            Ok((type_stream, value_stream)) => quote! {
                unsafe {
                    ::encrust::Encrusted::<#type_stream>::from_encrusted_data(
                        #value_stream,
                        #seed
                    )
                }
            },
            Err(error) => {
                let error_message = format!("{error}");
                quote_spanned! {error.span=>
                    compile_error!(#error_message)
                }
            }
        }
        .into()
    }
}

macro_rules! number_to_token_stream {
    ($ty:ty, $num:ident, $encruster:ident) => {{
        let mut n = *$num;
        <$ty as Encrust>::toggle_encrust(&mut n, $encruster);
        (quote! {$ty}, quote! {#n})
    }};
}

impl ToEncrustedTokenStream for Literal {
    fn to_token_stream(
        &self,
        encruster: &mut impl RngCore,
    ) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), TokenStreamError> {
        Ok(match self {
            Self::U8(n) => number_to_token_stream!(u8, n, encruster),
            Self::U16(n) => number_to_token_stream!(u16, n, encruster),
            Self::U32(n) => number_to_token_stream!(u32, n, encruster),
            Self::U64(n) => number_to_token_stream!(u64, n, encruster),
            Self::U128(n) => number_to_token_stream!(u128, n, encruster),
            Self::Usize(n) => number_to_token_stream!(usize, n, encruster),
            Self::I8(n) => number_to_token_stream!(i8, n, encruster),
            Self::I16(n) => number_to_token_stream!(i16, n, encruster),
            Self::I32(n) => number_to_token_stream!(i32, n, encruster),
            Self::I64(n) => number_to_token_stream!(i64, n, encruster),
            Self::I128(n) => number_to_token_stream!(i128, n, encruster),
            Self::Isize(n) => number_to_token_stream!(isize, n, encruster),
            Self::String(s) => {
                let mut bytes = s.as_bytes().to_vec();
                <String as Encrust>::toggle_encrust(&mut bytes, encruster);
                (quote! { String }, quote! { vec![#(#bytes),*] })
            }
            Self::BString(bstring) => {
                let encrusted_bytes = bstring
                    .iter()
                    .map(|n| number_to_token_stream!(u8, n, encruster))
                    .collect::<Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream)>>();

                let encrusted_tokens: Vec<proc_macro2::TokenStream> =
                    encrusted_bytes.into_iter().map(|(_, val)| val).collect();
                let len = encrusted_tokens.len();

                (quote! { [u8; #len] }, quote! {[#(#encrusted_tokens),*]})
            }
            Self::CString(cstr) => {
                let mut bytes = cstr.as_bytes_with_nul().to_vec();
                <::encrust_core::__private::CString as Encrust>::toggle_encrust(
                    &mut bytes, encruster,
                );
                (
                    quote! { ::encrust_core::__private::CString },
                    quote! { vec![#(#bytes),*] },
                )
            }
            Self::Array(arr) => {
                let encrusted_items = arr
                    .iter()
                    .map(|el| el.to_token_stream(encruster))
                    .collect::<Result<
                        Vec<(proc_macro2::TokenStream, proc_macro2::TokenStream)>,
                        TokenStreamError,
                    >>()?;

                let encrusted_values: Vec<proc_macro2::TokenStream> =
                    encrusted_items.into_iter().map(|(_, val)| val).collect();
                let len = encrusted_values.len();

                (quote! { [_; #len] }, quote! {[#(#encrusted_values),*]})
            }
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
        &self,
        encruster: &mut impl RngCore,
    ) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), TokenStreamError> {
        match std::fs::read_to_string(&self.0.path) {
            Ok(s) => Literal::String(s).to_token_stream(encruster),
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
        &self,
        encruster: &mut impl RngCore,
    ) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), TokenStreamError> {
        match std::fs::read_to_string(&self.0.path) {
            Ok(s) => {
                let cstr = CString::new(s).map_err(|_| TokenStreamError {
                    msg: format!(
                        "Error when attempting to file contents in `{}` to a `CString`.",
                        self.0.path.display(),
                    ),
                    span: self.0.span,
                })?;

                Literal::CString(cstr).to_token_stream(encruster)
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
        &self,
        encruster: &mut impl RngCore,
    ) -> Result<(proc_macro2::TokenStream, proc_macro2::TokenStream), TokenStreamError> {
        match std::fs::read(&self.0.path) {
            Ok(bytes) => Literal::Array(bytes.into_iter().map(Literal::U8).collect())
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
