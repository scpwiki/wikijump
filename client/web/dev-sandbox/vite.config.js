import svelte from "@sveltejs/vite-plugin-svelte"
import sveltePreprocess from "svelte-preprocess"
import workerPlugin from "../../scripts/vite-plugin-bundled-worker.js"
import tomlPlugin from "../../scripts/vite-plugin-toml.js"

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
    tomlPlugin(),
    svelte({
      preprocess: [
        sveltePreprocess({
          sass: { sourceMapEmbed: true, sourceMapContents: true, sourceMap: true }
        })
      ]
    })
  ]
}

export default config
