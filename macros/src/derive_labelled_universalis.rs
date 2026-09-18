use std::iter::repeat;

use crate::helpers::{
    FieldBinding, FieldBindings, VariantBinding, VariantBindings, ref_generics, to_ast,
};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;
use syn::{Data, DataEnum, DataStruct, Ident, ImplGenerics, TypeGenerics, WhereClause};

/// Given an AST, returns an implementation of Universalis using `HList` with
/// Field (see `ordofp_core::labelled`) elements
///
/// Works with structs (named and tuple) and enums (lowered to a labelled
/// `Disiunctio` of per-variant `HLists`). Unions are not supported and emit
/// a compile error.
pub fn impl_labelled_universalis(input: TokenStream) -> impl ToTokens {
    let ast = to_ast(input);
    let generics = &ast.generics;
    let generics_ref = ref_generics(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (impl_generics_ref, _, where_clause_ref) = generics_ref.split_for_impl();
    let ctx = GenericsCtx {
        name: &ast.ident,
        impl_generics,
        ty_generics,
        where_clause,
        impl_generics_ref,
        where_clause_ref,
    };
    match ast.data {
        Data::Struct(ref data) => struct_arm(&ctx, data),
        Data::Enum(ref data) => enum_arm(&ctx, data),
        Data::Union(_) => syn::Error::new_spanned(
            &ast.ident,
            "Only structs and enums can be turned into labelled Universalis.",
        )
        .to_compile_error(),
    }
}

/// Shared generics prelude for the struct/enum codegen arms.
///
/// Grouping the `split_for_impl` fragments keeps each arm helper to two
/// parameters instead of threading six bindings through every signature.
struct GenericsCtx<'a> {
    name: &'a Ident,
    impl_generics: ImplGenerics<'a>,
    ty_generics: TypeGenerics<'a>,
    where_clause: Option<&'a WhereClause>,
    impl_generics_ref: ImplGenerics<'a>,
    where_clause_ref: Option<&'a WhereClause>,
}

/// Generates the `NominataUniversalis` implementation for structs.
fn struct_arm(ctx: &GenericsCtx<'_>, data: &DataStruct) -> TokenStream2 {
    let name = ctx.name;
    let impl_generics = &ctx.impl_generics;
    let ty_generics = &ctx.ty_generics;
    let where_clause = ctx.where_clause;
    let impl_generics_ref = &ctx.impl_generics_ref;
    let where_clause_ref = ctx.where_clause_ref;
    let field_bindings = FieldBindings::new(&data.fields);
    let repr_type = field_bindings.build_hlist_type(FieldBinding::build_field_type);
    let repr_type_ref = field_bindings.build_hlist_type(FieldBinding::build_field_type_ref);
    let repr_type_mut = field_bindings.build_hlist_type(FieldBinding::build_field_type_mut);
    let coniunctio_expr = field_bindings.build_hlist_constr(FieldBinding::build_field_expr);
    let coniunctio_pat = field_bindings.build_hlist_constr(FieldBinding::build_field_pat);
    let type_constr = field_bindings.build_type_constr(FieldBinding::build);
    let type_pat_ref = field_bindings.build_type_constr(FieldBinding::build_pat_ref);
    let type_pat_mut = field_bindings.build_type_constr(FieldBinding::build_pat_mut);

    quote! {
        impl #impl_generics ::ordofp_core::labelled::NominataUniversalis for #name #ty_generics #where_clause {

            type Repr = #repr_type;

            #[inline]
            fn into(self) -> Self::Repr {
                let #name #type_constr = self;
                #coniunctio_expr
            }

            #[inline]
            fn from(r: Self::Repr) -> Self {
                let #coniunctio_pat = r;
                #name #type_constr
            }
        }

                        impl #impl_generics_ref ::ordofp_core::labelled::IntoNominataUniversalis for & '_ordofp_ref_ #name #ty_generics #where_clause_ref {

            type Repr = #repr_type_ref;

            #[inline]
            fn into(self) -> Self::Repr {
                let #name #type_pat_ref = *self;
                #coniunctio_expr
            }

        }

                        impl #impl_generics_ref ::ordofp_core::labelled::IntoNominataUniversalis for & '_ordofp_ref_ mut #name #ty_generics #where_clause_ref {

            type Repr = #repr_type_mut;

            #[inline]
            fn into(self) -> Self::Repr {
                let #name #type_pat_mut = *self;
                #coniunctio_expr
            }

        }
    }
}

/// Generates the `NominataUniversalis` implementation for enums.
fn enum_arm(ctx: &GenericsCtx<'_>, data: &DataEnum) -> TokenStream2 {
    let name = ctx.name;
    let impl_generics = &ctx.impl_generics;
    let ty_generics = &ctx.ty_generics;
    let where_clause = ctx.where_clause;
    let impl_generics_ref = &ctx.impl_generics_ref;
    let where_clause_ref = ctx.where_clause_ref;
    let variant_bindings = VariantBindings::new(&data.variants);
    let repr_type = &variant_bindings.build_disiunctio_type(VariantBinding::build_hlist_field_type);
    let repr_type_ref =
        &variant_bindings.build_disiunctio_type(VariantBinding::build_hlist_field_type_ref);
    let repr_type_mut =
        &variant_bindings.build_disiunctio_type(VariantBinding::build_hlist_field_type_mut);
    let disiunctio_exprs =
        &variant_bindings.build_disiunctio_constrs(VariantBinding::build_hlist_field_expr);
    let disiunctio_pats =
        &variant_bindings.build_disiunctio_constrs(VariantBinding::build_hlist_field_pat);
    let disiunctio_unreachable = &variant_bindings.build_disiunctio_unreachable_arm(false);
    let type_constrs1 = &variant_bindings.build_variant_constrs(VariantBinding::build_type_constr);
    let type_constrs2 = type_constrs1;
    let type_pat_ref = &variant_bindings.build_variant_constrs(VariantBinding::build_type_pat_ref);
    let type_pat_mut = &variant_bindings.build_variant_constrs(VariantBinding::build_type_pat_mut);
    let name_it1 = repeat(name);
    let name_it2 = repeat(name);
    let name_it3 = repeat(name);
    let name_it4 = repeat(name);

    let base_impl = quote! {
                        impl #impl_generics ::ordofp_core::labelled::NominataUniversalis for #name #ty_generics #where_clause {

            type Repr = #repr_type;

            #[inline]
            fn into(self) -> Self::Repr {
                match self {
                    #(
                        #name_it1 :: #type_constrs1 => #disiunctio_exprs,
                    )*
                }
            }

            #[inline]
            fn from(r: Self::Repr) -> Self {
                match r {
                    #(
                        #disiunctio_pats => #name_it2 :: #type_constrs2,
                    )*
                    #disiunctio_unreachable
                }
            }
        }
    };

    let ref_impl = quote! {
                        impl #impl_generics_ref ::ordofp_core::labelled::IntoNominataUniversalis for & '_ordofp_ref_ #name #ty_generics #where_clause_ref {

            type Repr = #repr_type_ref;

            #[inline]
            fn into(self) -> Self::Repr {
                match self {
                    #(
                        #name_it3 :: #type_pat_ref => #disiunctio_exprs,
                    )*
                }
            }

        }
    };

    let mut_impl = quote! {
                        impl #impl_generics_ref ::ordofp_core::labelled::IntoNominataUniversalis for & '_ordofp_ref_ mut #name #ty_generics #where_clause_ref {

            type Repr = #repr_type_mut;

            #[inline]
            fn into(self) -> Self::Repr {
                match self {
                    #(
                        #name_it4 :: #type_pat_mut => #disiunctio_exprs,
                    )*
                }
            }

        }
    };

    quote! { #base_impl #ref_impl #mut_impl }
}
