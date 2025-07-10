mod from_file;

use proc_macro::TokenStream;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(FromFile)]
pub fn derive_from_file(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    from_file::impl_from_file(input)
}
