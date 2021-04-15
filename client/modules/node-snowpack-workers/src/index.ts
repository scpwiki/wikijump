import esbuild from "esbuild"

export default function () {
  return {
    name: "bundle-workers",

    resolve: {
      input: [".worker.ts"],
      output: [".worker.js"]
    },

    async load({ filePath = "" }) {
      const result = await esbuild.build({
        entryPoints: [filePath],
        bundle: true,
        treeShaking: true,
        // sourcemap: true,
        outdir: "./",
        outbase: "./",
        format: "esm",
        platform: "browser",
        write: false
      })
      const ret = { code: result.outputFiles[0].text }
      return { ".worker.js": ret }
    }
  }
}
