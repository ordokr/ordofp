#![recursion_limit = "128"]
#![doc(html_playground_url = "https://play.rust-lang.org/")]
//! `OrdoFP` Macros
//!
//! The custom derives (`Universalis`, `NominataUniversalis`) and procedural
//! macros (`path!`, `path_type!`) for `OrdoFP`.
//!
//! Links:
//!   1. [Source on Github](https://github.com/ordokr/ordofp)
//!   2. [Crates.io page](https://crates.io/crates/ordofp)

extern crate proc_macro;

#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{Expr, parse_macro_input};

mod helpers;

mod derive_universalis;
use crate::derive_universalis::impl_universalis;

mod derive_labelled_universalis;
use crate::derive_labelled_universalis::impl_labelled_universalis;

/// Derives a Universalis instance based on `HList` for
/// a given Struct or Tuple Struct
#[proc_macro_derive(Universalis)]
pub fn derive_universalis(input: TokenStream) -> TokenStream {
    // Build the impl
    let generated = impl_universalis(input);
    // Return the generated impl
    generated.into_token_stream().into()
}

/// Derives a Universalis instance based on Field + `HList` for
/// a given Struct (Tuple Structs not supported because they have
/// no labels)
///
/// There *may* be problems if your field names contain certain characters.
/// This can be solved by adding letters to the `create_char_types`! macro invocation
/// in `ordofp_core::labelled` via a PR :)
#[proc_macro_derive(NominataUniversalis)]
pub fn derive_nominata_universalis(input: TokenStream) -> TokenStream {
    // Build the impl
    let generated = impl_labelled_universalis(input);
    // Return the generated impl
    generated.into_token_stream().into()
}

/// Build a Universalis path that can be used for traversals
#[proc_macro]
pub fn path(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    let path_type = match helpers::build_path_type(expr) {
        Ok(t) => t,
        Err(e) => return e.to_compile_error().into(),
    };
    let ast = quote! {
        {
            let p: #path_type = ::ordofp_core::path::Path::new();
            p
        }
    };
    TokenStream::from(ast)
}

/// Build the type of a Universalis path, for use in type position
/// (where [`path!`](macro@path) builds the value).
#[proc_macro]
pub fn path_type(input: TokenStream) -> TokenStream {
    let expr = parse_macro_input!(input as Expr);
    let path_type = match helpers::build_path_type(expr) {
        Ok(t) => t,
        Err(e) => return e.to_compile_error().into(),
    };
    let ast = quote! {
        #path_type
    };
    TokenStream::from(ast)
}
