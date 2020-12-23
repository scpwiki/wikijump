## ftml

<p>
  <a href="https://github.com/Nu-SCPTheme/ftml/actions?query=workflow%3A%22Rust+CI%22">
    <img src="https://github.com/Nu-SCPTheme/ftml/workflows/Rust%20CI/badge.svg"
         alt="Rust CI badge">
  </a>
</p>

**Foundation Text Markup Language**

A Rust library to parse Wikidot code ("Wikitext") into an abstract syntax tree (AST).
This aims to be a replacement for the aging [Text\_Wiki](https://github.com/gabrys/wikidot/tree/master/lib/Text_Wiki/Text) from Wikidot.
This is an experimental branch to try and have a nearly fully compatible parser for Wikidot, including malformed constructs.
The goal is to utilize a lexer generator, and consume the tokens in a custom parser to handle unusual cases with a lax approach.

In addition to providing the speed and safety benefits of Rust, this also improves maintainability, and allows exposing an AST to consumers
for more advanced analysis and transformation.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have `unsafe` internals.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).

### Compilation
This library targets the latest stable Rust. At time of writing, that is 1.48.0

```sh
$ cargo build --release
```

You can use this as a dependency by adding the following to your `Cargo.toml`:

```toml
ftml = { git = "https://github.com/NuSCP-Theme/ftml", branch = "master" }
```

The normal package on crates.io is, currently, not being regularly updated.

### Testing
```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.
If you wish to see the logging output, you can change `crate::build_logger()`
to use a different logger creation implementation. Or you can modify the test
you're inspecting to use a different logger.

### Server
If you wish to build the `ftml-server` subcrate, use the following:
Note that it was primarily designed for UNIX-like platforms, but with
some minor changes could be modified to work on Windows.

```sh
$ cargo build -p ftml-server --release
$ cargo run -p ftml-server
```

This will produce an HTTP server which a REST client can query to perform ftml operations.

It currently has the following routes:

Note that input text are really simple JSON objects in the following form:
```json
{
    "text": "<your input string>"
}
```

| Method | Route | Input | Output | Description |
|--------|-------|-------|--------|-------------|
| Any | `/ping` | None | `String` | See if you're able to connect to the server. |
| Any | `/version` | None | `String` | Outputs what version of ftml is being run. |
| `POST` | `/preprocess` | Text | `String` | Runs the preprocessor on the given input string. |
| `POST` | `/tokenize` | Text | `Vec<ExtractedToken>` | Runs the tokenizer on the input string and returns the extracted tokens. |
| `POST` | `/tokenize/only` | Text | `Vec<ExtractedToken>` | Same as above, but the preprocessor is not run first. |
| `POST` | `/parse` | Text | `ParseResult<SyntaxTree>` | Runs the parser on the input string and returns the abstract syntax tree. |
| `POST` | `/parse/only` | Text | `ParseResult<SyntaxTree>` | Same as above, but the preprocessor is not run first. |
| `POST` | `/render/html` | Text | `ParseResult<HtmlOutput>` | Performs the full rendering process, from preprocessing, tokenization, parsing, and then rendering. |
| `POST` | `/render/html/only` | Text | `ParseResult<HtmlOutput>` | Same as above, but the preprocessor is not run first. |
| `POST` | `/render/debug` | Text | `ParseResult<String>` | Performs rendering, as above, but uses `ftml::DebugRender`. |
| `POST` | `/render/debug/only` | Text | `ParseResult<String>` | Same as above, but the preprocessor is not run first. |

For typical applications the only relevant route would be `POST /render/html`.
The others are provided to expose library internals, such as extracted tokens,
if they are desired.

Its usage message (produced by adding `-- --help` to the above `cargo run` invocation)
is reproduced below:

```
ftml ftml-server v0.3.1 [8a42fccd]
Wikijump Team
REST server to parse and render Wikidot text.

USAGE:
    ftml-server [FLAGS] [OPTIONS]

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

### Philosophy
Wikitext is similar to Markdown and dissimilar to C in that the grammar is loose.
Any invalid token combinations are rendered as-is, rather than producing a fatal parsing
error which halts the process. This latter, C-like (or any programming language, really)
philosophy was how the original version of ftml operated. However this presents obvious
incomaptibilities with Wikidot, and the grammar had to be increasingly complicated to handle
edge-case conditions.

This rewrite of ftml performs preprocessing substitions and tokenization like the first version,
but the parser is hand-written to allow for loose fallback rules.

More specifically, for each encountered token, the parser will attempt to match the first rule
which expects it. Incoming tokens will be handled, producing elements, or until an invalid token
is received, at which point an "error" will be produced and this rule will abort.

Following this, the parser will attempt to apply the second rule (if any), etc., until all rules are
exhausted. At this point, if no match can be made, the default "fallback" rule is applied. This is
the case where all the tokens are interpreted literally.

For any tokens which are successfully consumed to produce elements, the pointer to the remaining tokens
is bumped up (really, a later subslice is taken), and the element is appended to the final list.
Note that this operation is applied recursively, so that any containers (elements which contain other
elements) will perform this same operation to populate themselves.

It is important to note that, in accordance to the Wikidot parsing strategy, all "errors" are non-fatal.
In the worst-case scenario, all tokens fail all rules, and all are parsed with the fallback, rule, producing
an error for each incident. In a more typical case, any invalid structures will produce errors, and will
parsed as best it can.

These errors are returned to the caller to provide information on where the process failed, while still
producing the fallback render. This provides the both of best worlds: errors to assist with wikitext
debugging, but also not hard-failing rendering in case of any error.

### Naming
"Foundation Text Markup Language" (ftml) is named for the file extension representing in-universe
SCP Foundation formatting as mentioned in [Kate McTiriss's Proposal](http://www.scpwiki.com/kate-mctiriss-s-proposal).
While the expanded form of the initialism is never explicitly stated, it is clearly implied given the
name similarity to HTML.

### Usage
There are three exported functions, which correspond to each of the main steps in the wikitext process.

First is `preprocess`, which will perform Wikidot's various minor text substitutions.

Second is `tokenize`, which takes the input string and returns a wrapper type. This can be `.into()`-ed into a `Vec<ExtractedToken<'t>>` should you want the token extractions it produced. This is used as the input for `parse`.

Then, borrowing a slice of said tokens, `parse` consumes them and produces a `SyntaxTree` representing the full structure of the parsed wikitext.

```rust
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

// Perform preprocess substitions
let mut text = str!("**some** test <<string?>>");
ftml::preprocess(&log, &mut text);

// Generate token from input text
let tokens = ftml::tokenize(&log, &text);

// Parse the token list to produce an AST.
//
// Note that this produces a `ParseResult<SyntaxTree>`, which records the
// parsing errors in addition to the final result.
let result = ftml::parse(&log, &tokens);

// Here we extract the tree separately from the error list.
//
// Now we have the final AST, as well as all the issues that
// occurred during the parsing process.
let (tree, errors) = result.into();
```

### JSON Serialization
All exposed fields are serializable with [`serde`](https://crates.io/crates/serde). If you use [`serde_json`](https://crates.io/crates/serde_json) to store syntax trees (as is used in `src/test.rs` and the `/test` directory), it is helpful to understand the basics of how these data types will be serialized. These principles will apply to other formats as well, but this section will focus on JSON.

The top level of a syntax tree contains two fields, `elements` and `styles`. The latter is simple, just a list of strings, each representing on CSS style within the wikitext. The first is of more interest, and more complex.

The Rust declaration of `Element` is as an enum, with each variant representing a different kind of element one may encounter. Most of these are leaf elements, such as `text` or `link`. Serde has been configured to use discriminated tagging, so the object representation will look like:

(Note that the serialized form for _all_ data structures uses `kebab-case`.
For instance, `Token::LeftLink` is represented as `left-link`.)

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
