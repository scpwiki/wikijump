mod case;
mod expand;
mod parse;
mod types;
mod util;

#[cfg(test)]
mod test;

use self::expand::expand_stream;
use self::parse::RelationSettings;
use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro]
pub fn impl_relation(stream: TokenStream) -> TokenStream {
    let settings = parse_macro_input!(stream as RelationSettings);
    expand_stream(settings).into()
}
