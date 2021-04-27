// I'm fairly sure this file doesn't actually do anything.
// I think everything is handled by esbuild or some other builder with its own config,
// so basically just ignore this file.
//
// It has to be kept because if it wasn't, editors wouldn't know to
// check for Svelte components.

const sveltePreprocess = require("svelte-preprocess")
const { typescript } = require("svelte-preprocess-esbuild")

module.exports = {
  // render typescript using esbuild rather than tsc
  preprocess: [typescript(), sveltePreprocess({ typescript: false })]
}
