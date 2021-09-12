import { svelte } from "@sveltejs/vite-plugin-svelte"
import sveltePreprocess from "svelte-preprocess"
import tsconfigPaths from "vite-tsconfig-paths"
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

  build: {
    outDir: "../dist",
    emptyOutDir: true,
    assetsDir: "static/assets",
    manifest: true,
    sourcemap: true,
    target: "esnext",
    minify: "esbuild",
    brotliSize: false,
    cssCodeSplit: false
  },

  optimizeDeps: {
    entries: ["index.html", "../../modules/*/src/**/*.{svelte,js,jsx,ts,tsx}"],
    exclude: ["wj-state", "ftml-wasm-worker"]
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
    tomlPlugin(),
    yamlPlugin(),
    svelte({
      onwarn: (warning, handler) => {
        if (warning.code === "unused-export-let") return
        if (handler) handler(warning)
      },
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
