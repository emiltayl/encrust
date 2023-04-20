use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    parse_quote, spanned::Spanned, Data, DeriveInput, Fields, GenericParam, Generics, Ident, Index,
    Variant,
};

pub fn derive_encrustable(input: DeriveInput) -> TokenStream {
    // Code copied from:
    // https://github.com/dtolnay/syn/blob/3da56a712abf7933b91954dbfb5708b452f88504/examples/heapsize/heapsize_derive/src/lib.rs
    // https://github.com/RustCrypto/utils/blob/72505ea620ee4d557a68372b6ba44a87f7d2ab1b/zeroize/derive/src/lib.rs

    let name = input.ident;
    let generics = add_trait_bounds(input.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let encrypatble_impl = gen_encrustable_impl(&input.data);

    quote! {
        #[doc(hidden)]
        impl #impl_generics ::encrust_core::Encrustable for #name #ty_generics #where_clause  {
            unsafe fn toggle_encrust(&mut self, encruster: &mut ::chacha20::XChaCha8) {
                #encrypatble_impl
            }
        }
    }
    .into()
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::encrust_core::Encrustable));
        }
    }
    generics
}

fn gen_encrustable_impl(data: &Data) -> proc_macro2::TokenStream {
    match data {
        Data::Struct(struct_data) => gen_struct_fields_calls(&struct_data.fields),
        Data::Enum(enum_data) => {
            let variants = enum_data.variants.iter().map(gen_variant_fields_calls);

            quote! {match self {
                #(#variants )*
            }}
        }

        Data::Union(_) => quote! { compile_error!("`Encrustable` does not support unions.");},
    }
}

fn gen_struct_fields_calls(fields: &Fields) -> proc_macro2::TokenStream {
    match fields {
        Fields::Named(named_fields) => {
            let field_calls = named_fields.named.iter().map(|field| {
                let name = &field.ident;

                quote_spanned! {field.span()=>
                    ::encrust_core::Encrustable::toggle_encrust(&mut self.#name, encruster);
                }
            });

            quote! {#(#field_calls) *}
        }

        Fields::Unnamed(numbered_fields) => {
            let field_calls = numbered_fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let index = Index::from(index);

                    quote_spanned! {field.span()=>
                        ::encrust_core::Encrustable::toggle_encrust(&mut self.#index, encruster);
                    }
                });

            quote! {#(#field_calls) *}
        }

        // Nothing to do because there is no data to encrust
        Fields::Unit => quote! {},
    }
}

fn gen_variant_fields_calls(variant: &Variant) -> proc_macro2::TokenStream {
    let variant_name = &variant.ident;
    match &variant.fields {
        Fields::Named(named_fields) => {
            let names = named_fields.named.iter().map(|field| &field.ident);
            let calls = named_fields.named.iter().map(|field| {
                let name = &field.ident;

                quote_spanned! {field.span()=>
                    ::encrust_core::Encrustable::toggle_encrust(#name, encruster);
                }
            });

            quote! {Self::#variant_name { #(#names),* } => {
                #(#calls);*
            }}
        }

        Fields::Unnamed(numbered_fields) => {
            let names = numbered_fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let name = format!("field_{index}");
                    Ident::new(&name, field.span())
                });
            let calls = numbered_fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(index, field)| {
                    let name = format!("field_{index}");
                    let ident = Ident::new(&name, field.span());

                    quote_spanned! {field.span()=>
                        ::encrust_core::Encrustable::toggle_encrust(#ident, encruster);
                    }
                });

            quote! {Self::#variant_name ( #(#names),* ) => {
                #(#calls);*
            }}
        }

        Fields::Unit => quote! {Self::#variant_name => {}},
    }
}
