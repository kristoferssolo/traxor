use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, DeriveInput, Error, Type, parse_quote};

pub fn impl_unit_conversions(input: DeriveInput) -> TokenStream {
    let name = &input.ident;

    let mut unsigned_types = Vec::new();
    let mut signed_types = Vec::new();
    let mut error_type: Option<Type> = None;

    for attr in &input.attrs {
        if attr.path().is_ident("units") {
            if let Ok(types) = parse_unit_types(attr) {
                for ty in types {
                    let type_str = quote!(#ty).to_string();
                    if is_signed_type(&type_str) {
                        signed_types.push(ty);
                    } else {
                        unsigned_types.push(ty);
                    }
                }
            } else if attr.path().is_ident("error") {
                if let Ok(err_type) = parse_error_types(attr) {
                    error_type = Some(err_type);
                }
            }
        }
    }

    if unsigned_types.is_empty() && signed_types.is_empty() {
        unsigned_types = vec![
            parse_quote!(u8),
            parse_quote!(u16),
            parse_quote!(u32),
            parse_quote!(u64),
            parse_quote!(usize),
        ];
        signed_types = vec![
            parse_quote!(i8),
            parse_quote!(i16),
            parse_quote!(i32),
            parse_quote!(i64),
            parse_quote!(isize),
        ];
    }

    let error_type = error_type.unwrap_or_else(|| parse_quote!(String));
    let is_string_error = quote!(#error_type).to_string() == "String";

    let from_impls = unsigned_types.iter().map(|ty| {
        let conversion_expr = if name == "Unit" {
            quote! { Self(value as u64) }
        } else {
            quote! { Self(crate::app::utils::unit::Unit::new(value as u64)) }
        };
        quote! {
            impl From<#ty> for #name {
                fn from(value: #ty) -> Self {
                    #conversion_expr
                }
            }
        }
    });

    let try_from_impls = signed_types.iter().map(|ty| {
        let error_creation = if is_string_error {
            quote! {
                format!("Cannot convert negative value {} to {}", value, stringify!(#name))
            }
        } else {
            // For custom error types, try to construct from a string message
            // This assumes the error type implements From<String> or similar
            quote! {
                #error_type::from(format!("Cannot convert negative value {} to {}", value, stringify!(#name)))
            }
        };

        let conversion_expr = if name == "Unit" {
            quote! { Ok(Self(value as u64)) }
        } else {
            quote! { Ok(Self(crate::app::utils::unit::Unit::try_from(value)?)) }
        };

        quote! {
            impl TryFrom<#ty> for #name {
                type Error = #error_type;

                fn try_from(value: #ty) -> Result<Self, Self::Error> {
                    if value < 0 {
                        return Err(#error_creation);
                    }
                    #conversion_expr
                }
            }
        }
    });

    let expanded = quote! {
        #(#from_impls)*
        #(#try_from_impls)*
    };

    TokenStream::from(expanded)
}

fn parse_unit_types(attr: &Attribute) -> Result<Vec<Type>, Error> {
    let mut types = Vec::new();

    attr.parse_nested_meta(|meta| {
        if let Ok(ty) = meta.value()?.parse::<Type>() {
            types.push(ty);
        }
        Ok(())
    })?;
    Ok(types)
}

fn parse_error_types(attr: &Attribute) -> Result<Type, Error> {
    let mut error_type = None;

    attr.parse_nested_meta(|meta| {
        if let Ok(ty) = meta.value()?.parse::<Type>() {
            error_type = Some(ty);
        }
        Ok(())
    })?;

    error_type.ok_or_else(|| Error::new_spanned(attr, "Expected error type"))
}

fn is_signed_type(type_str: &str) -> bool {
    matches!(type_str, "i8" | "i16" | "i32" | "i64" | "isize")
}
