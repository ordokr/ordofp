use std::iter::repeat;

use crate::helpers::{
    FieldBinding, FieldBindings, VariantBinding, VariantBindings, ref_generics, to_ast,
};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::Data;

/// Given an AST, returns an implementation of Universalis using `HList` with
/// Field (see `ordofp_core::labelled`) elements
///
/// Works with structs (named and tuple) and enums (lowered to a labelled
/// `Disiunctio` of per-variant `HLists`). Unions are not supported and emit
/// a compile error.
// Line count is dominated by the three quote! templates; splitting them out
// would just thread a dozen bindings through helper signatures.
#[allow(clippy::too_many_lines)]
pub(crate) fn impl_labelled_universalis(input: TokenStream) -> impl ToTokens {
    let ast = to_ast(input);
    let name = &ast.ident;

    let generics = &ast.generics;
    let generics_ref = ref_generics(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let (impl_generics_ref, _, where_clause_ref) = generics_ref.split_for_impl();

    match ast.data {
        Data::Struct(ref data) => {
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

                    #[inline(always)]
                    fn into(self) -> Self::Repr {
                        let #name #type_constr = self;
                        #coniunctio_expr
                    }

                    #[inline(always)]
                    fn from(r: Self::Repr) -> Self {
                        let #coniunctio_pat = r;
                        #name #type_constr
                    }
                }

                                impl #impl_generics_ref ::ordofp_core::labelled::IntoNominataUniversalis for & '_ordofp_ref_ #name #ty_generics #where_clause_ref {

                    type Repr = #repr_type_ref;

                    #[inline(always)]
                    fn into(self) -> Self::Repr {
                        let #name #type_pat_ref = *self;
                        #coniunctio_expr
                    }

                }

                                impl #impl_generics_ref ::ordofp_core::labelled::IntoNominataUniversalis for & '_ordofp_ref_ mut #name #ty_generics #where_clause_ref {

                    type Repr = #repr_type_mut;

                    #[inline(always)]
                    fn into(self) -> Self::Repr {
                        let #name #type_pat_mut = *self;
                        #coniunctio_expr
                    }

                }
            }
        }
        Data::Enum(ref data) => {
            let variant_bindings = VariantBindings::new(&data.variants);
            let repr_type =
                &variant_bindings.build_disiunctio_type(VariantBinding::build_hlist_field_type);
            let repr_type_ref =
                &variant_bindings.build_disiunctio_type(VariantBinding::build_hlist_field_type_ref);
            let repr_type_mut =
                &variant_bindings.build_disiunctio_type(VariantBinding::build_hlist_field_type_mut);
            let disiunctio_exprs =
                &variant_bindings.build_disiunctio_constrs(VariantBinding::build_hlist_field_expr);
            let disiunctio_pats =
                &variant_bindings.build_disiunctio_constrs(VariantBinding::build_hlist_field_pat);
            let disiunctio_unreachable = &variant_bindings.build_disiunctio_unreachable_arm(false);
            let type_constrs1 =
                &variant_bindings.build_variant_constrs(VariantBinding::build_type_constr);
            let type_constrs2 = type_constrs1;
            let type_pat_ref =
                &variant_bindings.build_variant_constrs(VariantBinding::build_type_pat_ref);
            let type_pat_mut =
                &variant_bindings.build_variant_constrs(VariantBinding::build_type_pat_mut);
            let name_it1 = repeat(name);
            let name_it2 = repeat(name);
            let name_it3 = repeat(name);
            let name_it4 = repeat(name);

            let base_impl = quote! {
                                impl #impl_generics ::ordofp_core::labelled::NominataUniversalis for #name #ty_generics #where_clause {

                    type Repr = #repr_type;

                    #[inline(always)]
                    fn into(self) -> Self::Repr {
                        match self {
                            #(
                                #name_it1 :: #type_constrs1 => #disiunctio_exprs,
                            )*
                        }
                    }

                    #[inline(always)]
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

                    #[inline(always)]
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

                    #[inline(always)]
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
        Data::Union(_) => syn::Error::new_spanned(
            &ast.ident,
            "Only structs and enums can be turned into labelled Universalis.",
        )
        .to_compile_error(),
    }
}
