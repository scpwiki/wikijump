const esbuild = require("esbuild")

const fileRegex = /\?bundled-worker$/

module.exports = function viteWorkerPlugin() {
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
          minify: true,
          treeShaking: true,
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
          if (file.path.endsWith(".map")) map = file.text
          if (file.path.endsWith(".js")) code = file.text
        })

        return `export default atob(\`${Buffer.from(code).toString("base64")}\`);`
      }
    }
  }

  return plugin
}
