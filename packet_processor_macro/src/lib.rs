
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Ident, Variant, Field};

#[proc_macro]
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

        let struct_name = variant_name.clone();
        let generated_struct = quote! {
            struct #struct_name;
        };

        structs.push(generated_struct);
    }

    let generated = quote! {
        #(#structs)*
    };

    TokenStream::from(generated)
}