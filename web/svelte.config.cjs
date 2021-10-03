const sveltePreprocess = require("svelte-preprocess")

module.exports = {
  preprocess: [
    sveltePreprocess({
      sass: { sourceMapEmbed: true, sourceMapContents: true, sourceMap: true }
    })
  ]
}
