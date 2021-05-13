import svelte from "@sveltejs/vite-plugin-svelte"
import sveltePreprocess from "svelte-preprocess"
import workerPlugin from "../client/scripts/vite-plugin-bundled-worker.js"
import tomlPlugin from "../client/scripts/vite-plugin-toml.js"

import { defineConfig } from "laravel-vite"

/** @type import("sass").Options */
const SASS_OPTIONS = {
  sourceMapEmbed: true,
  sourceMapContents: true,
  sourceMap: true
}

/** @type {import('vite').UserConfig} */
const config = {
  build: {
    sourcemap: true,
    target: "esnext",
    minify: "esbuild",
    brotliSize: false
  },

  css: {
    preprocessorOptions: {
      scss: SASS_OPTIONS
    }
  },

  plugins: [
    workerPlugin(),
    tomlPlugin(),
    svelte({
      preprocess: [
        sveltePreprocess({
          sourceMap: true,
          scss: SASS_OPTIONS
        })
      ]
    })
  ]
}

export default defineConfig(config)
