## ftml

<p>
  <a href="https://github.com/scpwiki/wikijump/actions?query=workflow%3A%22%5Bftml%5D+Rust%22">
    <img src="https://github.com/scpwiki/wikijump/workflows/%5Bftml%5D%20Rust/badge.svg"
         alt="Build status">
  </a>

  <!-- TODO: put crates.io badge here -->
</p>

### Foundation Text Markup Language

A Rust library to parse Wikidot text ("Wikitext") into an abstract syntax tree (AST).
This aims to be a replacement for the aging [Text\_Wiki](https://github.com/gabrys/wikidot/tree/master/lib/Text_Wiki/Text) from Wikidot.
This is version aims to have a nearly fully compatible parser for common Wikidot, including common malformed constructs.
The goal is to utilize a lexer generator, and consume the tokens in a custom parser to handle unusual cases with a lax approach.

In addition to providing the speed and safety benefits of Rust, this also improves maintainability, and allows exposing an AST to consumers
for more advanced analysis and transformation.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have `unsafe` internals.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE.md).

### Compilation
This library targets the latest stable Rust. At time of writing, that is `1.49.0`.

```sh
$ cargo build --release
```

You can use this as a dependency by adding the following to your `Cargo.toml`:

```toml
ftml = "0.4"
```

### Testing
```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.
If you wish to see the logging output, you can change `crate::build_logger()`
to use a different logger creation implementation. Or you can modify the test
you're inspecting to use a different logger.

### Philosophy

See [`Philosophy.md`](docs/Philosophy.md).

### Naming
"Foundation Text Markup Language" (ftml) is named for the file extension representing in-universe
SCP Foundation formatting as mentioned in [Kate McTiriss's Proposal](http://www.scpwiki.com/kate-mctiriss-s-proposal).
While the expanded form of the initialism is never explicitly stated, it is clearly implied given the
name similarity to HTML.

### Syntax
ftml is intended to be compatible with a subset of Wikidot text deemed to be "well-formed". Wikidot's general syntax documentation will be relevant here, but weird constructions or strange features may not be. During the development process, they are analyzed and either explicitly unimplemented, or implemented through more sensible syntax.

As ftml develops into its own branch of wikitext, pages here will document the syntax separately from Wikidot, with the goal of deprecating Wikidot's documentation entirely.

* [`Blocks.md`](docs/Blocks.md) -- Which blocks (e.g. `[[div]]`) are available in ftml and what options they take.

### Usage
There are a couple main exported functions, which correspond to each of the main steps in the wikitext process.

First is `include`, which substitutes all `[[include]]` blocks for their replaced page content. This returns the substituted wikitext as a new string, as long as the names of all the pages that were used. It requires an object that implement `Includer`, which handles the process of retrieving pages and generating missing page messages.

Second is `preprocess`, which will perform Wikidot's various minor text substitutions.

Third is `tokenize`, which takes the input string and returns a wrapper type. This can be `.into()`-ed into a `Vec<ExtractedToken<'t>>` should you want the token extractions it produced. This is used as the input for `parse`.

Then, borrowing a slice of said tokens, `parse` consumes them and produces a `SyntaxTree` representing the full structure of the parsed wikitext.

Finally, with the syntax tree you `render` it with whatever `Render` instance you need at the time. Most likely you want `HtmlRender`.

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
```

### JSON Serialization

See [`Serialization.md`](docs/Serialization.md).

### Server
If you wish to build the `ftml-http` subcrate, use the following:
Note that it was primarily designed for UNIX-like platforms, but with
some minor changes could be modified to work on Windows.

```sh
$ cargo build -p ftml-http --release
$ cargo run -p ftml-http
```

This will produce an HTTP server which a REST client can query to perform ftml operations.

For typical applications the only relevant route would be `POST /render/html`.
The others are provided to expose library internals, such as extracted tokens,
if they are desired.

You can see a full list of REST methods in [`ServerRoutes.md`](docs/ServerRoutes.md).

Its usage message (produced by adding `-- --help` to the above `cargo run` invocation)
is reproduced below:

```
ftml ftml-http v0.3.1 [8a42fccd]
Wikijump Team
REST server to parse and render Wikidot text.

USAGE:
    ftml-http [FLAGS] [OPTIONS]

FLAGS:
    -h, --help         Prints help information.
        --info-only    Print information then exit.
    -4, --ipv4         Only host the server on IPv4.
    -V, --version      Prints version information.

OPTIONS:
    -l, --log-file <FILE>      The log file to write formatted entries to [default: ftml.log]
    -L, --log-level <LEVEL>    Log level to be use when running the server [default: debug]
    -p, --port <PORT>          The port to be used by the server [default: 3865]
```

An example invocation with with `curl` is provided:

```
$ curl \
    -i \
    -X POST \
    -H 'Content-Type: application/json' \
    --compressed \
    --data '
{
    "text": "<your input here>",
    "callback-url": "http://localhost:8000/included-pages",
    "missing-include-template": "No page {{ page }} {% if site %}on site {{ site }} {% endif %}exists!"
}' \
        http://localhost:3865/parse
```
