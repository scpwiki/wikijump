## wikijiump-locales-validator

Checks through the files in the `/locales/fluent` directory to ensure that they are in order and do not contain errors. This is meant to run in CI when any changes are made to Fluent files.

Like the rest of Wikijump, this is licensed as AGPL 3.0 or later.

### Execution

The program assumes it is run in this directory, that is, the Fluent files are available at "`../fluent`".

```sh
$ cargo run
```

(You could use `--release`, but the increase in compile times is likely larger than the time savings from faster execution)

### Development

```sh
$ cargo fmt     # Ensure code is formatted
$ cargo clippy  # Check code for lints
```
