const path = require("path")
const esbuild = require("esbuild")
const globby = require("globby")

const sveltePreprocess = require("svelte-preprocess")
const { typescript } = require("svelte-preprocess-esbuild")
const sveltePlugin = require("esbuild-svelte")

build()

async function getTests() {
  const testFiles = await globby(["modules/*/tests/*.ts", "components/tests/*.ts"], {
    absolute: true
  })
  const tests = {}
  testFiles.forEach(file => {
    const moduleName = path.basename(path.resolve(path.dirname(file), "../"))
    const fileNameNoExt = path.basename(file, ".ts")
    tests[`${moduleName}_${fileNameNoExt}`] = file
  })
  return tests
}

async function build() {
  const tests = await getTests()
  console.log(`[tests] Compiling ${Object.keys(tests).length} files...`)
  await esbuild.build({
    // add other modules here if needed
    external: ["jsdom"],
    inject: ["./scripts/tests-shim.js"],
    outdir: "tests-dist",
    entryPoints: tests,
    bundle: true,
    treeShaking: true,
    minify: false,
    format: "cjs",
    platform: "node",
    sourcemap: false,
    outExtension: { ".js": ".cjs" },
    loader: { ".wasm": "file", ".yaml": "text" },
    plugins: [
      sveltePlugin({
        compileOptions: { css: true, cssHash: () => "svelte" },
        // render typescript using esbuild rather than tsc
        preprocess: [typescript(), sveltePreprocess({ typescript: false })]
      })
    ]
  })
}
