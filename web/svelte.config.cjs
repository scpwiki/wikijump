const sveltePreprocess = require("svelte-preprocess")

module.exports = {
  preprocess: [sveltePreprocess({ sourceMap: true })],
  compilerOptions: {
    enableSourcemap: true,
    immutable: true
  }
}
