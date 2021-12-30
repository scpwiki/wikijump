const path = require("path")

/** @type import("svelte-preprocess").default */
const preprocess = require("svelte-preprocess")

const abstracts = path.resolve(__dirname, "resources/css/abstracts.scss")

module.exports = {
  preprocess: [
    preprocess({
      sourceMap: true,
      scss: {
        // faster for Dart Sass, which we're using
        renderSync: true,
        // automatically imports the abstracts file (mixins, variables, etc)
        prependData: `@import "${abstracts}";\n`
      }
    })
  ],
  /** @type import("svelte/types/compiler/interfaces").CompileOptions */
  compilerOptions: {
    enableSourcemap: true,
    immutable: true
  }
}
