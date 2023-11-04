use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

#[proc_macro_error]
#[proc_macro_attribute]
pub fn test(_: TokenStream, _: TokenStream) -> TokenStream {}
