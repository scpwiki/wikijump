const esbuild = require("esbuild")
const path = require("path")

/** esbuild plugin for compiling and inlining web-workers.
 *  This plugin will take an import like:
 *  ```ts
 *  import worker from "./my-worker.worker.ts"
 *  ```
 *  and convert that into a constant expression that looks like:
 *  ```ts
 *  const worker = "...worker text..."
 *  ```
 *  This string can then be used to load a worker from a blob.
 *  This avoids most of the file import issues present with workers, and makes
 *  the worker safe to use from inside of a library. */
module.exports = {
  name: "compile-worker",
  setup(build) {
    build.onResolve({ filter: /\.worker(\..+)?$/ }, args => {
      // path can't be resolved, ignore
      if (args.resolveDir === "") return

      const pathWorker = path.join(path.dirname(args.importer), args.path)

      return {
        path: args.path,
        namespace: "web-worker",
        pluginData: { pathWorker }
      }
    })

    build.onLoad({ filter: /.*/, namespace: "web-worker" }, async args => {
      const { pathWorker } = args.pluginData

      const { minify = true } = build.initialOptions

      // build the worker under IIFE so that it has no exports, no imports
      // should be 100% web-worker compatible
      const built = await esbuild.build({
        entryPoints: [pathWorker],
        minify,
        bundle: true,
        treeShaking: true,
        outdir: "./",
        outbase: "./",
        format: "iife",
        platform: "browser",
        write: false,
        loader: { ".wasm": "file" },
        define: {
          "window": "globalThis",
          "import.meta.url": '""'
        }
      })

      const code = built.outputFiles?.[0]?.contents

      if (!code) throw new Error("Empty worker build result!")

      return {
        contents: code,
        loader: "text"
      }
    })
  }
}
