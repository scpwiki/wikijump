import svelte from "@sveltejs/vite-plugin-svelte"

import sveltePreprocess from "svelte-preprocess"
import { typescript } from "svelte-preprocess-esbuild"

import workerPlugin from "../../scripts/vite-plugin-bundled-worker.js"

/** @type {import('vite').UserConfig} */
const config = {
  publicDir: "../public",
  root: "./src",
  resolve: {
    dedupe: ["@codemirror/state", "@codemirror/view", "@codemirror/language"]
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
    workerPlugin(),
    svelte({
      // render typescript using esbuild rather than tsc
      preprocess: [typescript(), sveltePreprocess({ typescript: false })]
    })
  ]
}

export default config
