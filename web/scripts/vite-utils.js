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
  .filter(dir => !dir.startsWith("_"))
  .map(dir => `@wikijump/${dir}`)

const SVELTE_OPTIONS = {
  ...baseSvelteConfig,
  onwarn: (warning, handler) => {
    if (warning.code === "unused-export-let") return
    if (handler) handler(warning)
  },
  hot: !process.env.VITEST,
  experimental: {
    generateMissingPreprocessorSourcemaps: true
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

// TODO: optimize imported FTL files by removing blank lines and comments
/** Handles importing of Fluent FTL files. */
const PluginFTL = transformerPlugin("ftl", /\.ftl$/, src => jsonStringify(src))

/** Handles importing of TOML files. */
const PluginTOML = transformerPlugin("toml", /\.toml$/, src => {
  const obj = toml.parse(src, 1.0, "\n", false, { order: true, null: true })
  return jsonStringify(obj)
})

/** Handles importing of YAML files. */
const PluginYAML = transformerPlugin("yaml", /\.ya?ml$/, src => {
  const obj = yaml.load(src)
  if (!obj || typeof obj !== "object") throw new Error("Invalid YAML provided.")
  return jsonStringify(obj)
})

function manualChunks(id) {
  if (id.includes("node_modules/ziggy")) return "ziggy"
  if (id.includes("node_modules/svelte/store")) return "svelte-store"
  if (id.includes("node_modules/svelte")) return "svelte"
  if (id.includes("vendor/prism.js")) return "prism"
  if (id.includes("modules/codemirror/cm.ts")) return "codemirror"
}

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
    ],
    dedupe: ["svelte", "svelte/store", "svelte/internal"]
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
      input: entrypoints,
      output: { manualChunks }
    }
  },

  optimizeDeps: {
    entries: [...entrypoints, "./modules/*/src/**/*.{svelte,js,jsx,ts,tsx}"],
    esbuildOptions: { tsconfig: `${ROOT}/tsconfig.json` },
    exclude: modules
  },

  plugins: [PluginFTL, PluginTOML, PluginYAML, svelte(SVELTE_OPTIONS)],

  // Vitest

  test: {
    root: ROOT,
    isolate: false,
    environment: "jsdom",
    include: ["./modules/**/tests/**/*.{js,ts}"],
    setupFiles: ["./scripts/test-setup.js"],
    deps: {
      inline: ["threads", "observable-fns"]
    }
  }
})

module.exports = { PHP_CONFIG, BaseConfig }

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
