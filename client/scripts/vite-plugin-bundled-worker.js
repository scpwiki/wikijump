const esbuild = require("esbuild")

const fileRegex = /\?bundled-worker$/

// TODO: remove when testing is switched over to being done on browsers
const resolveSpellchecker = {
  name: "fixSpellchecker",
  setup(build) {
    build.onResolve({ filter: /spellchecker-wasm\/lib\/browser\/index/ }, args => {
      return { path: "spellchecker-wasm/lib/nodejs/index", external: true }
    })
  }
}

module.exports = function viteWorkerPlugin(cjs = false) {
  /** @type import("vite").Plugin */
  const plugin = {
    name: "bundle-workers",

    async load(id) {
      if (fileRegex.test(id)) {
        // build the worker under IIFE so that it has no exports, no imports
        // should be 100% web-worker compatible
        const built = await esbuild.build({
          entryPoints: [id],
          bundle: true,
          minifySyntax: true,
          minifyIdentifiers: false,
          minifyWhitespace: true,
          treeShaking: true,
          outdir: "./",
          outbase: "./",
          write: false,
          define: {
            "window": "globalThis",
            "import.meta.url": '""'
          },
          ...(cjs
            ? {
                format: "cjs",
                platform: "node",
                external: ["threads"],
                plugins: [resolveSpellchecker]
              }
            : {
                format: "iife",
                platform: "browser"
              })
        })

        let code, map
        built.outputFiles.forEach(file => {
          if (file.path.endsWith(".map")) map = file.text
          if (file.path.endsWith(".js")) code = file.text
        })

        return {
          code: `export default ${JSON.stringify(code)};`,
          map: { mappings: "" }
        }
      }
    }
  }

  return plugin
}
