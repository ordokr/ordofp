//! `OrdoFP` Proc Macro internals
//!
//! Shared internals for the derive and procedural macros.
//!
//! Links:
//!   1. [Source on Github](https://github.com/ordokr/ordofp)
//!   2. [Crates.io page](https://crates.io/crates/ordofp)

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{
    DeriveInput, Expr, Field, Fields, GenericParam, Generics, Ident, Lifetime, LifetimeParam,
    Member, Variant,
};

/// Lowercase letters - mapped to L-prefixed CamelCase names (La, Lb, Lc, ...)
const LOWERCASE_CHARS: &[char] = &[
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z',
];

/// Uppercase letters - mapped to U-prefixed CamelCase names (Ua, Ub, Uc, ...)
const UPPERCASE_CHARS: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
];

/// Digit characters - mapped to N-prefixed CamelCase names (N0, N1, ..., N9)
const DIGIT_CHARS: &[char] = &['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];

/// Parses a `TokenStream` (usually received as input into a
/// custom derive function), into a syn `MacroInput` AST,
/// which is nice.
pub(crate) fn to_ast(input: TokenStream) -> DeriveInput {
    // Parse the string representation
    syn::parse(input).unwrap()
}

/// Returns an Ident
pub(crate) fn call_site_ident(s: &str) -> Ident {
    Ident::new(s, Span::call_site())
}

/// Fold-right over `items`, wrapping an accumulator that starts at `base` —
/// the shared spine-building shape behind the HList/Disiunctio type and
/// constructor builders (rightmost item ends up innermost).
fn build_nested<L: IntoIterator>(
    items: L,
    base: TokenStream2,
    wrap: impl Fn(L::Item, TokenStream2) -> TokenStream2,
) -> TokenStream2
where
    L::IntoIter: DoubleEndedIterator,
{
    items
        .into_iter()
        .rev()
        .fold(base, |acc, item| wrap(item, acc))
}

/// Given a list of types, creates an AST for the corresponding `HList`
/// type.
pub(crate) fn build_hlist_type<L: IntoIterator>(items: L) -> TokenStream2
where
    L::Item: ToTokens,
    L::IntoIter: DoubleEndedIterator,
{
    build_nested(
        items,
        quote! { ::ordofp_core::hlist::Nihil },
        |item, acc| {
            quote! { ::ordofp_core::hlist::Coniunctio<#item, #acc> }
        },
    )
}

/// Given a list of expressions or patterns, creates an AST for the corresponding `HList`
/// constructor, which may itself be used as an expression or a pattern.
pub(crate) fn build_hlist_constr<L: IntoIterator>(items: L) -> TokenStream2
where
    L::Item: ToTokens,
    L::IntoIter: DoubleEndedIterator,
{
    build_nested(
        items,
        quote! { ::ordofp_core::hlist::Nihil },
        |item, acc| {
            quote! { ::ordofp_core::hlist::Coniunctio { head: #item, tail: #acc }}
        },
    )
}

/// Given a list of types, creates an AST for the corresponding Disiunctio
/// type.
pub(crate) fn build_disiunctio_type<L: IntoIterator>(items: L) -> TokenStream2
where
    L::Item: ToTokens,
    L::IntoIter: DoubleEndedIterator,
{
    build_nested(
        items,
        quote! { ::ordofp_core::disiunctio::Absurdum },
        |item, acc| quote! { ::ordofp_core::disiunctio::Disiunctio<#item, #acc> },
    )
}

/// Given an index and an expression or pattern, creates an AST for the corresponding Disiunctio
/// constructor, which may itself be used as an expression or a pattern.
pub(crate) fn build_disiunctio_constr(index: usize, item: impl ToTokens) -> TokenStream2 {
    (0..index).fold(
        quote! { ::ordofp_core::disiunctio::Disiunctio::Sinister(#item) },
        |acc, _| quote! { ::ordofp_core::disiunctio::Disiunctio::Dexter(#acc) },
    )
}

/// Given the length of a Disiunctio type, generates an "unreachable" match arm, matching
/// the Absurdum case in order to work around limitations in the compiler's exhaustiveness
/// checking.
pub(crate) fn build_disiunctio_unreachable_arm(length: usize, _deref: bool) -> TokenStream2 {
    let result = (0..length).fold(quote! { _ordofp_unreachable_ }, |acc, _| {
        quote! { ::ordofp_core::disiunctio::Disiunctio::Dexter(#acc)}
    });
    quote! { #result => unreachable!() }
}

/// Build the type-level representation of a labelled `Field`.
///
/// Given an identifier `name` and a type `inner_type`, produces the token
/// stream for `::ordofp_core::labelled::Field<L, T>` where `L` is the
/// type-level label derived from `name` (see [`build_label_type`]).
///
/// Used by derive macros to generate the `HList` field type for each
/// named struct field.
pub(crate) fn build_field_type(name: &Ident, inner_type: impl ToTokens) -> TokenStream2 {
    let label_type = build_label_type(name);
    quote! { ::ordofp_core::labelled::Field<#label_type, #inner_type> }
}
/// Build an expression constructing a labelled `Field` value.
///
/// Given an identifier `name` and a value expression `inner_expr`, produces
/// the token stream for
/// `::ordofp_core::labelled::field_with_name::<L, _>(name_str, inner_expr)`
/// where `L` is the type-level label derived from `name` and `name_str` is
/// its string representation.
///
/// Used by derive macros to generate the `HList` construction expression for
/// each named struct field.
pub(crate) fn build_field_expr(name: &Ident, inner_expr: impl ToTokens) -> TokenStream2 {
    let label_type = build_label_type(name);
    let literal_name = name.to_string();
    quote! { ::ordofp_core::labelled::field_with_name::<#label_type, _>(#literal_name, #inner_expr) }
}

/// Build a pattern that destructures a labelled `Field` value.
///
/// Produces the token stream for `::ordofp_core::labelled::Field { value: inner_pat, .. }`,
/// binding the inner value to `inner_pat` while ignoring the label.
///
/// Used by derive macros to generate the `HList` destructuring pattern for
/// each named struct field.
pub(crate) fn build_field_pat(inner_pat: impl ToTokens) -> TokenStream2 {
    quote! { ::ordofp_core::labelled::Field { value: #inner_pat, .. } }
}

/// Given an Ident returns an AST for its type level representation based on the
/// enums generated in `ordofp_core::labelled`.
///
/// For example, given `first_name`, returns an AST for (f,i,r,s,t,__,n,a,m,e)
pub(crate) fn build_label_type(ident: &Ident) -> impl ToTokens {
    let as_string = ident.to_string();
    let name = as_string.as_str();
    // Map each encoded char-ident straight into its token stream; no intermediate
    // `Vec<Ident>` buffer, which was previously collected then iterated once.
    let name_as_tokens: Vec<_> = name
        .chars()
        .flat_map(encode_as_ident)
        .map(|ident| quote! { ::ordofp_core::labelled::chars::#ident })
        .collect();
    quote! { (#(#name_as_tokens),*) }
}

/// Given a char, encodes it as a vector of Ident
///
/// Takes care of checking to see whether the char can be used as is,
/// or needs to be encoded with proper CamelCase naming.
///
/// Character encoding in `ordofp_core::labelled::chars`:
/// - Lowercase letters a-z -> La, Lb, ..., Lz
/// - Uppercase letters A-Z -> Ua, Ub, ..., Uz
/// - Digits 0-9 -> N0, N1, ..., N9
/// - Underscore _ -> Underscore
/// - Unicode -> `UnderscoreUc` + hex encoding + `UcUnderscore`
fn encode_as_ident(c: char) -> Vec<Ident> {
    if LOWERCASE_CHARS.contains(&c) {
        // Lowercase letters: 'a' -> "La", 'b' -> "Lb", etc.
        vec![call_site_ident(&format!("L{c}"))]
    } else if UPPERCASE_CHARS.contains(&c) {
        // Uppercase letters: 'A' -> "Ua", 'B' -> "Ub", etc.
        vec![call_site_ident(&format!("U{}", c.to_ascii_lowercase()))]
    } else if DIGIT_CHARS.contains(&c) {
        // Digits: '0' -> "N0", '1' -> "N1", etc.
        vec![call_site_ident(&format!("N{c}"))]
    } else if c == '_' {
        // Underscore
        vec![call_site_ident("Underscore")]
    } else {
        // UTF escape and get the hexcode
        let as_unicode = c.escape_unicode();
        // as_unicode can be multiple unicode codepoints encoded as \u{2764}\u{fe0f} (❤️)
        // so we filter on alphanumeric to get just u's as a delimiters along with the
        // hex portions
        let delimited_hex = as_unicode.filter(|c| c.is_alphanumeric());
        let mut hex_idents: Vec<Ident> = delimited_hex.flat_map(encode_as_ident).collect();
        // sandwich between UnderscoreUc and UcUnderscore
        let mut book_ended: Vec<Ident> = vec![call_site_ident("UnderscoreUc")];
        book_ended.append(&mut hex_idents);
        book_ended.push(call_site_ident("UcUnderscore"));
        book_ended
    }
}

/// Build a type-level representation of a path expression.
///
/// Converts a path expression (e.g. `foo.bar.baz`) into the corresponding
/// nested `Path<Coniunctio<…, Nihil>>` type token stream used by the
/// `ordofp_core` path system.  Each identifier in the expression becomes a
/// label type, and the labels are consed in reverse order so that the
/// leftmost segment ends up outermost.
///
/// # Errors
///
/// Propagates any `syn::Error` from [`find_idents_in_expr`] if `path_expr`
/// is not a valid field access chain.
pub(crate) fn build_path_type(path_expr: Expr) -> syn::Result<TokenStream2> {
    let idents = find_idents_in_expr(path_expr)?;
    Ok(idents
        .iter()
        .map(build_label_type)
        .fold(quote!(::ordofp_core::hlist::Nihil), |acc, t| {
            quote! {
            ::ordofp_core::path::Path<
                ::ordofp_core::hlist::Coniunctio<
                   #t,
                   #acc
                >
              >
            }
        }))
}

/// Returns the idents in a path like expression in reverse.
///
/// # Errors
///
/// This function returns a spanned `syn::Error` (rendered as a compile error
/// by the calling proc macro) if:
/// - Tuple field access is used (e.g., `foo.0`) - only named fields are supported
/// - The path contains `::` separators (e.g., `module::name`)
/// - The expression is not a valid field access chain
pub(crate) fn find_idents_in_expr(path_expr: Expr) -> syn::Result<Vec<Ident>> {
    fn go(current: Expr, mut v: Vec<Ident>) -> syn::Result<Vec<Ident>> {
        match current {
            Expr::Field(e) => {
                let m = e.member;
                match m {
                    Member::Named(i) => {
                        v.push(i);
                    }
                    Member::Unnamed(idx) => {
                        return Err(syn::Error::new_spanned(
                            &idx,
                            format!(
                                "Tuple field access (`.{}`) is not supported in path expressions. \
                                 Use named struct fields instead.",
                                idx.index
                            ),
                        ));
                    }
                }
                go(*e.base, v)
            }
            Expr::Path(p) => {
                if p.path.segments.len() == 1 {
                    let i = p.path.segments[0].ident.clone();
                    v.push(i);
                    Ok(v)
                } else {
                    let msg = format!(
                        "Path `{}` contains `::` separators. \
                         Only simple field access chains are supported (e.g., `foo.bar.baz`).",
                        p.path.to_token_stream()
                    );
                    Err(syn::Error::new_spanned(&p.path, msg))
                }
            }
            _ => Err(syn::Error::new_spanned(
                &current,
                "Invalid path expression. Expected a field access chain like `foo.bar.baz`, \
                 but found an unsupported expression type.",
            )),
        }
    }
    go(path_expr, Vec::new())
}

pub(crate) enum StructType {
    Named,
    Tuple,
    Unit,
}

pub(crate) struct FieldBinding {
    pub field: Field,
    pub binding: Ident,
}

impl FieldBinding {
    /// Returns a `TokenStream2` that represents the owned type of this field.
    ///
    /// This is used in proc-macro code generation to emit the field's type
    /// in contexts that require an owned value (e.g. struct field declarations
    /// or `HList` type parameters).
    ///
    /// # Example output
    ///
    /// For a field declared as `foo: String`, this emits `String`.
    pub(crate) fn build_type(&self) -> TokenStream2 {
        let ty = &self.field.ty;
        quote! { #ty }
    }
    /// Returns a `TokenStream2` for a shared reference to this field's type (`&'_ T`).
    pub(crate) fn build_type_ref(&self) -> TokenStream2 {
        let ty = &self.field.ty;
        quote! { &'_ordofp_ref_ #ty }
    }
    /// Returns a `TokenStream2` for a mutable reference to this field's type (`&'_ mut T`).
    pub(crate) fn build_type_mut(&self) -> TokenStream2 {
        let ty = &self.field.ty;
        quote! { &'_ordofp_ref_ mut #ty }
    }
    /// Returns a `TokenStream2` that emits the binding identifier for this field.
    pub(crate) fn build(&self) -> TokenStream2 {
        let binding = &self.binding;
        quote! { #binding }
    }
    /// Returns a `TokenStream2` for a `ref` pattern binding (`ref ident`).
    pub(crate) fn build_pat_ref(&self) -> TokenStream2 {
        let binding = &self.binding;
        quote! { ref #binding }
    }
    /// Returns a `TokenStream2` for a mutable `ref` pattern binding (`ref mut ident`).
    pub(crate) fn build_pat_mut(&self) -> TokenStream2 {
        let binding = &self.binding;
        quote! { ref mut #binding }
    }
    /// Returns a `TokenStream2` for a struct field declaration with the owned type (`ident: T`).
    pub(crate) fn build_field_type(&self) -> TokenStream2 {
        build_field_type(&self.binding, self.build_type())
    }
    /// Returns a `TokenStream2` for a struct field declaration with a shared reference type (`ident: &'_ T`).
    pub(crate) fn build_field_type_ref(&self) -> TokenStream2 {
        build_field_type(&self.binding, self.build_type_ref())
    }
    /// Returns a `TokenStream2` for a struct field declaration with a mutable reference type (`ident: &'_ mut T`).
    pub(crate) fn build_field_type_mut(&self) -> TokenStream2 {
        build_field_type(&self.binding, self.build_type_mut())
    }
    /// Returns a `TokenStream2` for a struct field expression (`ident: binding`).
    pub(crate) fn build_field_expr(&self) -> TokenStream2 {
        build_field_expr(&self.binding, &self.binding)
    }
    /// Returns a `TokenStream2` for a struct field destructuring pattern (`ident: binding`).
    pub(crate) fn build_field_pat(&self) -> TokenStream2 {
        build_field_pat(&self.binding)
    }
}

/// Represents the binding of a struct or enum variant's fields to a corresponding
/// set of similarly named local variables.
pub(crate) struct FieldBindings {
    pub type_: StructType,
    pub fields: Vec<FieldBinding>,
}

impl FieldBindings {
    /// Construct a [`FieldBindings`] from a syn [`Fields`] node.
    ///
    /// Inspects each field of the struct or variant to determine the struct
    /// kind (`Named`, `Tuple`, or `Unit`) and creates a [`FieldBinding`] for
    /// every field. Named fields keep their original identifier; tuple fields
    /// receive a generated identifier of the form `_0`, `_1`, etc.
    pub(crate) fn new(fields: &Fields) -> Self {
        Self {
            type_: match fields {
                Fields::Named(_) => StructType::Named,
                Fields::Unnamed(_) => StructType::Tuple,
                Fields::Unit => StructType::Unit,
            },
            fields: fields
                .iter()
                .enumerate()
                .map(|(index, field)| FieldBinding {
                    field: field.clone(),
                    binding: field
                        .ident
                        .clone()
                        .unwrap_or_else(|| Ident::new(&format!("_{index}"), field.span())),
                })
                .collect(),
        }
    }

    /// Builds a type constructor for use with structs or enum variants. Does not include the name
    /// of the type or variant.
    pub(crate) fn build_type_constr<R: ToTokens>(
        &self,
        f: impl Fn(&FieldBinding) -> R,
    ) -> TokenStream2 {
        let bindings: Vec<_> = self.fields.iter().map(f).collect();
        match self.type_ {
            StructType::Named => quote! { { #(#bindings,)* } },
            StructType::Tuple => quote! { ( #(#bindings,)* ) },
            StructType::Unit => TokenStream2::new(),
        }
    }

    /// Build the `HList` type token stream for this field set.
    ///
    /// Applies `f` to each [`FieldBinding`] to produce a token for its type,
    /// then delegates to the free [`build_hlist_type`] function to wrap those
    /// tokens in the `Coniunctio<_, Nihil>` spine.
    pub(crate) fn build_hlist_type<R: ToTokens>(
        &self,
        f: impl Fn(&FieldBinding) -> R,
    ) -> TokenStream2 {
        build_hlist_type(self.fields.iter().map(f))
    }

    /// Build the `HList` constructor (or pattern) token stream for this field set.
    ///
    /// Applies `f` to each [`FieldBinding`] to produce a token for the field
    /// value or binding, then delegates to the free [`build_hlist_constr`]
    /// function to wrap those tokens in the `Coniunctio { head, tail }` spine.
    /// The resulting token stream is valid as both an expression and a pattern.
    pub(crate) fn build_hlist_constr<R: ToTokens>(
        &self,
        f: impl Fn(&FieldBinding) -> R,
    ) -> TokenStream2 {
        build_hlist_constr(self.fields.iter().map(f))
    }
}

/// Augment a set of generics for use in reference (`&T` / `&mut T`) impls.
///
/// Introduces a fresh `'_ordofp_ref_` lifetime, constrains every existing
/// lifetime parameter to outlive it, and appends it to the parameter list.
/// The resulting `Generics` is suitable for implementing traits on `&'_ordofp_ref_ Type<…>`
/// and `&'_ordofp_ref_ mut Type<…>` inside derive macros.
pub(crate) fn ref_generics(generics: &Generics) -> Generics {
    let mut generics_ref = generics.clone();

    // instantiate a lifetime and lifetime def to add
    let ref_lifetime = Lifetime::new("'_ordofp_ref_", Span::call_site());
    let ref_lifetime_def = LifetimeParam::new(ref_lifetime.clone());

    // Constrain existing lifetimes in the concrete type to the reference lifetime
    // of our implementation of NominataUniversalis for the reference case (& and &mut)
    {
        let generics_ref_lifetimes_mut = generics_ref.lifetimes_mut();
        for existing_lifetime_mut in generics_ref_lifetimes_mut {
            existing_lifetime_mut.bounds.push(ref_lifetime.clone());
        }
    }

    // Add our current reference lifetime to the generic parameter list.
    let ref_lifetime_param = GenericParam::Lifetime(ref_lifetime_def);
    generics_ref.params.push(ref_lifetime_param);

    generics_ref
}

pub(crate) struct VariantBinding {
    pub name: Ident,
    pub fields: FieldBindings,
}

impl VariantBinding {
    /// Build the type constructor token stream for this enum variant, including
    /// the variant name.
    ///
    /// Produces `VariantName { field, … }` for named-field variants,
    /// `VariantName(field, …)` for tuple variants, or `VariantName` for unit
    /// variants. Unlike [`FieldBindings::build_type_constr`], which omits the
    /// leading name, this method prepends `self.name` so the result is a
    /// complete constructor or pattern fragment suitable for use in `match`
    /// arms or expression positions.
    pub(crate) fn build_type_constr(&self) -> TokenStream2 {
        let name = &self.name;
        let constr = self.fields.build_type_constr(FieldBinding::build);
        quote! { #name #constr }
    }
    /// Build pattern for matching on a reference.
    /// In Rust 2024, we don't use `ref` patterns when match ergonomics apply.
    pub(crate) fn build_type_pat_ref(&self) -> TokenStream2 {
        let name = &self.name;
        // Use regular bindings - match ergonomics handles the reference binding
        let constr = self.fields.build_type_constr(FieldBinding::build);
        quote! { #name #constr }
    }
    /// Build pattern for matching on a mutable reference.
    /// In Rust 2024, we don't use `ref mut` patterns when match ergonomics apply.
    pub(crate) fn build_type_pat_mut(&self) -> TokenStream2 {
        let name = &self.name;
        // Use regular bindings - match ergonomics handles the reference binding
        let constr = self.fields.build_type_constr(FieldBinding::build);
        quote! { #name #constr }
    }
    /// Build the `Field`-wrapped `HList` type token stream for this variant's owned fields.
    ///
    /// Produces a token stream for
    /// `::ordofp_core::labelled::Field<L, HCons<F1, HCons<F2, HNil>>>`
    /// where `L` is the type-level label derived from the variant name and
    /// `F1, F2, …` are the owned types of the variant's fields.
    ///
    /// Used by derive macros to generate the per-variant `HList` type in an
    /// enum's `Generic` representation.
    pub(crate) fn build_hlist_field_type(&self) -> TokenStream2 {
        build_field_type(
            &self.name,
            self.fields.build_hlist_type(FieldBinding::build_field_type),
        )
    }
    /// Build the `Field`-wrapped `HList` type token stream for this variant's fields as shared references.
    ///
    /// Like [`build_hlist_field_type`](Self::build_hlist_field_type), but uses `&F` instead of `F`
    /// for each field type, producing
    /// `::ordofp_core::labelled::Field<L, HCons<&F1, HCons<&F2, HNil>>>`.
    ///
    /// Used by derive macros to generate the per-variant `HList` type in a `&Self` generic
    /// representation.
    pub(crate) fn build_hlist_field_type_ref(&self) -> TokenStream2 {
        build_field_type(
            &self.name,
            self.fields
                .build_hlist_type(FieldBinding::build_field_type_ref),
        )
    }
    /// Build the `Field`-wrapped `HList` type token stream for this variant's fields as mutable references.
    ///
    /// Like [`build_hlist_field_type_ref`](Self::build_hlist_field_type_ref), but uses `&mut F`
    /// instead of `&F` for each field type, producing
    /// `::ordofp_core::labelled::Field<L, HCons<&mut F1, HCons<&mut F2, HNil>>>`.
    ///
    /// Used by derive macros to generate the per-variant `HList` type in a `&mut Self` generic
    /// representation.
    pub(crate) fn build_hlist_field_type_mut(&self) -> TokenStream2 {
        build_field_type(
            &self.name,
            self.fields
                .build_hlist_type(FieldBinding::build_field_type_mut),
        )
    }
    /// Build the `Field`-wrapped `HList` constructor expression for this variant's fields.
    ///
    /// Produces a token stream for
    /// `::ordofp_core::labelled::field_with_name::<L, _>(label, HCons(v1, HCons(v2, HNil)))`,
    /// where `L` is the type-level label derived from the variant name and `v1, v2, …` are
    /// the per-field value expressions.
    ///
    /// Used by derive macros to generate the `HList` construction expression when converting
    /// an enum variant into its `Generic` representation.
    pub(crate) fn build_hlist_field_expr(&self) -> TokenStream2 {
        build_field_expr(
            &self.name,
            self.fields
                .build_hlist_constr(FieldBinding::build_field_expr),
        )
    }
    /// Build the `Field`-wrapped `HList` destructuring pattern for this variant's fields.
    ///
    /// Produces a token stream for
    /// `::ordofp_core::labelled::Field { value: HCons { head: p1, tail: … }, .. }`,
    /// binding each field to its corresponding pattern variable `p1, p2, …`.
    ///
    /// Used by derive macros to generate the `HList` destructuring pattern when converting
    /// an enum's `Generic` representation back into a concrete variant.
    pub(crate) fn build_hlist_field_pat(&self) -> TokenStream2 {
        build_field_pat(
            self.fields
                .build_hlist_constr(FieldBinding::build_field_pat),
        )
    }
}

pub(crate) struct VariantBindings {
    pub variants: Vec<VariantBinding>,
}

impl VariantBindings {
    /// Collect all enum variants into a [`VariantBindings`] helper.
    ///
    /// Iterates over `data`, converting each `syn::Variant` into a
    /// [`VariantBinding`] (name + field bindings) and stores them in
    /// declaration order.  The resulting [`VariantBindings`] is the starting
    /// point for all derive-macro code generation involving sum types.
    ///
    /// # Parameters
    ///
    /// * `data` – An iterator of references to parsed `syn::Variant` nodes,
    ///   typically obtained from `syn::DataEnum::variants`.
    pub(crate) fn new<'a>(data: impl IntoIterator<Item = &'a Variant>) -> Self {
        VariantBindings {
            variants: data
                .into_iter()
                .map(|variant| VariantBinding {
                    name: variant.ident.clone(),
                    fields: FieldBindings::new(&variant.fields),
                })
                .collect(),
        }
    }

    /// Builds a nested `Disiunctio<_, Absurdum>` type token stream for all variants.
    ///
    /// Applies `f` to each [`VariantBinding`] to produce a per-variant token
    /// fragment (e.g. a field type), then delegates to the free
    /// [`build_disiunctio_type`] function to wrap them in the canonical
    /// right-nested `Disiunctio<A, Disiunctio<B, … Absurdum>>` structure used
    /// by the derive macros.
    pub(crate) fn build_disiunctio_type<R: ToTokens>(
        &self,
        f: impl Fn(&VariantBinding) -> R,
    ) -> TokenStream2 {
        build_disiunctio_type(self.variants.iter().map(f))
    }

    /// Generates one `Disiunctio` constructor token stream per enum variant.
    ///
    /// For each variant (with its zero-based index), `f` is called to produce the
    /// payload tokens, and the result is wrapped in the appropriate number of
    /// `Disiunctio::Dexter` layers followed by a `Disiunctio::Sinister` — matching
    /// the positional encoding used by [`build_disiunctio_constr`].
    ///
    /// Returns a `Vec` whose length equals the number of variants, in declaration order.
    pub(crate) fn build_disiunctio_constrs<R: ToTokens>(
        &self,
        f: impl Fn(&VariantBinding) -> R,
    ) -> Vec<TokenStream2> {
        self.variants
            .iter()
            .enumerate()
            .map(|(index, variant)| build_disiunctio_constr(index, f(variant)))
            .collect()
    }

    /// Applies `f` to each [`VariantBinding`] and collects the results into a `Vec`.
    ///
    /// Unlike `build_disiunctio_constrs`, this does not wrap each result in
    /// `Disiunctio` constructor layers — the caller receives the raw `R` values,
    /// one per variant in declaration order.
    pub(crate) fn build_variant_constrs<R: ToTokens>(
        &self,
        f: impl Fn(&VariantBinding) -> R,
    ) -> Vec<R> {
        self.variants.iter().map(f).collect()
    }

    /// Generates the catch-all unreachable arm for a `Disiunctio` match.
    ///
    /// Returns a token stream for the wildcard arm that covers the impossible
    /// `Absurdum` branch in exhaustive matches over `Disiunctio` values.
    /// When `deref` is `true`, the pattern dereferences the scrutinee.
    pub(crate) fn build_disiunctio_unreachable_arm(&self, deref: bool) -> TokenStream2 {
        build_disiunctio_unreachable_arm(self.variants.len(), deref)
    }
}
