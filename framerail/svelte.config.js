import adapter from "@sveltejs/adapter-auto"
import preprocess from "svelte-preprocess"
import switchCase from "svelte-switch-case"

/** @type {import("@sveltejs/kit").Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: [
    preprocess(),
    switchCase()
  ],

  kit: {
    adapter: adapter()
  },

  compilerOptions: {
    enableSourcemap: true,
    immutable: true
  }
}

export default config
