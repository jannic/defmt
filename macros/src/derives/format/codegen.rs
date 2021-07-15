use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_quote, DataStruct, GenericParam, Ident, ImplGenerics, TypeGenerics, WhereClause};

pub(crate) use enum_data::encode as encode_enum_data;

mod enum_data;
mod fields;

pub(crate) struct EncodeData {
    pub(crate) format_tag: TokenStream2,
    pub(crate) stmts: Vec<TokenStream2>,
}

pub(crate) fn encode_struct_data(ident: &Ident, data: &DataStruct) -> EncodeData {
    let mut format_string = ident.to_string();
    let mut stmts = vec![];
    let mut patterns = vec![];

    let encode_field_stmts = fields::codegen(&data.fields, &mut format_string, &mut patterns);

    stmts.push(quote!(match self {
        Self { #(#patterns),* } => {
            #(#encode_field_stmts;)*
        }
    }));

    let format_tag = crate::mksym(&format_string, "derived", false);
    EncodeData { format_tag, stmts }
}

pub(crate) struct Generics<'a> {
    pub(crate) impl_generics: ImplGenerics<'a>,
    pub(crate) type_generics: TypeGenerics<'a>,
    pub(crate) where_clause: WhereClause,
}

impl<'a> Generics<'a> {
    pub(crate) fn codegen(generics: &'a mut syn::Generics) -> Self {
        let mut where_clause = generics.make_where_clause().clone();
        let (impl_generics, type_generics, _) = generics.split_for_impl();

        // Extend where-clause with `Format` bounds for type parameters.
        for param in &generics.params {
            if let GenericParam::Type(ty) = param {
                let ident = &ty.ident;

                where_clause
                    .predicates
                    .push(parse_quote!(#ident: defmt::Format));
            }
        }

        Self {
            impl_generics,
            type_generics,
            where_clause,
        }
    }
}
