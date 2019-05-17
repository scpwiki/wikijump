## wikidot-to-html
[![Build Status](https://travis-ci.org/Nu-SCPTheme/wikidot-to-html.svg?branch=master)](https://travis-ci.org/Nu-SCPTheme/wikidot-to-html)

A Rust library and executable to convert Wikidot code into HTML. A reimplementation of the aging [Text\_Wiki](https://github.com/gabrys/wikidot/tree/master/lib/Text_Wiki/Text) from Wikidot.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE).

### Compilation
This library targets the latest stable Rust. At time of writing, that is 1.34.2

```sh
$ cargo build --release
$ cargo build --release --example wdtohtml
$ cargo build --release --example wdhtmlserv
```

This will create the appropriate Rust library files and the two packaged binaries:

* `wdtohtml` is a command-line tool to permit use of the library
* `wdhtmlserv` is a server that listens on a Unix Domain Socket for library commands

The programs can be executed using the following:

```sh
$ cargo run --example [wdtohtml|wdhtmlserv] -- [arguments]
```

### Testing
```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.
