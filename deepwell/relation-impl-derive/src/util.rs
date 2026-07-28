//! Miscellaneous utilities.

use proc_macro2::Span;
use std::fmt::Display;
use syn::{Ident, Type};

/// Convert `Type` to `Option<Type>` (`None` is if the type is `()`)).
pub fn process_type(t_type: Type) -> Option<Type> {
    match t_type {
        Type::Paren(inner_type) => process_type(*inner_type.elem),
        Type::Tuple(inner_type) if inner_type.elems.is_empty() => None,
        // leave as-is
        _ => Some(t_type),
    }
}

#[inline]
pub fn make_ident(value: impl AsRef<str>) -> Ident {
    Ident::new(value.as_ref(), Span::call_site())
}

#[inline]
pub fn make_error(message: impl Display) -> syn::Error {
    syn::Error::new(Span::call_site(), message)
}
