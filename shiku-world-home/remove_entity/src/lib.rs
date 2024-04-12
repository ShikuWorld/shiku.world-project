use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Fields, Type};

#[proc_macro_derive(RemoveEntity)]
pub fn derive_remove_entity(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let remove_calls = match input.data {
        syn::Data::Struct(data_struct) => match data_struct.fields {
            Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|field| {
                    if let Type::Path(type_path) = &field.ty {
                        if type_path.path.segments.len() == 1
                            && type_path.path.segments[0].ident == "HashMap"
                        {
                            let field_name = &field.ident;
                            return Some(quote! {
                                self.#field_name.remove(&entity);
                            });
                        }
                    }
                    None
                })
                .collect::<Vec<_>>(),
            _ => panic!(
                "The RemoveEntity derive macro can only be applied to structs with named fields."
            ),
        },
        _ => panic!("The RemoveEntity derive macro can only be applied to structs."),
    };

    let expanded = quote! {
        impl #struct_name {
            pub fn remove_entity(&mut self, entity: Entity) {
                #(#remove_calls)*
            }
        }
    };

    TokenStream::from(expanded)
}
