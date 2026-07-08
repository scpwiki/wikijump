# @wikijump/prism

This module wraps around the Prism syntax highlighting library.

> ### IMPORTANT
> The Prism library is vendored into this package. The file Prism resides in is mostly unmodified, except in two ways:
> 1. The `manual` property is hardcoded to `true`
> 2. The `disableWorkerMessageHandler` is hardcoded to `true`
>
> If the `prism.js` file is updated, e.g. with new languages, you must make the same edits to the file.
>
> These values will be found near the top of whatever Prism file you are editing.

The following vendored Prism grammars are disabled at runtime because CodeQL flags exponential backtracking risk in their current regular expressions:
* `aspnet`
* `csharp`, `cs`, `dotnet`
* `jsx`
* `rust`
* `svelte`
* `tsx`

The following languages have highlighting support:
* `css`
* `clike`
* `javascript`
* `abnf`
* `actionscript`
* `apl`
* `arduino`
* `asciidoc`
* `autohotkey`
* `bash`
* `basic`
* `batch`
* `bnf`
* `brainfuck`
* `brightscript`
* `c`
* `cpp`
* `clojure`
* `cobol`
* `coffeescript`
* `crystal`
* `csv`
* `d`
* `dart`
* `diff`
* `docker`
* `ebnf`
* `editorconfig`
* `elixir`
* `elm`
* `erlang`
* `fsharp`
* `flow`
* `fortran`
* `git`
* `glsl`
* `go`
* `graphql`
* `haskell`
* `hcl`
* `hlsl`
* `http`
* `hpkp`
* `hsts`
* `ignore`
* `ini`
* `java`
* `json`
* `json5`
* `jsonp`
* `julia`
* `kotlin`
* `latex`
* `less`
* `lisp`
* `log`
* `lua`
* `makefile`
* `markdown`
* `matlab`
* `nasm`
* `nginx`
* `nim`
* `objectivec`
* `ocaml`
* `opencl`
* `pascal`
* `perl`
* `php`
* `plsql`
* `powershell`
* `purescript`
* `python`
* `qml`
* `r`
* `regex`
* `rest`
* `ruby`
* `sass`
* `scss`
* `scala`
* `scheme`
* `smalltalk`
* `smarty`
* `sql`
* `stylus`
* `swift`
* `toml`
* `typescript`
* `v`
* `vim`
* `wasm`
* `yaml`
* `zig`
