[<< Return to the README](../README.md)

## JSON Serialization

All exposed fields are serializable with [`serde`](https://crates.io/crates/serde). If you use [`serde_json`](https://crates.io/crates/serde_json) to store syntax trees (as is used in `src/test.rs` and the `/test` directory), it is helpful to understand the basics of how these data types will be serialized. These principles will apply to other formats as well, but this section will focus on JSON.

The top level of a syntax tree contains two fields, `elements` and `styles`. The latter is simple, just a list of strings, each representing on CSS style within the wikitext. The first is of more interest, and more complex.

The Rust declaration of `Element` is as an enum, with each variant representing a different kind of element one may encounter. Most of these are leaf elements, such as `text` or `link`. Serde has been configured to use discriminated tagging, so the object representation will look like:

Note that the serialized form for _all_ data structures uses `kebab-case`.
For instance, `Token::LeftLink` is represented as `left-link`.

However you should also note that internal names for enums are, in typical Rust
tradition, in `AdaCase`. These should not be exposed during serialization however.

```json
{
    "element": "<type-of-element>",
    "data": { ... <whatever> }
}
```

Where `data` is adapted for each enum's value.

`Element::Text` and `Element::Raw` for instance only have a single string as their data, so it would just be a string object:

```json
{
    "element": "text",
    "data": "Apple"
}
```

Some elements have no associated data at all, such as `Element::LineBreak` or `Element::HorizontalRule`, and so would only have the element variant:

```json
{
    "element": "line-break"
}
```

Any element which contains other elements (e.g. bold, paragraph, divs) is called a "container", and are typically represented by the generic `Container` structure. These are similar to `Element` in that they are a discriminated enum, however their data fields are always the same (just `elements: Vec<Element<'t>>`).

For instance:

```json
{
    "element": "container",
    "data": {
        "type": "italics",
        "elements": [
            {
                "element": "text",
                "data": "Banana"
            },
            {
                ... <some other element>
            },
            {
                ... <yet another element>
            }
        ]
    }
}
```

This should hopefully help with understanding how these structures are represented, permitting library consumers not written in Rust to interpret the data.
For a full list of the fields of all elements, see the rustdoc. Particular files of interest are [`src/tree/element.rs`](https://github.com/Nu-SCPTheme/ftml/blob/master/src/tree/element.rs) and [`src/tree/container.rs`](https://github.com/Nu-SCPTheme/ftml/blob/master/src/tree/container.rs).
