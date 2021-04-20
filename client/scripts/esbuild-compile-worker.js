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

/*
 * the way this plugin works is that it will accept imports that look like this:
 * import url from "./foo.worker.ts"
 * and transform that "url" variable into a constant with a resolved path leading
 * to a transformed web-worker compatible build of "foo.worker.ts".
 *
 * to get that to work smoothly, this plugin builds "foo.worker.ts" to the temp directory
 * and then points esbuild's "file" loader at it, which does that
 * url import transformation, and copies the built file to the build directory.
 */

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

      // get what we can from the build options
      const { abs, outdir, minify = true } = build.initialOptions

      // build the worker under IIFE so that it has no exports, no imports
      // should be 100% web-worker compatible
      const built = await esbuild.build({
        entryPoints: [pathWorker],
        minify,
        bundle: true,
        treeShaking: true,
        sourcemap: true,
        sourcesContent: true,
        outdir: "./",
        outbase: "./",
        format: "iife",
        platform: "browser",
        write: false,
        define: {
          "window": "globalThis",
          "import.meta.url": '""'
        }
      })

      let code, map
      built.outputFiles.forEach(file => {
        if (file.path.endsWith(".map")) map = file.contents
        if (file.path.endsWith(".js")) code = file.contents
      })

      // write output to temp dir
      await mkdir(dir)
      await fs.writeFile(dir + filename, code)

      // if we can, we'll try to resolve where the dist folder is
      // that way we can make the sourcemap work
      if (abs && outdir) {
        const out = `${path.join(abs, outdir)}/`
        await mkdir(out)
        await fs.writeFile(`${out + filename}.map`, map)
      }

      // file loader just returns a URL reference when you import something
      return {
        contents: code,
        loader: "file"
      }
    })
  }
}
