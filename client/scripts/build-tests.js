const path = require("path")
const fs = require("fs/promises")

// make sure we're at root
process.chdir(path.resolve(__dirname, "../"))

const esbuild = require("esbuild")
const globby = require("globby")

const sveltePreprocess = require("svelte-preprocess")
const { typescript } = require("svelte-preprocess-esbuild")
const sveltePlugin = require("esbuild-svelte")

build()

async function generateMegaBundleEntrypoint() {
  const sourceFiles = await globby("modules/*/src/**/*.ts")
  const testFiles = await globby("modules/*/tests/*.ts")

  // compile the entire monorepo into a dynamic import that won't get executed
  // we do this so that the _entire monorepo_ gets bundled, but won't break anything
  // this makes sure that code coverage and the like works
  //
  // the expression chosen here is designed to _always_ be false, but also
  // not obviously false to an optimizer - this is so this section isn't
  // tree-shaked or dead-coded out of the bundle (ruining the whole point of this)
  let entrypoint = "if (!globalThis && 'obviously_not_undefined' === undefined) {\n"
  for (const file of sourceFiles) {
    entrypoint += `  import(${JSON.stringify(`./${file}`)});\n`
  }
  entrypoint += "}\n"

  // actually import the test files
  for (const file of testFiles) {
    entrypoint += `import ${JSON.stringify(`./${file}`)};\n`
  }

  return entrypoint
}

async function build() {
  console.log(`[tests] Compiling test megabundle...`)

  const entrypoint = await generateMegaBundleEntrypoint()

  const result = await esbuild.build({
    stdin: {
      contents: entrypoint,
      resolveDir: process.cwd(),
      sourcefile: "test-megabundle.ts",
      loader: "ts"
    },
    // add other modules here if needed
    external: [
      "uvu",
      // codemirror has CJS exports now, so this is ok
      "@codemirror/*",
      "lezer-tree",
      // breaks things if included
      "threads",
      // large CJS-compatible dependencies (so we'll exclude them)
      "@happy-dom/*",
      "globby",
      "fs-extra",
      "@ltd/j-toml",
      "svelte"
    ],
    inject: ["./scripts/tests-shim.js"],
    outfile: "tests-dist/test-megabundle.cjs",
    bundle: true,
    treeShaking: true,
    minify: false,
    format: "cjs",
    platform: "node",
    sourcemap: true,
    sourcesContent: false,
    metafile: true,
    loader: { ".wasm": "file", ".toml": "text", ".worker.ts": "text" },
    logLevel: "error",
    plugins: [
      sveltePlugin({
        compileOptions: { css: true, cssHash: () => "svelte" },
        // render typescript using esbuild rather than tsc
        preprocess: [typescript(), sveltePreprocess({ typescript: false })]
      })
    ]
  })

  // write metafile for examination purposes
  // e.g. use with https://www.bundle-buddy.com
  await fs.writeFile("tests-dist/meta.json", JSON.stringify(result.metafile))

  console.log("[tests] Megabundle compile complete.")
}
