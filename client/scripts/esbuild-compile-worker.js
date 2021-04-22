const esbuild = require("esbuild")
const tempDir = require("temp-dir")
const path = require("path")
const fs = require("fs/promises")

const dir = `${tempDir}/esbuild-compile-worker/`

/** Makes a folder if it doesn't exist. */
async function mkdir(dir) {
  try {
    await fs.mkdir(dir)
  } catch {}
}

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

      const pathImporter = args.importer
      const pathWorker = path.join(path.dirname(pathImporter), args.path)
      const filename = path.basename(pathWorker).replace(/ts$/, "js")

      return {
        path: dir + filename,
        namespace: "web-worker",
        pluginData: { pathWorker, filename }
      }
    })

    build.onLoad({ filter: /.*/, namespace: "web-worker" }, async args => {
      const {
        pluginData: { pathWorker, filename }
      } = args

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

      // write output to temp dir
      await mkdir(dir)
      await fs.writeFile(dir + filename, code)

      return {
        contents: code,
        loader: "text"
      }
    })
  }
}
