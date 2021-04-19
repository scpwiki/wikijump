/*
  Big thanks goes to "tompark" on Github.
  https://github.com/evanw/esbuild/issues/408#issuecomment-757555771
  They went through the agonizing process of figuring out how you get this to work.

  This file is an esbuild plugin for resolving `wasm` files.
  If you import a `wasm` file, you'll get a binary bundle for it.
*/

module.exports = {
  name: "wasm",
  setup(build) {
    let path = require("path")
    let fs = require("fs")

    // Resolve ".wasm" files to a path with a namespace
    build.onResolve({ filter: /\.wasm$/ }, args => {
      if (args.resolveDir === "") {
        return // Ignore unresolvable paths
      }
      return {
        path: path.isAbsolute(args.path)
          ? args.path
          : path.join(args.resolveDir, args.path),
        namespace: "wasm-binary"
      }
    })

    // Virtual modules in the "wasm-binary" namespace contain the
    // actual bytes of the WebAssembly file. This uses esbuild's
    // built-in "binary" loader instead of manually embedding the
    // binary data inside JavaScript code ourselves.
    build.onLoad({ filter: /.*/, namespace: "wasm-binary" }, async args => ({
      contents: await fs.promises.readFile(args.path),
      loader: "binary"
    }))
  }
}
