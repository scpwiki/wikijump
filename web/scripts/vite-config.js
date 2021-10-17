const { svelte } = require("@sveltejs/vite-plugin-svelte")
const sveltePreprocess = require("svelte-preprocess")
const tomlPlugin = require("./vite-plugin-toml.js")
const yamlPlugin = require("./vite-plugin-yaml.js")
const path = require("path")
const fs = require("fs")

const ROOT = path.resolve(__dirname, "../")

/** @type import("sass").Options */
const SASS_OPTIONS = {
  sourceMapEmbed: true,
  sourceMapContents: true,
  sourceMap: true
}

const SVELTE_OPTIONS = {
  onwarn: (warning, handler) => {
    if (warning.code === "unused-export-let") return
    if (handler) handler(warning)
  },
  preprocess: [sveltePreprocess({ sourceMap: true })],
  compilerOptions: {
    enableSourcemap: true,
    immutable: true
  }
}

const SVELTE_TEST_OPTIONS = {
  ...SVELTE_OPTIONS,
  emitCss: false,
  compilerOptions: {
    enableSourcemap: true,
    immutable: true,
    cssHash: () => "svelte"
  }
}

const BUILD_TEST_OPTIONS = {
  assetsDir: "./",
  sourcemap: "inline",
  target: "esnext",
  minify: false,
  brotliSize: false,
  cssCodeSplit: false,

  rollupOptions: {
    plugins: [
      // because esbuild rips out comments, esbuild will not preserve
      // the comments we need for ignoring lines
      // however, it does preserve legal comments, /*! or //!
      // but c8 doesn't recognize those!
      // so we have to transform those back into normal comments
      // before we let c8 parse them
      {
        transform(code, id) {
          // use two spaces so we don't change the length of the document
          code = code.replaceAll("/*! c8", "/*  c8")
          // null map informs rollup to preserve the current sourcemap
          return { code, map: null }
        }
      }
    ],

    treeshake: false
  }
}

const modules = fs
  .readdirSync(`${ROOT}/modules`)
  .filter(dir => fs.statSync(`${ROOT}/modules/${dir}`).isDirectory())
  .map(dir => `@wikijump/${dir}`)

/** @returns {import("vite").UserConfig} */
const getConfig = (test = false) => ({
  publicDir: test ? false : "../public",
  root: test ? ROOT : "./src",

  resolve: {
    // wikijump repo root, this is also in tsconfig.json
    alias: [{ find: "@root", replacement: path.resolve(ROOT, "../") }]
  },

  server: {
    proxy: {
      "/api--v1": {
        target: "http://localhost:3500"
      }
    }
  },

  build: test
    ? BUILD_TEST_OPTIONS
    : {
        outDir: "../dist",
        emptyOutDir: true,
        assetsDir: "files--common/assets",
        manifest: true,
        sourcemap: true,
        target: "esnext",
        minify: "esbuild",
        brotliSize: false,
        cssCodeSplit: false
      },

  css: {
    preprocessorOptions: {
      scss: SASS_OPTIONS
    }
  },

  optimizeDeps: test
    ? {
        entries: "modules/*/{tests,src}/**/*.{svelte,js,jsx,ts,tsx}",
        include: ["@esm-bundle/chai", "@testing-library/svelte"],
        exclude: modules
      }
    : {
        entries: ["index.html", "../../../modules/*/src/**/*.{svelte,js,jsx,ts,tsx}"],
        esbuildOptions: { tsconfig: `${ROOT}/tsconfig.json` },
        exclude: modules
      },

  plugins: [
    tomlPlugin(),
    yamlPlugin(),
    svelte(test ? SVELTE_TEST_OPTIONS : SVELTE_OPTIONS)
  ]
})

module.exports = { SASS_OPTIONS, SVELTE_OPTIONS, getConfig }
