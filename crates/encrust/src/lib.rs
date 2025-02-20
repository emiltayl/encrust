#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(all(not(test), not(feature = "std")), no_std)]
#![doc = include_str!("../README.md")]

#[doc(inline)]
pub use encrust_core::*;
#[cfg_attr(feature = "macros", doc(inline))]
#[cfg(feature = "macros")]
pub use encrust_macros::*;
