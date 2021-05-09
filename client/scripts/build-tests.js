const path = require("path")
const fs = require("fs-extra")

// make sure we're at root
process.chdir(path.resolve(__dirname, "../"))

const vite = require("vite")
const globby = require("globby")

const svelte = require("@sveltejs/vite-plugin-svelte")
const sveltePreprocess = require("svelte-preprocess")
const workerPlugin = require("./vite-plugin-bundled-worker.js")
const tomlPlugin = require("./vite-plugin-toml.js")

const istanbul = require("./rollup-plugin-istanbul")

build()

const EXCLUDE = [/ftml-wasm-worker/]

async function generateMegaBundleEntrypoint() {
  const sourceFiles = await globby("modules/*/src/**/*.ts", { ignore: ["*.d.ts"] })
  const testFiles = await globby("modules/*/tests/*.ts", { ignore: ["*.d.ts"] })

  let entrypoint = "import '../scripts/tests-shim.js';\n"

  for (const file of sourceFiles) {
    if (EXCLUDE.some(regex => regex.test(file))) continue
    entrypoint += `import ${JSON.stringify(`../${file}`)};\n`
  }

  // actually import the test files
  for (const file of testFiles) {
    entrypoint += `import ${JSON.stringify(`../${file}`)};\n`
  }

  return entrypoint
}

async function build() {
  console.log(`[tests] Compiling test megabundle...`)

  const entrypoint = await generateMegaBundleEntrypoint()
  const dir = path.resolve(process.cwd(), "tests-dist")
  const file = path.resolve(dir, "test-megabundle.ts")

  await fs.outputFile(file, entrypoint)

  await vite.build({
    root: dir,

    define: {
      "import.meta.url": '"file://test-megabundle"'
    },

    css: {
      preprocessorOptions: {
        scss: { sourceMapEmbed: true, sourceMapContents: true, sourceMap: true }
      }
    },

    plugins: [
      workerPlugin(),
      tomlPlugin(),
      svelte({
        compilerOptions: { cssHash: () => "svelte" },
        preprocess: [
          sveltePreprocess({
            sass: { sourceMapEmbed: true, sourceMapContents: true, sourceMap: true }
          })
        ]
      })
    ],

    build: {
      lib: {
        entry: file,
        name: "test-megabundle",
        fileName: "test-megabundle",
        formats: ["cjs"]
      },

      sourcemap: true,
      target: "esnext",
      minify: false,
      brotliSize: false,
      cssCodeSplit: false,

      rollupOptions: {
        plugins: [
          istanbul({
            babelrc: false,
            babelHelpers: "bundled",
            extensions: [".js", ".jsx", ".es6", ".es", ".mjs", ".ts", ".tsx", ".svelte"]
          })
        ],
        treeshake: false,
        output: {
          interop: "esModule",
          compact: true,
          preferConst: true,
          sourcemapExcludeSources: true
        },
        external: [
          /uvu/,
          // /vendor\/prism\.js/,
          // codemirror has CJS exports now, so this is ok
          /^@codemirror/,
          "lezer-tree",
          // breaks things if included
          "threads",
          // large CJS-compatible dependencies (so we'll exclude them)
          /^@happy-dom/,
          "globby",
          "fs-extra",
          "fs/promises",
          "@ltd/j-toml",
          "diff",
          // from esbuild source:
          // node builtins
          "assert",
          "async_hooks",
          "buffer",
          "child_process",
          "cluster",
          "console",
          "constants",
          "crypto",
          "dgram",
          "dns",
          "domain",
          "events",
          "fs",
          "http",
          "http2",
          "https",
          "inspector",
          "module",
          "net",
          "os",
          "path",
          "perf_hooks",
          "process",
          "punycode",
          "querystring",
          "readline",
          "repl",
          "stream",
          "string_decoder",
          "sys",
          "timers",
          "tls",
          "trace_events",
          "tty",
          "url",
          "util",
          "v8",
          "vm",
          "worker_threads",
          "zlib"
        ]
      }
    }
  })

  console.log("[tests] Megabundle compile complete.")
}
