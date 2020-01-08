## ftml-rpc

An RPC server and client for [ftml](https://github.com/Nu-SCPTheme/ftml) calls.
See the relevant crate documentation for more information on "Foundation Text Markup Language".

Currently it does not connect to a [DEEPWELL](https://github.com/Nu-SCPTheme/deepwell) server for
necessary external information (other pages, users, etc.), but will in the future.

### Compilation
This crate targets the latest stable Rust. At time of writing, that is 1.40.0

```sh
$ cargo build --release
$ cargo run --release -- [arguments] # server
```

If you wish to use its client, import the crate and use it as a library.

### API
Currently, all errors are transmitted as `Err(String)`.

The current API provided by the RPC server is as follows:

`protocol() -> io::Result<String>`:  
Returns a static protocol version. Currently `"0"`.

`ping() -> io::Result<()>`:  
Determines if the server is reachable.

`time() -> io::Result<()>`:  
Returns the system time on the server. It may be in any timezone and is not monotonic.

Followed by the three core ftml methods:

`prefilter(input: String) -> io::Result<Result<String, String>>`:  
Preprocesses the text prior to parsing or rendering.
This will load any includes or perform typographic transformations.
See [ftml Usage](https://github.com/Nu-SCPTheme/ftml#usage) for more information.

`parse(input: String) -> io::Result<Result<String, String>>`:  
Prefilters and then parses the input string, returning a JSON object corresponding to the syntax tree.
This is currently not typed in code, but follows the object pattern in [ftml's `SyntaxTree`](https://github.com/Nu-SCPTheme/ftml/blob/master/src/parse/tree/object.rs).

`render(page_info: PageInfoOwned, input: String) -> io::Result<Result<HtmlOutput, String>>`:  
Prefilters, parses, and renders the page into HTML. The definition of the `HtmlOutput` object is available
[here](https://github.com/Nu-SCPTheme/ftml/blob/master/src/render/html/object.rs), though most notable are
the `html` field containing partial HTML (i.e. not including tags like `<html>` or `<body>`), and `style`,
which contains any generated CSS.

The argument of type [`PageInfoOwned`](https://github.com/Nu-SCPTheme/ftml/blob/master/src/info.rs) passes
in information about the article being rendered.
