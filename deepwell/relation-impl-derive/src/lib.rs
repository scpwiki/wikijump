use proc_macro::TokenStream;

#[proc_macro]
pub fn impl_relation(stream: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}

// TODO
