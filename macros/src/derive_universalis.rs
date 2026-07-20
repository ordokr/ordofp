use crate::helpers::{FieldBinding, FieldBindings, to_ast};
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::Data;

/// Given an AST, returns an implementation of Universalis using `HList`
///
/// Only works with Structs and Tuple Structs
pub(crate) fn impl_universalis(input: TokenStream) -> impl ToTokens {
    let ast = to_ast(input);
    let name = &ast.ident;

    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    match ast.data {
        Data::Struct(ref data) => {
            let field_bindings = FieldBindings::new(&data.fields);
            let repr_type = field_bindings.build_hlist_type(FieldBinding::build_type);
            let coniunctio_constr = field_bindings.build_hlist_constr(FieldBinding::build);
            let type_constr = field_bindings.build_type_constr(FieldBinding::build);

            quote! {
                                impl #impl_generics ::ordofp_core::universalis::Universalis for #name #ty_generics #where_clause {

                    type Repr = #repr_type;

                    #[inline]
                    fn into(self) -> Self::Repr {
                        let #name #type_constr = self;
                        #coniunctio_constr
                    }

                    #[inline]
                    fn from(r: Self::Repr) -> Self {
                        let #coniunctio_constr = r;
                        #name #type_constr
                    }
                }
            }
        }
        _ => syn::Error::new_spanned(
            &ast.ident,
            "Only structs are supported. Enums/unions cannot be turned into Universalis.",
        )
        .to_compile_error(),
    }
}
