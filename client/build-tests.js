const esbuild = require("esbuild")
const { readdirSync } = require("fs")
const { basename } = require("path")
const { nodeExternalsPlugin } = require("esbuild-node-externals")
const { performance } = require("perf_hooks")

const start = performance.now()

const dir = process.cwd()
const name = basename(dir)
const testDir = `${dir}/tests`

function filesOf(source) {
  return readdirSync(source, { withFileTypes: true })
    .filter(dirent => dirent.isFile())
    .map(dirent => dirent.name)
}

const tests = filesOf(testDir).map(name => `./tests/${name}`)

if (tests.length) {
  const build = esbuild.build({
    absWorkingDir: dir,
    entryPoints: [...tests],
    outdir: "./tests/dist",
    bundle: true,
    platform: "node",
    format: "esm",
    plugins: [nodeExternalsPlugin()]
  })

  build.then(() =>
    console.log(`[${name}] Tests compiled. (${(performance.now() - start).toFixed(2)}ms)`)
  )
} else {
  console.log(`[${name}] No tests, skipping.`)
}
