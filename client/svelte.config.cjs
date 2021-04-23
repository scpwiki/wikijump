// I'm fairly sure this file doesn't actually do anything.
// I think everything is handled by esbuild or some other builder with its own config,
// so basically just ignore this file.
//
// It has to be kept because if it wasn't, editors wouldn't know to
// check for Svelte components.

const sveltePreprocess = require("svelte-preprocess")

module.exports = {
  preprocess: sveltePreprocess()
}
