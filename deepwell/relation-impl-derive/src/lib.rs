mod case;
mod expand;
mod parse;
mod types;
mod util;

use self::expand::expand_stream;
use self::parse::RelationSettings;
use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn impl_relation(stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(stream as RelationSettings);
    let generated = expand_stream(input);
    generated
}
