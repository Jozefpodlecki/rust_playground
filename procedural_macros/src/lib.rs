use proc_macro::TokenStream;
use quote::quote;
use quote::format_ident;
use syn::{Data, DeriveInput};

mod derive_all_variants;

/// Implements methods for provided enum `all_variants()` which returns all enum variants
#[proc_macro_derive(AllVariants)]
pub fn derive_all_variants(input: TokenStream) -> TokenStream {
    derive_all_variants::derive_all_variants_inner(input)
}

#[proc_macro_derive(Service)]
pub fn derive_service_trait(input: TokenStream) -> TokenStream {
    let syn_item: DeriveInput = syn::parse(input).unwrap();

    let struct_name = &syn_item.ident;
    let fields = match syn_item.data {
        Data::Struct(ref data_struct) => &data_struct.fields,
        _ => panic!("It can only be used on structs"),
    };
    let service_name = format_ident!("{}Service", struct_name);
    let struct_name_str = struct_name.to_string();

    let expanded = quote! {

        pub trait #service_name {
            fn name(&self) -> &'static str;
        }

        impl #service_name for #struct_name {
            fn name(&self) -> &'static str {
                #struct_name_str
            }
        }
    };

    expanded.into()
}