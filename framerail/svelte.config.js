import adapter from "@sveltejs/adapter-node"
import preprocess from "svelte-preprocess"

/** @type {import("@sveltejs/kit").Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    adapter: adapter(),
    csrf: {
      // Allow flexible hosts on local, since we don't have real DNS
      checkOrigin: process.env.FRAMERAIL_ENV !== "local"
    }
  },

  compilerOptions: {
    enableSourcemap: true,
    immutable: true
  }
}

export default config
