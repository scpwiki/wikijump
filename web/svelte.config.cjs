const path = require("path")
const mergeQueries = require("postcss-merge-queries")
const autoprefixer = require("autoprefixer")
const { globalStyle, postcss, scss, typescript } = require("svelte-preprocess")

let abstracts = path.resolve(__dirname, "resources/css/abstracts.scss")
// removing the drive letter and normalizing the slashes is required for Windows,
// which is absurd but that's just what Sass wants I guess
abstracts = abstracts.replace(/^[A-Z]:/, "")
abstracts = abstracts.replaceAll("\\", "/")

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
