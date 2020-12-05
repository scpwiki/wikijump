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
