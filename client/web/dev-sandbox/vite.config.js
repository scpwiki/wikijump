import svelte from "@sveltejs/vite-plugin-svelte"

import sveltePreprocess from "svelte-preprocess"
import { typescript } from "svelte-preprocess-esbuild"

/** @type {import('vite').UserConfig} */
const config = {
  publicDir: "../public",
  root: "./src",
  resolve: {
    dedupe: ["@codemirror/state"]
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    assetsDir: "static/assets",
    manifest: true,
    sourcemap: true,
    target: "esnext",
    minify: "esbuild",
    brotliSize: false
  },
  plugins: [
    svelte({
      // render typescript using esbuild rather than tsc
      preprocess: [typescript(), sveltePreprocess({ typescript: false })]
    })
  ]
}

export default config
