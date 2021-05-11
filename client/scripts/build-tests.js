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
            sourceMap: true,
            scss: { sourceMapEmbed: true, sourceMapContents: true, sourceMap: true }
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
          // because esbuild rips out comments, esbuild will not preserve
          // the comments we need for ignoring lines
          // however, it does preserve legal comments, /*! or //!
          // but the istanbul plugin doesn't recognize those!
          // so we have to transform those back into normal comments
          // before we let istanbul parse them
          {
            transform(code, id) {
              // use two spaces so we don't change the length of the document
              code = code.replaceAll("/*! istanbul", "/*  istanbul")
              // null map informs rollup to preserve the current sourcemap
              return { code, map: null }
            }
          },
          istanbul({
            babelrc: false,
            babelHelpers: "bundled",
            sourceType: "module",
            extensions: [".ts", ".tsx", ".svelte"],
            // speeds things up or lowers file size
            skipPreflightCheck: true,
            compact: true,
            retainLines: true
          })
        ],

        output: {
          interop: "esModule",
          sourcemapExcludeSources: true
        },

        treeshake: false,
        external: EXTERNAL
      }
    }
  })

  console.log("[tests] Megabundle compile complete.")
}

const EXTERNAL = [
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
