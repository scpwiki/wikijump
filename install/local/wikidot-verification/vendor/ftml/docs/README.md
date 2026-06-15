## ftml

<p>
  <a href="https://github.com/scpwiki/ftml/actions/workflows/build.yaml">
    <img src="https://github.com/scpwiki/ftml/actions/workflows/build.yaml/badge.svg"
         alt="Build status">
  </a>

  <a href="https://github.com/scpwiki/ftml/actions/workflows/config.yaml">
    <img src="https://github.com/scpwiki/ftml/actions/workflows/config.yaml/badge.svg"
         alt="Configuration">
  </a>

  <a href="https://docs.rs/ftml">
    <img src="https://docs.rs/ftml/badge.svg"
         alt="docs.rs link">
  </a>
</p>

### Foundation Text Markup Language

(Alternatively, ftml: the markup language)

A Rust library to parse Wikidot text ("Wikitext") into an abstract syntax tree (AST).
This aims to be a replacement for the aging [Text\_Wiki](https://github.com/gabrys/wikidot/tree/master/lib/Text_Wiki/Text) from Wikidot.
This is version aims to have a nearly fully compatible parser for common Wikidot, including common malformed constructs.
The goal is to utilize a lexer generator, and consume the tokens in a custom parser to handle unusual cases with a lax approach.

In addition to providing the speed and safety benefits of Rust, this also improves maintainability, and allows exposing an AST to consumers
for more advanced analysis and transformation.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have `unsafe` internals.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE.md). This library was originally part of [Wikijump](https://github.com/scpwiki/wikijump/) at `/ftml`, before being moved to an independent repository, per [WJ-1219](https://scuttle.atlassian.net/browse/WJ-1219). Issues for this project will remain in the `WJ` Jira project.

### Compilation

This library targets the latest stable Rust. At time of writing, that is `1.92.0`.

```sh
$ cargo build --release
```

You can use this as a dependency by adding the following to your `Cargo.toml`:

```toml
ftml = "1"
```

The library has two features:
* `html` (enabled by default) &mdash; This includes the HTML renderer in the crate.
* `mathml` (enabled by default) &mdash; This includes `latex2mathml`, which is used to compile any LaTeX into MathML for inclusion in rendered HTML.

They can be disabled by building without features:

```
$ cargo check --no-default-features
```

If you wish to build the WebAssembly target for ftml, use `wasm-pack`:

```
$ RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build -- --no-default-features
```

This optimizes the final WASM, which can take some time. If you are developing and are only interested in the build passing, you should instead use:

```
$ RUSTFLAGS='--cfg getrandom_backend="wasm_js"' wasm-pack build --dev
```

### Testing

```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output. You can additionally inspect logging by exposing a `log`-compatible logger.

One test function of note is the "AST test" suite, which reads test descriptions in `/test` and ensures that parsing/rendering produces the structures in those files. For more information, see [`TreeTests.md`](docs/TreeTests.md).

### Philosophy

See [`Philosophy.md`](docs/Philosophy.md).

### Styling

CSS classes are named consistently, in kebab-case only, with prefixes:

* Any classes with the `wj-` prefix are those generated automatically, and not intended for direct use by users. An example would be `wj-collapsible-block`.
* Any classes with the `wiki-` prefix are "premade" classes. These are not necessarily generated automatically, but are instead intended for direct use by users wanting to make use of standard styling. An example would be `wiki-note`.

### Naming

"Foundation Text Markup Language" (ftml) is named for the file extension representing in-universe
SCP Foundation formatting as mentioned in [Kate McTiriss's Proposal](https://scpwiki.com/kate-mctiriss-s-proposal).
While the expanded form of the initialism is never explicitly stated, it is clearly implied given the
name similarity to HTML.

### Syntax

ftml is intended to be compatible with a subset of Wikidot text deemed to be "well-formed". Wikidot's general syntax documentation will be relevant here, but weird constructions or strange features may not be. During the development process, they are analyzed and either explicitly unimplemented, or implemented through more sensible syntax. Additionally, it supports several new features and blocks not present in Wikidot, such as checkboxes, and fixes bugs, such as allowing collapsibles to be nested.

As ftml develops into its own branch of wikitext, pages here will document the syntax separately from Wikidot, with the goal of deprecating Wikidot's documentation entirely.

- [`Blocks.md`](docs/Blocks.md) -- Which blocks (e.g. `[[div]]`) are available in ftml and what options they take.

There are some lesser-used or troublesome features which are implemented in a different, backwards-incompatible way with Wikidot. For instance:

* `[[include]]` is split into `[[include]]` (legacy Wikidot behavior), and `[[include-elements]]` (experimental self-contained element insertion).
* Interwiki links are implemented by prefixing `!` in triple-bracket links. So `[[[!wp:Amazon.com | Amazon]]]` instead of `[wp:Amazon.com Amazon]`.

### Usage

There are a couple main exported functions, which correspond to each of the main steps in the wikitext process.

First is `include`, which substitutes all `[[include]]` blocks for their replaced page content. This returns the substituted wikitext as a new string, as long as the names of all the pages that were used. It requires an object that implement `Includer`, which handles the process of retrieving pages and generating missing page messages.

Second is `preprocess`, which will perform Wikidot's various minor text substitutions.

Third is `tokenize`, which takes the input string and returns a wrapper type. This can be `.into()`-ed into a `Vec<ExtractedToken<'t>>` should you want the token extractions it produced. This is used as the input for `parse`.

Then, borrowing a slice of said tokens, `parse` consumes them and produces a `SyntaxTree` representing the full structure of the parsed wikitext.

Finally, with the syntax tree you `render` it with whatever `Render` instance you need at the time. Most likely you want `HtmlRender`. There is also `TextRender` for text-only, such as for searching article contents or a "printer-friendly" view.

```rust
fn include<'t, I, E>(
    input: &'t str,
    includer: I,
    settings: &WikitextSettings,
) -> Result<(String, Vec<PageRef<'t>>), E>
where
    I: Includer<'t, Error = E>;

fn preprocess(
    text: &mut String,
);

fn tokenize<'t>(
    text: &'t str,
) -> Tokenization<'t>;

fn parse<'r, 't>(
    tokenization: &'r Tokenization<'t>,
) -> ParseResult<SyntaxTree<'t>>;

trait Render {
    type Output;

    fn render(
        &self,
        info: &PageInfo,
        tree: &SyntaxTree,
    ) -> Self::Output;
}
```

When performing a parse, you will need to first run `preprocess()`, then run `parse()`
on the fully expanded text:

Consider the lifetimes of each of the artifacts being generated, should you want to
store the results in a `struct`.

```rust
// Get an `Includer`.
//
// See trait documentation for what this requires, but
// essentially it is some abstract handle that gets the
// contents of a page to be included.
//
// Two sample includers you could try are `NullIncluder`
// and `DebugIncluder`.
let includer = MyIncluderImpl::new();

// Get our source text
let mut input = "**some** test <<string?>>";

// Substitute page inclusions
let (mut text, included_pages) = ftml::include(input, includer, &settings);

// Perform preprocess substitutions
ftml::preprocess(&log, &mut text);

// Generate token from input text
let tokens = ftml::tokenize(&text);

// Parse the token list to produce an AST.
//
// Note that this produces a `ParseResult<SyntaxTree>`, which records the
// parsing warnings in addition to the final result.
let result = ftml::parse(&tokens, &page_info, &settings);

// Here we extract the tree separately from the warning list.
//
// Now we have the final AST, as well as all the issues that
// occurred during the parsing process.
let (tree, warnings) = result.into();

// Finally, we render with our renderer. Generally this is `HtmlRender`,
// but you could have a custom implementation here too.
//
// You must provide a `PageInfo` struct, which describes the page being rendered.
// You must also provide a handle to provide various remote sources, such as
// module content, but this is not stabilized yet.
let html_output = HtmlRender.render(&tree, &page_info, &settings);
```

### JSON Serialization

See [`Serialization.md`](docs/Serialization.md).
