use encrust_core::{Encrustable, Hashbytes, Hashstring, Sensitivity};
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use rand::{RngCore, SeedableRng, rngs::SmallRng};

use crate::parser::{FilePath, Literal, LiteralVec, ToHashBytes, ToHashString};

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
    ) -> Result<proc_macro2::TokenStream, TokenStreamError>;

    fn generate_output_tokens(&self) -> proc_macro::TokenStream {
        let seed = rand::rng().next_u64();
        let mut encruster = SmallRng::seed_from_u64(seed);

        match self.to_token_stream(&mut encruster) {
            Ok(token_stream) => quote! {
                unsafe {
                    ::encrust::Encrusted::from_encrusted_data(
                        #token_stream,
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
    ($num:ident, $encruster:ident) => {{
        let mut n = *$num;
        // Safety: The underlying data must be encrusted to be used with `from_encrusted_data`.
        // It should not be exposed without calling `toggle_encrust` again.
        unsafe {
            n.toggle_encrust($encruster);
        }
        quote! {#n}
    }};
}

impl ToEncrustedTokenStream for Literal {
    fn to_token_stream(
        &self,
        encruster: &mut impl RngCore,
    ) -> Result<proc_macro2::TokenStream, TokenStreamError> {
        Ok(match self {
            Self::U8(n) => number_to_token_stream!(n, encruster),
            Self::U16(n) => number_to_token_stream!(n, encruster),
            Self::U32(n) => number_to_token_stream!(n, encruster),
            Self::U64(n) => number_to_token_stream!(n, encruster),
            Self::U128(n) => number_to_token_stream!(n, encruster),
            Self::Usize(n) => number_to_token_stream!(n, encruster),
            Self::I8(n) => number_to_token_stream!(n, encruster),
            Self::I16(n) => number_to_token_stream!(n, encruster),
            Self::I32(n) => number_to_token_stream!(n, encruster),
            Self::I64(n) => number_to_token_stream!(n, encruster),
            Self::I128(n) => number_to_token_stream!(n, encruster),
            Self::Isize(n) => number_to_token_stream!(n, encruster),
            Self::String(s) => {
                let mut string = s.clone();

                // Safety: The underlying data must be encrusted to be used with
                // `from_encrusted_data`. It should not be exposed without calling `toggle_encrust`
                // again.
                unsafe {
                    string.toggle_encrust(encruster);
                }

                let bytes = Vec::from(string.as_bytes());

                #[cfg(feature = "std")]
                quote! {unsafe { ::std::string::String::from_utf8_unchecked([#(#bytes),*].to_vec()) }}
                #[cfg(not(feature = "std"))]
                quote! {unsafe { ::alloc::string::String::from_utf8_unchecked([#(#bytes),*].to_vec()) }}
            }
            Self::Array(arr) => {
                let encrusted_items = arr
                    .iter()
                    .map(|el| el.to_token_stream(encruster))
                    .collect::<Result<Vec<proc_macro2::TokenStream>, TokenStreamError>>()?;
                quote! {[#(#encrusted_items),*]}
            }
        })
    }
}

impl ToEncrustedTokenStream for LiteralVec {
    fn to_token_stream(
        &self,
        encruster: &mut impl RngCore,
    ) -> Result<proc_macro2::TokenStream, TokenStreamError> {
        let encrusted_items = self
            .0
            .iter()
            .map(|el| el.to_token_stream(encruster))
            .collect::<Result<Vec<proc_macro2::TokenStream>, TokenStreamError>>()?;
        Ok(quote! {[#(#encrusted_items),*].to_vec()})
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
    ) -> Result<proc_macro2::TokenStream, TokenStreamError> {
        match std::fs::read_to_string(&self.0.path) {
            Ok(s) => Literal::String(s).to_token_stream(encruster),
            Err(error) => Err(TokenStreamError {
                msg: format!(
                    "Error when attempting to read `{}` to a String: {}",
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
    ) -> Result<proc_macro2::TokenStream, TokenStreamError> {
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
