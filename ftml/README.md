## ftml

<p>
  <a href="https://github.com/Nu-SCPTheme/ftml/actions?query=workflow%3A%22Rust+CI%22">
    <img src="https://github.com/Nu-SCPTheme/ftml/workflows/Rust%20CI/badge.svg"
         alt="Rust CI badge">
  </a>
</p>

**Foundation Text Markup Language**

A Rust library to parse Wikidot code into an abstract syntax tree (AST).
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

### Usage
There are three exported functions, which correspond to each of the main steps in the wikitext process.

First is `preprocess`, which will perform Wikidot's various minor text substitutions.

Second is `tokenize`, which takes the input string and returns a list of extracted tokens from it, all borrowing from it.

Then, borrowing a slice of said tokens, `parse` consumes them and produces a `SyntaxTree` representing the full structure of the parsed wikitext.

```rust
fn preprocess(
    log: &slog::Logger,
    text: &mut String,
);

fn tokenize<'t>(
    log: &slog::Logger,
    text: &'t str,
) -> Vec<ExtractedToken<'t>>;

fn parse<'r, 't>(
    log: &slog::Logger,
    tokens: &'r [ExtractedToken<'t>],
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
