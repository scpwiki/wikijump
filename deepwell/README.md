## DEEPWELL

<p>
  <a href="https://github.com/scpwiki/wikijump/actions?query=workflow%3A%22%5Bdeepwell%5D+Rust%22">
    <img src="https://github.com/scpwiki/wikijump/workflows/%5Bdeepwell%5D%20Rust/badge.svg"
         alt="Build status">
  </a>

  <!-- TODO publish to crates.io
  <a href="https://docs.rs/deepwell">
    <img src="https://docs.rs/deepwell/badge.svg"
         alt="docs.rs link">
  </a>
  -->
</p>

DEEPWELL is an experimental backend system to provide core wiki operations via an API for Wikijump.
This is intended as an internal API consumed by the web server as part of its logical tasks.

The lint `#![forbid(unsafe_code)]` is set, and therefore this crate has only safe code.

Available under the terms of the GNU Affero General Public License. See [LICENSE.md](LICENSE.md).

### Development

If you have [`sea-orm-cli`](https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/), and have a local instance of Wikijump running, you can use the following script to autogenerate SeaORM model files:

```sh
$ scripts/generate-models.sh
```

### Compilation

This executable targets the latest stable Rust. At time of writing, that is `1.58.0`.

```sh
$ cargo build --release
```

### Testing

Tests have not yet been implemented, but when they are, run:

```sh
$ cargo test
```

Add `-- --nocapture` to the end if you want to see test output.

### Development

```sh
$ cargo fmt     # Ensure code is formatted
$ cargo clippy  # Check code for lints
```
