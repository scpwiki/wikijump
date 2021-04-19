[<< Return to the README](../README.md)

## ftml-http Routes

The currently available API routes in the server are:

| Method | Route | Input | Output | Description |
|--------|-------|-------|--------|-------------|
| Any | `/ping` | None | `String` | See if you're able to connect to the server. |
| Any | `/version` | None | `String` | Outputs what version of ftml is being run. |
| `POST` | `/include` | `TextInput` | `Response<IncludeOutput>` | Substitutes all include blocks in the input string. |
| `POST` | `/preprocess` | `TextInput` | `Response<PreprocessOutput>` | Runs the preprocessor on the given input string. |
| `POST` | `/tokenize` | `TextInput` | `Response<TokenizeOutput>` | Runs the tokenizer on the input string and returns the extracted tokens. |
| `POST` | `/parse` | `TextInput` | `Response<ParseOutput>` | Runs the parser on the input string and returns the abstract syntax tree. |
| `POST` | `/render/html` | `RenderInput` | `Response<HtmlRenderOutput>` | Performs the full rendering process, from inclusion, preprocessing, tokenization, parsing, and then rendering. |
| `POST` | `/render/debug` | `RenderInput` | `Response<DebugRenderOutput>` | Performs rendering, as above, but uses `ftml::render::DebugRender`. |

Where the structures expected are the following:

**TextInput`** is the object describing a text input, and the specifications necessary to perform include substitution.

* `text` is the input wikitext to be processed.
* `callback-url` is the URL that ftml-http will POST to with an `IncludeRequest`, to get the pages to be included.
* `missing-include-template` is the template used to generate the "missing include" string if the `callback-url` does not return a result for a page. This allows jinja2-like syntax, backed by the crate [`tera`](https://crates.io/crates/tera). Three context variables are provided: `site` (nullable), `page`, `path`.

```json
{
    "text": "**My** //wikitext//!",
    "callback-url": "http://localhost:8000/includes",
    "missing-include-template": "Page '{{ page }}' is missing!"
}
```

**`IncludeRequest`** is the object requesting a foreign server return contents for each of these pages. It is just the field `includes` pointing to a list of `IncludeRef`s.

**`IncludeRef`** is the object describing one particular page to be included. It has two fields, `page-ref`, which specifies the page being included, and a map of all the variables to substitute.

Page references are composed of an optional site, then the page name. For instance `component:blah` would be on-site (`null`), and `:scp-wiki:main` would be off-site (site would be `scp-wiki`).

```json
{
    "page-ref": {
        "site": null,
        "page": "page-name"
    },
    "variables": {
        "each": "variable",
        "here!": ""
    }
}
```

**`IncludeResponse`** is the object expected from the foreign server returning contents of the fetched pages. It is a list of `FetchedPage` objects.

**`FetchedPage`** is the object describing one retrieved page. The first field, `page-ref`, describes which page it has content for. The second, `content`, has the data to be replaced, or null, if the page was not found.

The number of returned pages should exactly match the order and count of the requested pages. Each index between the request and the response must share the same `PageRef` in the same order.

```json
{
    "page-ref": {
        "site": null,
        "page": "theme:black-highlighter-theme"
    },
    "content": "[[module CSS]]\n...\n[[/module]]"
}
```

**`RenderInput`** is the object requesting a rendering of a page. It wraps objects we've seen above, plus `PageData`, which is found in `crate::data::page_info`.

```json
{
    "info": {
        "slug": "my-page",
        "category": "archived",
        "title": "Special page!",
        "alt-title": null,
        "header": null,
        "subheader": null,
        "rating": 100,
        "tags": ["archived", "project"],
        "locale": "en_US"
    },
    "contents": {
        "text": "**My** //wikitext//!",
        "callback-url": "http://localhost:8000/includes",
        "missing-include-template": "Page '{{ page }}' is missing!"
    }
}
```

**`Response`** is a wrapper to describe the state of an API call. It takes one of two forms:

Success:
```json
{
    "result": [ "data", "here" ]
}
```

Error:
```json
{
    "error": "Error message here"
}
```

This is a generic type, so what is inside depends on what is being wrapped. Errors will always be strings.

**`IncludeOutput`** is the object describing the result of a successful `/include` call.

The `text` fields represents the replaced wikitext. The `pages-included` is a list of `PageRef` instances, describing the pages that were included in the text.

```json
{
    "text": "Wikidot text following replacement",
    "pages-included": [
        {
            "site": null,
            "page": "some-page"
        }
    ],
}
```

**`PreprocessOutput`** is the object describing the result of a successful `/preprocess` call.

It is functionally the same as `IncludeOutput`, except also describes the preprocess step being applied after inclusion.

```json
{
    "text": "My //wikitext// here!",
    "pages-included": []
}
```

**`ParseOutput`** is the object describing the result of a successful `/parse` call.

It extends `PreprocessOutput`, with two added fields.

* `syntax_tree` is the JSON representation of the abstract syntax tree (AST) created by the parser, a recursively nested series of elements which describe its structure.
* `warnings` is a list of warning objects, describing parsing issues.

```json
{
    "text": "My //wikitext// here!",
    "pages-included": [],
    "syntax-tree": {
        "elements": [],
        "styles": []
    },
    "warnings": []
}
```

**`HtmlRenderOutput`** is the object describing the result of a successful `/render/html` call.

It extends `ParseOutput`, with three new fields.

* `html` is the generated HTML body, corresponding to the wikitext.
* `style` is the full collected stylesheet, as specified through CSS in the wikitext.
* `meta` is the list of HTML meta tags to add to the HTML document's `<head>`.

```json
{
    "text": "My //wikitext// here!",
    "pages-included": [],
    "syntax-tree": {
        "elements": [],
        "styles": []
    },
    "warnings": [],
    "html": "<strong>test</strong>",
    "style": "a { display: none }",
    "meta": []
}
```

**`DebugRenderOutput`** is the object describing the result of a successful `/render/html` call.

It extends `ParseOutput`, with one new fields.

* `output` is the string output of the `DebugRender` implementation.

```json
{
    "text": "My //wikitext// here!",
    "pages-included": [],
    "syntax-tree": {
        "elements": [],
        "styles": []
    },
    "warnings": [],
    "output": "< Debug! >"
}
```
