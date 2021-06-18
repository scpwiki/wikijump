# Espells: Hunspell ported to Python ported to JavaScript

Pure JS/TS spellchecker, using Hunspell dictionaries. Direct port of the [Spylls](https://github.com/zverok/spylls) library. Without zverok's (the author of Spylls) work, this library couldn't exist.

Espells makes no use of features that would prevent it from running within Node, a browser, or even a web worker. Effectively, it's just a pure spellchecking library and it's up to you to connect it to whatever interface you want.

### Why?

Two main reasons:
1. You can't access the browser's spellchecking functionality in JS.
2. There wasn't a fully featured spellchecker for clientside JS.

To elaborate on that second point: libraries using Hunspell, or are compatible with Hunspell dictionaries, do exist. However, they're either incapable of handling many dictionaries, aren't fully featured, can only run in Node, or bastardize Hunspell to run in WASM, with mixed results. Espells doesn't have those problems.
