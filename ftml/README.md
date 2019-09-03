## ftml
[![Build Status](https://travis-ci.org/Nu-SCPTheme/ftml.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/ftml)

**Foundation Text Markup Language** (formerly `wikidot-to-html`)

A Rust library and executable to convert Wikidot code into HTML. A reimplementation of the aging [Text\_Wiki](https://github.com/gabrys/wikidot/tree/master/lib/Text_Wiki/Text) from Wikidot.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).

### Compilation
This library targets the latest stable Rust. At time of writing, that is 1.37.0

```sh
$ cargo build --release
```

This will create the appropriate Rust library files and the following binary:

* `ftml` is a command-line tool to permit use of the library

The programs can be executed using the following:

```sh
$ cargo run --example ftml -- [arguments]
```

### Testing
```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.

### Usage
There are three relevant aspects to converting ftml-compatible sources into HTML:

* Prefilter
* Parse
* Render

**Prefilter**  
This is somewhat analogous to the preprocessor in C. It pastes in the sources of any files loaded with `[[include]]`, as well as miscellaneous transformations Wikidot adds, such as fancy typography.

The signature for this call is:
```rust
fn prefilter(text: &mut String, handle: &dyn RemoteHandle);
```

It modifies the input string `text` in-place, using the specified `handle` for any remote calls. (Most notably fetching pages for `[[include]]`.

**Parse**  
The primary bulk of this library's code exists here. The parser is built using [Pest](https://pest.rs), and constructs the abstract syntax tree (type `SyntaxTree`) from it. This operation can fail if the code is invalid, yielding an `Error`.

The signature for this call is:
```rust
fn parse<'a>(text: &'a str) -> Result<SyntaxError<'a>, Error>;
```

Relevant sections of text are borrowed directly from the input string if relevant.

**Render**  
The library provides a trait `Render`, which has the following signature:
```rust
trait Render {
    type Output;

    fn render(&self, tree: &SyntaxTree, info: PageInfo) -> Result<Self::Output>;
}
```
With a helper function `transform()` pre-implemented which calls all three of these steps for you.

This does not constrain the output type, but obviously the most desirable output here is HTML in the form of a string.

Of note are two helper renderers with trivial implementations for testing. `NullRender` outputs the `&'static str` "[[content]]", while `TreeRender` outputs a `String` with the formatted page information and abstract syntax tree.

The `HtmlRender` struct requires a `&dyn RemoteHandle` to be passed in at creation. This was mentioned in passing in the prefilter section, but it is essentially a trait which the library consumer can implement to retrieve information about an article. This is essentially information like getting user data, or the content of other pages, which in a "real" system you'd need to go to the database for.

Of note that `HtmlRender` does return a `String`, but an `HtmlOutput`:
```rust
#[derive(Debug, Clone, Default)]
pub struct HtmlOutput {
    pub html: String,
    pub style: String,
    pub meta: Vec<HtmlMeta>,
}
```

This is not a complete HTML document, but rather the body, styling, and `<meta>` tags which can be used by the consumer to produce the final output. This way it is easier to modify the output, or add additional metadata tags or load the correct CSS theme.
