use proc_macro::TokenStream;

mod regex;

#[proc_macro]
pub fn re(input: TokenStream) -> TokenStream {
    regex::re_impl(input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}
