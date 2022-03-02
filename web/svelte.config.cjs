const path = require("path")
const mergeQueries = require("postcss-merge-queries")
const autoprefixer = require("autoprefixer")
const { globalStyle, postcss, scss, typescript } = require("svelte-preprocess")

const abstracts = path.resolve(__dirname, "resources/css/abstracts.scss")

module.exports = {
  preprocess: [
    scss({
      // faster for Dart Sass, which we're using
      renderSync: true,
      // automatically imports the abstracts file (mixins, variables, etc)
      prependData: `@import "${abstracts}";\n`
    }),
    postcss({
      plugins: [autoprefixer(), mergeQueries({ sort: true })]
    }),
    globalStyle(),
    typescript()
  ],
  /** @type import("svelte/types/compiler/interfaces").CompileOptions */
  compilerOptions: {
    enableSourcemap: true,
    immutable: true
  }
}
