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
