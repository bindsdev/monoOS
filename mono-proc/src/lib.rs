use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::ItemFn;

/// Attribute that tests are marked with.
#[proc_macro_error]
#[proc_macro_attribute]
pub fn test(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = attr.to_string();
    let quiet = if attr.contains("quiet") { true } else { false };

    let input = syn::parse_macro_input!(item as ItemFn);
    let name = &input.sig.ident;
    let body = &input.block;

    let test_marker = quote::format_ident!("{name}_test_marker");
    let result = quote::quote! {
        #[test_case]
        static #test_marker: crate::tests::Test = crate::tests::Test {
            path: concat!(module_path!(), "::", stringify!(#name)),
            func: #name,
            quiet: #quiet
        };

        fn #name() {
            #body
        }
    };

    result.into()
}
