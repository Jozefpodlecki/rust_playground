
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Ident, Meta, Variant};

#[proc_macro_derive(GenerateTraits, attributes(no_data, with_struct))]
pub fn connect_packet_to_structs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = &input.ident;
    let data_enum = if let Data::Enum(data_enum) = &input.data {
        data_enum
    } else {
        panic!("This macro only works for enums");
    };

    let mut structs = vec![];

    for variant in &data_enum.variants {
        let variant_name = &variant.ident;
        let mut has_no_data = false;
        let mut struct_name = None;

        for attr in &variant.attrs {
            let attr_path = attr.path();
            let has_attribute = attr_path.is_ident("no_data");

            if has_attribute {
                has_no_data = true;
            }

            let has_struct = attr_path.is_ident("with_struct");

            if has_struct {
                let iden: Ident = attr.parse_args().unwrap();
                struct_name = Some(iden);
            }
        }


        let trait_name = syn::Ident::new(&format!("{}Handler", variant_name), variant_name.span());
        let generated_trait: proc_macro2::TokenStream;

        if has_no_data {
            generated_trait = quote! {
                pub trait #trait_name {
                    fn process(&self);
                }
            };
        }
        else {
            if let Some(struct_name) = struct_name {
                generated_trait = quote! {
                    pub trait #trait_name {
                        fn process(&self, data: #struct_name);
                    }
                };
            }
            else {
                generated_trait = quote! {
                    pub trait #trait_name {
                        fn process(&self, data: Vec<u8>);
                    }
                };
            }
        }

        structs.push(generated_trait);
    }

    let generated = quote! {
        #(#structs)*
    };

    TokenStream::from(generated)
}