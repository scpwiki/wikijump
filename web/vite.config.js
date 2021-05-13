const svelte = require("@sveltejs/vite-plugin-svelte")
const sveltePreprocess = require("svelte-preprocess")
const workerPlugin = require("../client/scripts/vite-plugin-bundled-worker.js")
const tomlPlugin = require("../client/scripts/vite-plugin-toml.js")

const { defineConfig } = require("laravel-vite")

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

module.exports = defineConfig(config)
