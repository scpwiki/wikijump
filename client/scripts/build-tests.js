const path = require("path")
const fs = require("fs-extra")
const vite = require("vite")
const globby = require("globby")
const svelte = require("@sveltejs/vite-plugin-svelte")
const sveltePreprocess = require("svelte-preprocess")
const workerPlugin = require("./vite-plugin-bundled-worker.js")
const tomlPlugin = require("./vite-plugin-toml.js")
const yamlPlugin = require("./vite-plugin-yaml.js")
const istanbul = require("./rollup-plugin-istanbul")

// make sure we're at root
process.chdir(path.resolve(__dirname, "../"))
const DIR = path.resolve(process.cwd(), "tests-dist")
const EXCLUDE = [/\.worker\.ts$/]

build()

// ----

async function collectModules() {
  const files = await globby("modules/*/src/**/*.ts", { ignore: ["*.d.ts"] })
  let entrypoint = "import '../../scripts/tests-shim.js';\n"
  for (const file of files) {
    if (EXCLUDE.some(regex => regex.test(file))) continue
    entrypoint += `import ${JSON.stringify(`../${file}`)};\n`
  }
  return entrypoint
}

async function collectTests() {
  const files = await globby("modules/*/tests/*.ts", { ignore: ["*.d.ts"] })
  let entrypoint = "import '../../scripts/tests-shim.js';\n"
  for (const file of files) {
    entrypoint += `import ${JSON.stringify(`../${file}`)};\n`
  }
  return entrypoint
}

async function generateMegaBundleEntrypoint() {
  const moduleImport = await collectModules()
  const testImport = await collectTests()

  const modules = path.resolve(DIR, "megabundle-modules.ts")
  const tests = path.resolve(DIR, "megabundle-tests.ts")
  const entrypoint = path.resolve(DIR, "test-megabundle.ts")

  // prettier-ignore
  const src =
    `import "./megabundle-tests";\n` +
    `import "./megabundle-modules";`

  await Promise.all([
    fs.outputFile(modules, moduleImport),
    fs.outputFile(tests, testImport),
    fs.outputFile(entrypoint, src)
  ])

  return { entrypoint, modules, tests }
}

async function build() {
  console.log(`[tests] Compiling test megabundle...`)

  const { entrypoint, modules, tests } = await generateMegaBundleEntrypoint()

  await vite.build({
    root: DIR,

    define: {
      "import.meta.url": '"file://test-megabundle"'
    },

    plugins: [
      workerPlugin(),
      tomlPlugin(),
      yamlPlugin(),
      svelte({
        emitCss: false,
        compilerOptions: { cssHash: () => "svelte" },
        preprocess: [sveltePreprocess({ sourceMap: true })]
      })
    ],

    build: {
      assetsDir: "./",
      sourcemap: true,
      target: "esnext",
      minify: false,
      brotliSize: false,
      cssCodeSplit: false,

      rollupOptions: {
        input: [entrypoint],
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
            compact: true
          })
        ],

        output: {
          format: "cjs",
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
  // externalize this so that it gets required first
  // this has required polyfills, so it needs to run first
  "../../scripts/tests-shim.js",
  /uvu/,
  // codemirror has CJS exports now, so this is ok
  /^@codemirror/,
  "lezer-tree",
  // breaks things if included
  "threads",
  // large CJS-compatible dependencies (so we'll exclude them)
  /happy-dom/,
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
