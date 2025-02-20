use std::path::{Path, PathBuf};

use proc_macro2::Span;
use syn::{LitInt, LitStr, Token, bracketed, parse::Parse};

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum Literal {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    Usize(usize),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Isize(isize),
    String(String),
    Array(Vec<Literal>),
}

impl Parse for Literal {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitInt) || input.peek(Token![-]) {
            let integer: LitInt = input.parse()?;

            Ok(match integer.suffix() {
                "i8" => Self::I8(integer.base10_parse::<i8>()?),
                "i16" => Self::I16(integer.base10_parse::<i16>()?),
                "i32" => Self::I32(integer.base10_parse::<i32>()?),
                "i64" => Self::I64(integer.base10_parse::<i64>()?),
                "i128" => Self::I128(integer.base10_parse::<i128>()?),
                "isize" => Self::Isize(integer.base10_parse::<isize>()?),
                "u8" => Self::U8(integer.base10_parse::<u8>()?),
                "u16" => Self::U16(integer.base10_parse::<u16>()?),
                "u32" => Self::U32(integer.base10_parse::<u32>()?),
                "u64" => Self::U64(integer.base10_parse::<u64>()?),
                "u128" => Self::U128(integer.base10_parse::<u128>()?),
                "usize" => Self::Usize(integer.base10_parse::<usize>()?),
                "" => {
                    return Err(syn::Error::new(
                        integer.span(),
                        "No integer data type suffix supplied.",
                    ));
                }
                _ => {
                    return Err(syn::Error::new(
                        integer.span(),
                        format!(
                            "Supplied integer type `{}` not supported by `encrust_integer`.",
                            integer.suffix()
                        ),
                    ));
                }
            })
        } else if input.peek(LitStr) {
            let string: LitStr = input.parse()?;

            Ok(Self::String(string.value()))
        } else if input.peek(syn::token::Bracket) {
            let mut content = Vec::new();
            let buffer;
            bracketed!(buffer in input);

            while !buffer.is_empty() {
                content.push(buffer.parse()?);

                if !buffer.is_empty() {
                    buffer.parse::<Token![,]>()?;
                }
            }

            Ok(Self::Array(content))
        } else {
            Err(syn::Error::new(
                input.span(),
                "Unsupported input to `encrust`.",
            ))
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct LiteralVec(pub Vec<Literal>);

impl Parse for LiteralVec {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut vec = Vec::new();

        while !input.is_empty() {
            vec.push(input.parse()?);

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self(vec))
    }
}

pub struct FilePath {
    pub path: PathBuf,
    pub span: Span,
}

impl Parse for FilePath {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path_lit: LitStr = input.parse()?;
        let path_str = path_lit.value();
        let input_path = Path::new(path_str.as_str());

        let path = if input_path.is_absolute() {
            input_path.into()
        } else {
            Path::new(std::env!("CARGO_MANIFEST_DIR")).join(input_path)
        };

        Ok(Self {
            path,
            span: path_lit.span(),
        })
    }
}

#[cfg(feature = "hashstrings")]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ToHashString(pub String);

#[cfg(feature = "hashstrings")]
impl Parse for ToHashString {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_str: LitStr = input.parse()?;

        Ok(Self(lit_str.value()))
    }
}

#[cfg(feature = "hashstrings")]
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct ToHashBytes(pub Vec<u8>);

#[cfg(feature = "hashstrings")]
impl Parse for ToHashBytes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut bytes: Vec<u8> = Vec::new();
        let buffer;
        bracketed!(buffer in input);

        while !buffer.is_empty() {
            let lit: LitInt = buffer.parse()?;
            bytes.push(lit.base10_parse()?);

            if !buffer.is_empty() {
                buffer.parse::<Token![,]>()?;
            }
        }

        Ok(Self(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_numbers() {
        let literal = syn::parse_str::<Literal>("-1i8").expect("Unable to parse literal");
        assert_eq!(Literal::I8(-1), literal);
        let literal = syn::parse_str::<Literal>("1u8").expect("Unable to parse literal");
        assert_eq!(Literal::U8(1), literal);

        let literal = syn::parse_str::<Literal>("-1i16").expect("Unable to parse literal");
        assert_eq!(Literal::I16(-1), literal);
        let literal = syn::parse_str::<Literal>("1u16").expect("Unable to parse literal");
        assert_eq!(Literal::U16(1), literal);

        let literal = syn::parse_str::<Literal>("-1i32").expect("Unable to parse literal");
        assert_eq!(Literal::I32(-1), literal);
        let literal = syn::parse_str::<Literal>("1u32").expect("Unable to parse literal");
        assert_eq!(Literal::U32(1), literal);

        let literal = syn::parse_str::<Literal>("-1i64").expect("Unable to parse literal");
        assert_eq!(Literal::I64(-1), literal);
        let literal = syn::parse_str::<Literal>("1u64").expect("Unable to parse literal");
        assert_eq!(Literal::U64(1), literal);

        let literal = syn::parse_str::<Literal>("-1i128").expect("Unable to parse literal");
        assert_eq!(Literal::I128(-1), literal);
        let literal = syn::parse_str::<Literal>("1u128").expect("Unable to parse literal");
        assert_eq!(Literal::U128(1), literal);

        let literal = syn::parse_str::<Literal>("-1isize").expect("Unable to parse literal");
        assert_eq!(Literal::Isize(-1), literal);
        let literal = syn::parse_str::<Literal>("1usize").expect("Unable to parse literal");
        assert_eq!(Literal::Usize(1), literal);
    }

    #[test]
    fn parse_number_fail_on_no_type() {
        let literal = syn::parse_str::<Literal>("-1");
        assert!(literal.is_err());
    }

    #[test]
    fn parse_numbers_fail_on_outside_range() {
        let literal = syn::parse_str::<Literal>("-1usize");
        assert!(literal.is_err());

        let literal = syn::parse_str::<Literal>("128i8");
        assert!(literal.is_err());
    }

    #[test]
    fn parse_string_literal() {
        let literal =
            syn::parse_str::<Literal>("\"The quick brown fox jumps over the lazy dog😊\"")
                .expect("Unable to parse literal");
        assert_eq!(
            Literal::String("The quick brown fox jumps over the lazy dog😊".to_string()),
            literal
        );
    }

    #[test]
    fn parse_array() {
        let literal = syn::parse_str::<Literal>("[1u8,2u8,3u8]").expect("Unable to parse literal");
        assert_eq!(
            Literal::Array(vec![Literal::U8(1u8), Literal::U8(2u8), Literal::U8(3u8)]),
            literal
        );
    }

    #[test]
    fn parse_vec() {
        let literal = syn::parse_str::<LiteralVec>("1u8,2u8,3u8").expect("Unable to parse literal");
        assert_eq!(
            LiteralVec(vec![Literal::U8(1u8), Literal::U8(2u8), Literal::U8(3u8)]),
            literal
        );
    }

    #[test]
    fn parse_paths() {
        let path = syn::parse_str::<FilePath>("\"//absolute/path\"")
            .expect("Unable to parse path literal");
        assert_eq!(Path::new("//absolute/path"), path.path);

        let rel_path =
            syn::parse_str::<FilePath>("\"relative/path\"").expect("Unable to parse path literal");
        assert_eq!(
            Path::new(std::env!("CARGO_MANIFEST_DIR")).join("relative/path"),
            rel_path.path
        );
    }

    #[test]
    fn parse_tohashstring() {
        let string =
            syn::parse_str::<ToHashString>("\"The quick brown fox jumps over the lazy dog😊\"")
                .expect("Unable to parse literal");
        assert_eq!(
            ToHashString("The quick brown fox jumps over the lazy dog😊".to_string()),
            string
        );
    }

    #[test]
    fn parse_tohashbytes() {
        let bytes =
            syn::parse_str::<ToHashBytes>("[0x01, 2, 3u8, 0b0]").expect("Unable to parse literal");
        assert_eq!(ToHashBytes(vec![1, 2, 3, 0]), bytes);
    }

    #[test]
    fn tohashbytes_fails_when_numbers_cannot_fit_u8() {
        let too_large = syn::parse_str::<ToHashBytes>("[0, 256, 0]");
        assert!(too_large.is_err());

        let negative = syn::parse_str::<ToHashBytes>("[-1, 2, 3]");
        assert!(negative.is_err());
    }
}
