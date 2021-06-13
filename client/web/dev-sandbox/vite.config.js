import { svelte } from "@sveltejs/vite-plugin-svelte"
import sveltePreprocess from "svelte-preprocess"
import tsconfigPaths from "vite-tsconfig-paths"
import workerPlugin from "../../scripts/vite-plugin-bundled-worker.js"
import tomlPlugin from "../../scripts/vite-plugin-toml.js"
import yamlPlugin from "../../scripts/vite-plugin-yaml.js"

/** @type import("sass").Options */
const SASS_OPTIONS = {
  sourceMapEmbed: true,
  sourceMapContents: true,
  sourceMap: true
}

/** @type {import("vite").UserConfig} */
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
    brotliSize: false,
    cssCodeSplit: false,
    cleanCssOptions: {
      sourceMap: true
    }
  },

  css: {
    preprocessorOptions: {
      scss: SASS_OPTIONS
    }
  },

  plugins: [
    tsconfigPaths({
      projects: ["../../../"],
      loose: true
    }),
    workerPlugin(),
    tomlPlugin(),
    yamlPlugin(),
    svelte({
      hot: false,
      preprocess: [
        sveltePreprocess({
          sourceMap: true,
          scss: {
            ...SASS_OPTIONS,
            renderSync: true
          }
        })
      ]
    })
  ]
}

export default config
