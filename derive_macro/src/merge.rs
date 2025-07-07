use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type};

pub fn impl_merge_derive(input: DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => unimplemented!("Only named fields are supported for Merge derive macro"),
        },
        _ => unimplemented!("Only structs are supported for Merge derive macro"),
    };

    let merge_logic = fields.iter().map(|field| {
        let field_name = &field.ident;
        let field_type = &field.ty;

        // Check if the field is an Option<T>
        if let Type::Path(type_path) = field_type {
            if let Some(segment) = type_path.path.segments.last() {
                if segment.ident == "Option" {
                    // This is an Option<T> field
                    return quote! {
                        if let Some(o_val) = other.#field_name {
                            if let Some(s_val) = self.#field_name.as_mut() {
                                // If both are Some, attempt to merge recursively
                                s_val.merge(o_val);
                            } else {
                                // If self is None, take the other's Some value
                                self.#field_name = Some(o_val);
                            }
                        }
                        // If other is None, self remains unchanged
                    };
                }
            }
        }

        // For non-Option fields, attempt to merge recursively
        quote! {
            self.#field_name.merge(other.#field_name);
        }
    });

    let expanded = quote! {
        impl #impl_generics Merge for #name #ty_generics #where_clause {
            fn merge(&mut self, other: Self) {
                #(#merge_logic)*
            }
        }
    };

    expanded.into()
}

