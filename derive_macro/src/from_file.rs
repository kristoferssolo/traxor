use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident};

pub fn impl_from_file(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let file_name = format!("{name}File");
    let file_ident = Ident::new(&file_name, name.span());

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_idents = fields.iter().map(|f| &f.ident);
    let field_idents2 = field_idents.clone();

    quote! {
        impl #name {
            fn from_file(file: Option<#file_ident>) -> Self 
            where
                #file_ident: Default + Clone
            {
                let default = #file_ident::default();
                let file = file.unwrap_or_else(|| default.clone());

                Self {
                    #(#field_idents: file.#field_idents2.unwrap_or_else(|| default.#field_idents.clone().unwrap())),*
                }
            }
        }

        impl From<Option<#file_ident>> for #name {
            fn from(value: Option<#file_ident>) -> Self {
                Self::from_file(value)
            }
        }
    }.into()
}
