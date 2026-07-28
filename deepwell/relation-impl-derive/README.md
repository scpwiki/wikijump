# deepwell-relation-impl-derive

This helper crate exists to store the procedural macro for generating methods and structures for relations (see `RelationService`). The [old `impl_relation!` macro](https://github.com/scpwiki/wikijump/blob/0a26522e50e0a2bdc0abddfe6338e6a882d24b74/deepwell/src/services/relation/macros.rs#L23) was defined via `macro_rules!`, but as it grew more complicated and took more options, it became increasingly cumbersome to maintain.

This proc macro performs the same tasks as the old macro, but with better syntax, more options, and avoiding automatable work (e.g. converting between `PascalCase` and `snake_case`).

See #3039 for further information.
