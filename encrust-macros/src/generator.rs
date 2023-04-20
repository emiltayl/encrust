use crate::parser::{FilePath, Literal, LiteralVec};

use chacha20::{cipher::KeyIvInit, Key, XChaCha8, XNonce};
use encrust_core::Encrustable;
use proc_macro2::Span;
use quote::{quote, quote_spanned};

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

pub trait ToTokenStream {
    fn to_token_stream(
        &self,
        encruster: &mut XChaCha8,
    ) -> Result<proc_macro2::TokenStream, TokenStreamError>;

    fn generate_output_tokens(&self) -> proc_macro::TokenStream {
        let raw_key: [u8; 32] = rand::random();
        let raw_nonce: [u8; 24] = rand::random();
        let key = Key::from(raw_key);
        let nonce = XNonce::from(raw_nonce);
        let mut encruster = XChaCha8::new(&key, &nonce);

        match self.to_token_stream(&mut encruster) {
            Ok(token_stream) => quote! {
                unsafe {
                    ::encrust_core::Encrusted::from_encrusted_data(
                        #token_stream,
                        ::chacha20::Key::from([#(#raw_key),*]),
                        ::chacha20::XNonce::from([#(#raw_nonce),*])
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

impl ToTokenStream for Literal {
    fn to_token_stream(
        &self,
        encruster: &mut XChaCha8,
    ) -> Result<proc_macro2::TokenStream, TokenStreamError> {
        Ok(match self {
            Self::U8(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::U16(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::U32(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::U64(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::U128(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::Usize(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::I8(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::I16(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::I32(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::I64(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::I128(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::Isize(n) => {
                let mut n = *n;
                unsafe {
                    n.toggle_encrust(encruster);
                }
                quote! {#n}
            }
            Self::String(s) => {
                let mut bytes = Vec::from(s.as_bytes());
                unsafe {
                    bytes.toggle_encrust(encruster);
                }
                quote! {unsafe { String::from_utf8_unchecked([#(#bytes),*].to_vec()) }}
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

impl ToTokenStream for LiteralVec {
    fn to_token_stream(
        &self,
        encruster: &mut XChaCha8,
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

impl ToTokenStream for StringFileReader {
    fn to_token_stream(
        &self,
        encruster: &mut XChaCha8,
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

impl ToTokenStream for BytesFileReader {
    fn to_token_stream(
        &self,
        encruster: &mut XChaCha8,
    ) -> Result<proc_macro2::TokenStream, TokenStreamError> {
        match std::fs::read(&self.0.path) {
            Ok(bytes) => Literal::Array(bytes.iter().map(|byte| Literal::U8(*byte)).collect())
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
