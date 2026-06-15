[<< Return to the README](../README.md)

## Abstract Syntax Tree Tests

An important part of verifying that ftml performs as expected is having an expansive battery of unit tests. This is essentially an organized set of files which feeds a particular wikitext as input into the parser and renderer(s), and then checks the outputs against the expected results.

### File Structure

Within `/test` in this repository, there are directories which correspond to "test groups", a set of tests which are related in some way. Within each test group is a particular test case.

Consider this structure:

```
test/
├── diff
│   ├── alias
│   ├── basic
│   └── newlines
└── underline
    ├── basic
    ├── empty
    └── fail
```

These directories define the following six test cases:
* `diff/alias`
* `diff/basic`
* `diff/newlines`
* `underline/basic`
* `underline/empty`
* `underline/fail`

Within each unit test directory, we have a series of files with standard names:
* `input.ftml` (required) &mdash; The input wikitext for this test case.
* `tree.json` &mdash; The syntax tree produced from parsing this wikitext.
* `errors.json` (required if `tree.json` exists and parsing yields errors) &mdash; If absent, assumed to be equivalent to `[]`. If present, then all errors within are compared to those produced by the parser.
* `wikidot.html` &mdash; The rendered output (HTML renderer, Wikidot layout).
* `output.html` &mdash; The rendered output (HTML renderer, Wikijump layout).
* `output.txt` &mdash; The rendered output (text renderer).

This way, if a particular test only cares about, say, the Wikidot-compatible HTML output, you can check only that, or if it only wants to ensure the syntax tree is as expected then you can do that.

### Execution

You can run the AST test suite and see the output by using `cargo`:

```
$ cargo test -- ast --nocapture
```

This will run all syntax tree tests currently in the repository.

### Adding Tests

Adding a new test is easy. Simply create a new directory with the appropriate name in the structure described above. Let's say we're adding a new block called `[[foobar]]`. Then I may wish to make `/test/foobar/basic` and `/test/foobar/fail`.

One convention within our test system is to have "fail" tests as a separate case, meaning inputs with intentionally incorrect syntax to handle how our system processes and emits parser errors.

Then within each test case directory, create the files you care about. You will always need an `input.ftml`, but you may want to check all the outputs or only one. (The test system will complain if there are no outputs to check against.)

You can either create the outputs manually, or (especially for the syntax tree JSON which can be tedious to write out), you can use the in-built system to update tests (see below).

### Updating Tests

In `/src/test/ast/mod.rs`, there is a constant called `UPDATE_TESTS`. Tests are only able to pass if this is set to `false`, but setting this value to `true` can be of use during development. When enabled, any output files which diff from the current expected values will instead **overwrite those files** with the new outputs.

This is intended to be used in cases where you are adding new tests (see above), or a breaking change has been made to the tree structure or HTML generation for one or more tests.

In such a case, temporarily enable the feature, run `cargo test -- ast`, and then **check the git diff**. It is _imperative_ that developers carefully review what the parser or renderer now produces instead of blindly accepting it.

If a change is causing a ton of tree elements to change or disappear, and you weren't expecting that, _that's probably a bug_! If a bunch of things that were list structures is now all text, that doesn't seem right! Carefully review outputs whenever you use this feature.

__NOTE:__ If your test case is not meant to produce errors, then **do not include an `errors.json` file**. Since in such case errors should never be happening, even when `UPDATE_TESTS` is enabled, instead of writing a `errors.json` file it will instead crash. If errors are expected, add a blank `errors.json`, if they're not, then fix your parser or fix your test.

To assist developers when adding new JSON test output files, `UPDATE_TESTS` mode includes a handy feature. Whenever it is updating outputs and encounters an _empty_ (size 0) JSON file, it will instead interpret it as a valid empty structure of that type.

This is to avoid monotonous additions of basic fields to satisfy the JSON parser, before it all gets ignored and overwritten anyways. In essence, think of truly empty files as having a special meaning for AST tests in update mode.

### Deleting Tests

In general, developers should avoid deleting tests. Whenever a new edge case is found, a feature is added, a Wikidot bug discovered, there should be new tests added to increase the project's overall coverage.

However, in cases where you do need to delete a test, it's simple: delete the relevant directories.
