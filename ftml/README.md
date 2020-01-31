## ftml
[![Build Status](https://travis-ci.org/Nu-SCPTheme/ftml.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/ftml)

**Foundation Text Markup Language**

A Rust library and executable to convert Wikidot code into HTML. This aims to be a replacement for the aging [Text\_Wiki](https://github.com/gabrys/wikidot/tree/master/lib/Text_Wiki/Text) from Wikidot. However, it is not a completely backwards-compatible library. Instead it aims to support a subset referred to here as being "well-formed". Additionally it has some extensions to make certain design patterns easier to accomplish. See the section below for more information.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code. However dependencies may have `unsafe` internals.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).

### Compilation
This library targets the latest stable Rust. At time of writing, that is 1.41.0

```sh
$ cargo build --release
```

This will create the appropriate Rust library files and the following `ftml` binary.

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
fn parse<'a>(text: &'a str) -> Result<SyntaxTree<'a>, Error>;
```

Relevant sections of text are borrowed directly from the input string if needed.

**Render**  
The library provides a trait `Render`, which has the following signature:
```rust
trait Render {
    type Output;

    fn render(&self, tree: &SyntaxTree, info: PageInfo) -> Result<Self::Output, Error>;
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

### Well-formed Wikidot
The library does not support all possible Wikidot code, as a fully-compatible parser would essentially be a clone of the hacks used in the original PHP source. `Text_Wiki` functions by searching+replacing various terms throughout the document until it creates the final output, and unsurprisingly can produce invalid HTML.

For instance, the following is valid code:
```
> [[div class="test"]
> A man, a plan, a canal, Panama.
[[/div]]
```

However the actual extent of the blockquote intersects with the div, and it essentially is the HTML equivalent of
```html
<div class="outer">
  <p class="inner">
  </div>
</p>
```

Which is obviously invalid syntax, and can cause issues.

Instead the library's parser defines a grammar, which is designed to be compatible with all common Wikidot constructions, or has extensions for situations that are not directly supported. This largely-overlapping but slightly dissimilar specification ("ftml code") aims at being able to _effectively_ replace Wikidot code with minor human involvement to replace malformed original sources.

### Extensions
TODO
