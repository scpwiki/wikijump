const fs = require("fs")
const path = require("path")
const { svelte } = require("@sveltejs/vite-plugin-svelte")
const toml = require("@ltd/j-toml")
const yaml = require("js-yaml")
const baseSvelteConfig = require("../svelte.config.cjs")

const ROOT = path.resolve(__dirname, "../")

const entrypoints = fs
  .readdirSync(`${ROOT}/resources/scripts`)
  .filter(ent => fs.statSync(`resources/scripts/${ent}`).isFile())
  .map(file => path.resolve(`${ROOT}/resources/scripts/${file}`))

const modules = fs
  .readdirSync(`${ROOT}/modules`)
  .filter(dir => fs.statSync(`${ROOT}/modules/${dir}`).isDirectory())
  .map(dir => `@wikijump/${dir}`)

const SVELTE_OPTIONS = {
  onwarn: (warning, handler) => {
    if (warning.code === "unused-export-let") return
    if (handler) handler(warning)
  },
  ...baseSvelteConfig
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

const PHP_CONFIG = {
  "entrypoints": entrypoints,
  "ignore_patterns": ["/\\.d\\.ts$/"],
  "aliases": { "@": "resources", "@root": "../" },
  "public_directory": "resources/static",
  "ping_timeout": 1,
  "ping_url": "http://host.docker.internal:3000",
  "build_path": "files--built",
  "dev_url": "",
  "commands": []
}

// because esbuild rips out comments, esbuild will not preserve
// the comments we need for ignoring lines
// however, it does preserve legal comments, /*! or //!
// but c8 doesn't recognize those!
// so we have to transform those back into normal comments
// before we let c8 parse them
const RollupFixLegalCommentsPlugin = {
  transform(code, id) {
    // use two spaces so we don't change the length of the document
    code = code.replaceAll("/*! c8", "/*  c8")
    // null map informs rollup to preserve the current sourcemap
    return { code, map: null }
  }
}

// TODO: optimize imported FTL files by removing blank lines and comments
/** Handles importing of Fluent FTL files. */
const PluginFTL = transformerPlugin("ftl", /\.ftl$/, src => jsonStringify(src))

/** Handles importing of TOML files. */
const PluginTOML = transformerPlugin("toml", /\.toml$/, src => {
  const obj = toml.parse(src, 1.0, "\n", false, { order: true, null: true })
  return jsonStringify(obj)
})

const PluginYAML = transformerPlugin("yaml", /\.ya?ml$/, src => {
  const obj = yaml.load(src)
  if (!obj || typeof obj !== "object") throw new Error("Invalid YAML provided.")
  return jsonStringify(obj)
})

/** @returns {import("vite").UserConfig} */
const BaseConfig = () => ({
  server: {
    port: 3000,
    strictPort: true,
    fs: { strict: false },
    // listen on all addresses (fixes Docker issue on Linux)
    host: "0.0.0.0"
  },

  resolve: {
    alias: [
      { find: "@", replacement: `${ROOT}/resources` },
      { find: "@root", replacement: path.resolve(ROOT, "../") }
    ]
  },

  clearScreen: false,

  css: {
    preprocessorOptions: {
      scss: { sourceMap: true }
    }
  },

  build: {
    target: "esnext",
    sourcemap: true,
    reportCompressedSize: false,
    cssCodeSplit: true,
    rollupOptions: {
      input: entrypoints
    }
  },

  optimizeDeps: {
    entries: [...entrypoints, "./modules/*/src/**/*.{svelte,js,jsx,ts,tsx}"],
    esbuildOptions: { tsconfig: `${ROOT}/tsconfig.json` },
    exclude: modules
  },

  plugins: [PluginFTL, PluginTOML, PluginYAML, svelte(SVELTE_OPTIONS)]
})

/** @returns {import("vite").UserConfig} */
const TestConfig = () => ({
  ...BaseConfig(),

  publicDir: false,
  root: ROOT,

  server: {
    middlewareMode: "ssr",
    hmr: false,
    fs: { strict: false }
  },

  build: {
    assetsDir: "./",
    sourcemap: "inline",
    target: "esnext",
    minify: false,
    brotliSize: false,
    cssCodeSplit: false,
    rollupOptions: {
      plugins: [RollupFixLegalCommentsPlugin],
      treeshake: false
    }
  },

  optimizeDeps: {
    entries: "modules/*/{tests,src}/**/*.{svelte,js,jsx,ts,tsx}",
    include: ["@esm-bundle/chai", "@testing-library/svelte"],
    exclude: modules
  },

  plugins: [PluginFTL, PluginTOML, PluginYAML, svelte(SVELTE_TEST_OPTIONS)]
})

module.exports = { PHP_CONFIG, BaseConfig, TestConfig }

// internal utility functions

function jsonStringify(src) {
  // stringify twice so that we get an escaped string
  // e.g. "{\"foo\": \"bar\"}"
  return `export default JSON.parse(${JSON.stringify(JSON.stringify(src))})`
}

function transformerPlugin(name, regex, transformer) {
  return {
    name,
    transform(src, id) {
      if (regex.test(id)) {
        return { code: transformer(src), map: { mappings: "" } }
      }
    }
  }
}
