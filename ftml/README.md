## ftml

<p>
  <a href="https://github.com/scpwiki/wikijump/actions?query=workflow%3A%22%5Bftml%5D+Rust%22">
    <img src="https://github.com/scpwiki/wikijump/workflows/%5Bftml%5D%20Rust/badge.svg"
         alt="Build status">
  </a>

  <a href="https://docs.rs/ftml">
    <img src="https://docs.rs/ftml/badge.svg"
         alt="docs.rs link">
  </a>
</p>

### Foundation Text Markup Language

A Rust library to parse Wikidot text ("Wikitext") into an abstract syntax tree (AST).
This aims to be a replacement for the aging [Text\_Wiki](https://github.com/gabrys/wikidot/tree/master/lib/Text_Wiki/Text) from Wikidot.
This is version aims to have a nearly fully compatible parser for common Wikidot, including common malformed constructs.
The goal is to utilize a lexer generator, and consume the tokens in a custom parser to handle unusual cases with a lax approach.

In addition to providing the speed and safety benefits of Rust, this also improves maintainability, and allows exposing an AST to consumers
for more advanced analysis and transformation.

The lint `#![deny(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have `unsafe` internals, and the `ffi` module contains unsafe code to interface with C ABIs.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE.md).

### Compilation

This library targets the latest stable Rust. At time of writing, that is `1.57.0`.

```sh
$ cargo build --release
```

You can use this as a dependency by adding the following to your `Cargo.toml`:

```toml
ftml = "1"
```

The library comes with two default features, `log`, `ffi`, and `mathml`.

The `log` feature adds all `slog` logging code, which when removed replaces all of them with no-ops.
This may be desirable on certain platforms where the performance difference is significant.

The `ffi` feature introduces an FFI interface for ftml, permitting C and C API-compatible code
to interface with the library.

The `mathml` feature includes `latex2mathml`, which compiles any LaTeX into MathML for inclusion
in rendered HTML output.

Note that, when compiling for the `wasm32` target, even if the `ffi` feature is enabled, its
corresponding code is not built.

```
$ cargo check --no-default-features
```

If you wish to build the WebAssembly target for ftml, use `wasm-pack`:

```
$ wasm-pack build -- --no-default-features
```

This produces a build with no `slog` logging at all, which is helpful for limiting the binary footprint and improving performance.

However, there is a `wasm-log` feature, which initializes a `console.log()`-based `slog::Logger` for WebASM. Note that this will slam your brower's console hard and is **not** recommended for production, only local testing.

If developing and just want to check that the build passes, use:

```
$ wasm-pack build --dev
```

Without release optimizations, this runs fast enough to use during development.

If for some reason you want to invoke `cargo check` instead, call `cargo check --target wasm32-unknown-unkown`.

### Testing

```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.
If you wish to see the logging output, you can change `crate::build_logger()` to use a different logger
creation implementation. Or you can modify the test you're inspecting to use a different logger.

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

ftml is intended to be compatible with a subset of Wikidot text deemed to be "well-formed". Wikidot's general syntax documentation will be relevant here, but weird constructions or strange features may not be. During the development process, they are analyzed and either explicitly unimplemented, or implemented through more sensible syntax.

As ftml develops into its own branch of wikitext, pages here will document the syntax separately from Wikidot, with the goal of deprecating Wikidot's documentation entirely.

- [`Blocks.md`](docs/Blocks.md) -- Which blocks (e.g. `[[div]]`) are available in ftml and what options they take.

### Usage

There are a couple main exported functions, which correspond to each of the main steps in the wikitext process.

First is `include`, which substitutes all `[[include]]` blocks for their replaced page content. This returns the substituted wikitext as a new string, as long as the names of all the pages that were used. It requires an object that implement `Includer`, which handles the process of retrieving pages and generating missing page messages.

Second is `preprocess`, which will perform Wikidot's various minor text substitutions.

Third is `tokenize`, which takes the input string and returns a wrapper type. This can be `.into()`-ed into a `Vec<ExtractedToken<'t>>` should you want the token extractions it produced. This is used as the input for `parse`.

Then, borrowing a slice of said tokens, `parse` consumes them and produces a `SyntaxTree` representing the full structure of the parsed wikitext.

Finally, with the syntax tree you `render` it with whatever `Render` instance you need at the time. Most likely you want `HtmlRender`. There is also `TextRender` for text-only, such as for searching article contents or a "printer-friendly" view.

```rust
fn include<'t, I, E>(
    log: &slog::Logger,
    input: &'t str,
    includer: I,
) -> Result<(String, Vec<PageRef<'t>>), E>
where
    I: Includer<'t, Error = E>;

fn preprocess(
    log: &slog::Logger,
    text: &mut String,
);

fn tokenize<'t>(
    log: &slog::Logger,
    text: &'t str,
) -> Tokenization<'t>;

fn parse<'r, 't>(
    log: &slog::Logger,
    tokenization: &'r Tokenization<'t>,
) -> ParseResult<SyntaxTree<'t>>;

trait Render {
    type Output;

    fn render(
        &self,
        log: &slog::Logger,
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
// Generate slog logger.
//
// See https://docs.rs/slog/2.7.0/slog/ for crate information.
// You will need a drain to produce an instance, as that's where
// journalled messages are outputted to.
let log = slog::Logger::root(/* drain */);

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
let (mut text, included_pages) = ftml::include(&log, input, includer);

// Perform preprocess substitions
ftml::preprocess(&log, &mut text);

// Generate token from input text
let tokens = ftml::tokenize(&log, &text);

// Parse the token list to produce an AST.
//
// Note that this produces a `ParseResult<SyntaxTree>`, which records the
// parsing warnings in addition to the final result.
let result = ftml::parse(&log, &tokens);

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
let html_output = HtmlRender.render(&log, &page_info, &tree);
```

### JSON Serialization

See [`Serialization.md`](docs/Serialization.md).
