use std::ffi::CString;
use std::path::{Path, PathBuf};

use proc_macro2::Span;
use syn::parse::Parse;
use syn::{LitByteStr, LitCStr, LitInt, LitStr, Token, bracketed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ArrayElement {
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
    NestedArray(Vec<ArrayElement>),
}

impl ArrayElement {
    fn to_array_element_type(&self) -> ArrayElementType {
        match self {
            ArrayElement::U8(_) => ArrayElementType::U8,
            ArrayElement::U16(_) => ArrayElementType::U16,
            ArrayElement::U32(_) => ArrayElementType::U32,
            ArrayElement::U64(_) => ArrayElementType::U64,
            ArrayElement::U128(_) => ArrayElementType::U128,
            ArrayElement::Usize(_) => ArrayElementType::Usize,
            ArrayElement::I8(_) => ArrayElementType::I8,
            ArrayElement::I16(_) => ArrayElementType::I16,
            ArrayElement::I32(_) => ArrayElementType::I32,
            ArrayElement::I64(_) => ArrayElementType::I64,
            ArrayElement::I128(_) => ArrayElementType::I128,
            ArrayElement::Isize(_) => ArrayElementType::Isize,
            ArrayElement::NestedArray(arr) => ArrayElementType::Array {
                size: arr.len(),
                element: Box::new(
                    arr.first()
                        .map_or_else(|| unreachable!(), ArrayElement::to_array_element_type),
                ),
            },
        }
    }
}

/// The type signature of an array literal. Used for type checking arrays when parsing.
#[derive(Debug, PartialEq, Eq)]
enum ArrayElementType {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
    Array {
        size: usize,
        element: Box<ArrayElementType>,
    },
}

impl std::fmt::Display for ArrayElementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayElementType::U8 => write!(f, "u8"),
            ArrayElementType::U16 => write!(f, "u16"),
            ArrayElementType::U32 => write!(f, "u32"),
            ArrayElementType::U64 => write!(f, "u64"),
            ArrayElementType::U128 => write!(f, "u128"),
            ArrayElementType::Usize => write!(f, "usize"),
            ArrayElementType::I8 => write!(f, "i8"),
            ArrayElementType::I16 => write!(f, "i16"),
            ArrayElementType::I32 => write!(f, "i32"),
            ArrayElementType::I64 => write!(f, "i64"),
            ArrayElementType::I128 => write!(f, "i128"),
            ArrayElementType::Isize => write!(f, "isize"),
            ArrayElementType::Array { size, element } => write!(f, "[{element}; {size}]"),
        }
    }
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub enum LiteralKind {
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
    BString(Vec<u8>),
    CString(CString),
    Array(Vec<ArrayElement>),
}

#[cfg_attr(test, derive(Debug))]
pub struct LiteralNode {
    pub kind: LiteralKind,
    pub span: Span,
}

#[cfg(test)]
impl PartialEq for LiteralNode {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl LiteralKind {
    fn try_to_array_element(kind: LiteralKind, span: Span) -> syn::Result<ArrayElement> {
        match kind {
            LiteralKind::U8(v) => Ok(ArrayElement::U8(v)),
            LiteralKind::U16(v) => Ok(ArrayElement::U16(v)),
            LiteralKind::U32(v) => Ok(ArrayElement::U32(v)),
            LiteralKind::U64(v) => Ok(ArrayElement::U64(v)),
            LiteralKind::U128(v) => Ok(ArrayElement::U128(v)),
            LiteralKind::Usize(v) => Ok(ArrayElement::Usize(v)),
            LiteralKind::I8(v) => Ok(ArrayElement::I8(v)),
            LiteralKind::I16(v) => Ok(ArrayElement::I16(v)),
            LiteralKind::I32(v) => Ok(ArrayElement::I32(v)),
            LiteralKind::I64(v) => Ok(ArrayElement::I64(v)),
            LiteralKind::I128(v) => Ok(ArrayElement::I128(v)),
            LiteralKind::Isize(v) => Ok(ArrayElement::Isize(v)),
            LiteralKind::Array(arr) => Ok(ArrayElement::NestedArray(arr)),
            LiteralKind::String(_) | LiteralKind::BString(_) | LiteralKind::CString(_) => {
                Err(syn::Error::new(
                    span,
                    "`encrust!` only supports arrays with numeric literals or nested arrays.",
                ))
            }
        }
    }

    fn parse_array(input: syn::parse::ParseStream) -> syn::Result<(Vec<ArrayElement>, Span)> {
        let mut content = Vec::new();
        let buffer;
        let bracket = bracketed!(buffer in input);
        let span = bracket.span.join();

        while !buffer.is_empty() {
            let node = buffer.parse::<LiteralNode>()?;
            let element = Self::try_to_array_element(node.kind, node.span)?;
            content.push(element);

            if !buffer.is_empty() {
                buffer.parse::<Token![,]>()?;
            }
        }

        let first_elem = content.first().ok_or_else(|| {
            syn::Error::new(
                span,
                "Empty arrays are not supported by the `encrust!` macro.",
            )
        })?;

        let first_type = first_elem.to_array_element_type();

        // Check all elements have the same type/structure
        for elem in &content[1..] {
            let elem_type = elem.to_array_element_type();

            if first_type != elem_type {
                return Err(syn::Error::new(
                    span,
                    format!(
                        "`encrust!` macro expected an array of `{first_type}`, but found an element of type `{elem_type}`."
                    ),
                ));
            }
        }

        Ok((content, span))
    }
}

impl Parse for LiteralNode {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(LitInt) || input.peek(Token![-]) {
            let integer: LitInt = input.parse()?;

            Ok(LiteralNode {
                kind: match integer.suffix() {
                    "i8" => LiteralKind::I8(integer.base10_parse::<i8>()?),
                    "i16" => LiteralKind::I16(integer.base10_parse::<i16>()?),
                    "i32" => LiteralKind::I32(integer.base10_parse::<i32>()?),
                    "i64" => LiteralKind::I64(integer.base10_parse::<i64>()?),
                    "i128" => LiteralKind::I128(integer.base10_parse::<i128>()?),
                    "isize" => LiteralKind::Isize(integer.base10_parse::<isize>()?),
                    "u8" => LiteralKind::U8(integer.base10_parse::<u8>()?),
                    "u16" => LiteralKind::U16(integer.base10_parse::<u16>()?),
                    "u32" => LiteralKind::U32(integer.base10_parse::<u32>()?),
                    "u64" => LiteralKind::U64(integer.base10_parse::<u64>()?),
                    "u128" => LiteralKind::U128(integer.base10_parse::<u128>()?),
                    "usize" => LiteralKind::Usize(integer.base10_parse::<usize>()?),
                    "" => {
                        return Err(syn::Error::new(
                            integer.span(),
                            "No integer type suffix supplied in `encrust!` macro.",
                        ));
                    }
                    _ => {
                        return Err(syn::Error::new(
                            integer.span(),
                            format!(
                                "Supplied integer type `{}` is not supported by the `encrust!` macro.",
                                integer.suffix()
                            ),
                        ));
                    }
                },
                span: integer.span(),
            })
        } else if input.peek(LitStr) {
            let string: LitStr = input.parse()?;

            Ok(LiteralNode {
                kind: LiteralKind::String(string.value()),
                span: string.span(),
            })
        } else if input.peek(LitByteStr) {
            let bytes: LitByteStr = input.parse()?;

            Ok(LiteralNode {
                kind: LiteralKind::BString(bytes.value()),
                span: bytes.span(),
            })
        } else if input.peek(LitCStr) {
            let cstring: LitCStr = input.parse()?;

            Ok(LiteralNode {
                kind: LiteralKind::CString(cstring.value()),
                span: cstring.span(),
            })
        } else if input.peek(syn::token::Bracket) {
            let (content, span) = LiteralKind::parse_array(input)?;
            Ok(LiteralNode {
                kind: LiteralKind::Array(content),
                span,
            })
        } else {
            Err(syn::Error::new(
                input.span(),
                "Unsupported input to `encrust!`.",
            ))
        }
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
        let literal = syn::parse_str::<LiteralNode>("-1i8").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::I8(-1));
        let literal = syn::parse_str::<LiteralNode>("1u8").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::U8(1));

        let literal = syn::parse_str::<LiteralNode>("-1i16").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::I16(-1));
        let literal = syn::parse_str::<LiteralNode>("1u16").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::U16(1));

        let literal = syn::parse_str::<LiteralNode>("-1i32").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::I32(-1));
        let literal = syn::parse_str::<LiteralNode>("1u32").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::U32(1));

        let literal = syn::parse_str::<LiteralNode>("-1i64").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::I64(-1));
        let literal = syn::parse_str::<LiteralNode>("1u64").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::U64(1));

        let literal = syn::parse_str::<LiteralNode>("-1i128").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::I128(-1));
        let literal = syn::parse_str::<LiteralNode>("1u128").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::U128(1));

        let literal = syn::parse_str::<LiteralNode>("-1isize").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::Isize(-1));
        let literal = syn::parse_str::<LiteralNode>("1usize").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::Usize(1));
    }

    #[test]
    fn parse_number_fail_on_no_type() {
        let literal = syn::parse_str::<LiteralNode>("-1");
        assert!(literal.is_err());
    }

    #[test]
    fn parse_number_fail_on_unsupported_type() {
        let literal = syn::parse_str::<LiteralNode>("1f32");
        assert!(literal.is_err());

        let literal = syn::parse_str::<LiteralNode>("1f64");
        assert!(literal.is_err());

        let literal = syn::parse_str::<LiteralNode>("1u");
        assert!(literal.is_err());
    }

    #[test]
    fn parse_numbers_fail_on_outside_range() {
        let literal = syn::parse_str::<LiteralNode>("-1usize");
        assert!(literal.is_err());

        let literal = syn::parse_str::<LiteralNode>("128i8");
        assert!(literal.is_err());
    }

    #[test]
    fn parse_string_literal() {
        let literal =
            syn::parse_str::<LiteralNode>("\"The quick brown fox jumps over the lazy dog😊\"")
                .expect("Unable to parse literal");
        assert_eq!(
            literal.kind,
            LiteralKind::String("The quick brown fox jumps over the lazy dog😊".to_owned())
        );
    }

    #[test]
    fn parse_empty_string_literals() {
        let literal = syn::parse_str::<LiteralNode>("\"\"").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::String(String::new()));

        let literal = syn::parse_str::<LiteralNode>("b\"\"").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::BString(Vec::new()));

        let literal = syn::parse_str::<LiteralNode>("c\"\"").expect("Unable to parse literal");
        assert_eq!(literal.kind, LiteralKind::CString(c"".to_owned()));
    }

    #[test]
    fn parse_bstring_literal() {
        let literal =
            syn::parse_str::<LiteralNode>("b\"The quick brown fox jumps over the lazy dog\"")
                .expect("Unable to parse literal");
        assert_eq!(
            literal.kind,
            LiteralKind::BString(b"The quick brown fox jumps over the lazy dog".to_vec())
        );
    }

    #[test]
    fn parse_cstring_literal() {
        let literal =
            syn::parse_str::<LiteralNode>("c\"The quick brown fox jumps over the lazy dog😊\"")
                .expect("Unable to parse literal");
        assert_eq!(
            literal.kind,
            LiteralKind::CString(c"The quick brown fox jumps over the lazy dog😊".to_owned())
        );
    }

    #[test]
    fn parse_array() {
        let literal =
            syn::parse_str::<LiteralNode>("[1u8,2u8,3u8]").expect("Unable to parse literal");
        assert_eq!(
            literal.kind,
            LiteralKind::Array(vec![
                ArrayElement::U8(1u8),
                ArrayElement::U8(2u8),
                ArrayElement::U8(3u8)
            ])
        );
    }

    #[test]
    fn parse_nested_array() {
        let literal = syn::parse_str::<LiteralNode>("[[1u8,2u8,3u8],[3u8,2u8,1u8]]")
            .expect("Unable to parse literal");
        assert_eq!(
            literal.kind,
            LiteralKind::Array(vec![
                ArrayElement::NestedArray(vec![
                    ArrayElement::U8(1u8),
                    ArrayElement::U8(2u8),
                    ArrayElement::U8(3u8)
                ]),
                ArrayElement::NestedArray(vec![
                    ArrayElement::U8(3u8),
                    ArrayElement::U8(2u8),
                    ArrayElement::U8(1u8)
                ])
            ])
        );
    }

    #[test]
    fn parse_array_validation_failures() {
        syn::parse_str::<LiteralNode>("[1u8, 2u16]")
            .expect_err("Parser did not reject heterogeneous array.");

        syn::parse_str::<LiteralNode>(r#"["string"]"#)
            .expect_err("Parser did not reject array with a string.");

        syn::parse_str::<LiteralNode>("[1u8, [2u8, 3u8]]")
            .expect_err("Parser did not reject heterogeneous array.");

        syn::parse_str::<LiteralNode>("[[1u8, 2u8], [3u8]]")
            .expect_err("Parser did not reject heterogeneous array.");

        syn::parse_str::<LiteralNode>("[[1u8, 2u8], [3u16, 4u16]]")
            .expect_err("Parser did not reject heterogeneous array.");

        syn::parse_str::<LiteralNode>("[]").expect_err("Parser did not reject empty array.");

        syn::parse_str::<LiteralNode>("[[1u8], []]")
            .expect_err("Parser did not reject nested empty arrays.");

        syn::parse_str::<LiteralNode>("[[[], []], [[], []], [[], []]]")
            .expect_err("Parser did not reject nested empty arrays.");

        syn::parse_str::<LiteralNode>("[1u8, 2u8,]")
            .expect("Parser did not accept an array with a trailing comma.");
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
            ToHashString("The quick brown fox jumps over the lazy dog😊".to_owned()),
            string
        );

        let empty = syn::parse_str::<ToHashString>("\"\"").expect("Unable to parse literal");
        assert_eq!(ToHashString("".to_owned()), empty);
    }

    #[test]
    fn parse_tohashbytes() {
        let bytes = syn::parse_str::<ToHashBytes>("[0x01, 2, 3u8, 0b0, 255]")
            .expect("Unable to parse literal");
        assert_eq!(ToHashBytes(vec![1, 2, 3, 0, 255]), bytes);

        let empty = syn::parse_str::<ToHashBytes>("[]").expect("Unable to parse literal");
        assert_eq!(ToHashBytes(Vec::new()), empty);
    }

    #[test]
    fn tohashbytes_fails_on_invalid_input() {
        let too_large = syn::parse_str::<ToHashBytes>("[0, 256, 0]");
        assert!(too_large.is_err());

        let negative = syn::parse_str::<ToHashBytes>("[-1, 2, 3]");
        assert!(negative.is_err());

        let string = syn::parse_str::<ToHashBytes>("[\"not bytes\"]");
        assert!(string.is_err());

        let nested = syn::parse_str::<ToHashBytes>("[[1], 2, 3]");
        assert!(nested.is_err());
    }
}
