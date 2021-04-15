const { build, cliopts, basename, stdoutStyle: styl, fmtDuration } = require("estrella")
const { nodeExternalsPlugin } = require("esbuild-node-externals")
const { readdirSync, realpathSync } = require("fs")
const { performance } = require("perf_hooks")
const browserslist = require("browserslist")

const [opts] = cliopts.parse(["self", "Compiles the calling module package only."])

const dev = Boolean(cliopts.watch)
const self = opts.self ?? false

// demangle browserslist versions
// prettier-ignore
const DEMANGLE_TARGET_REGEX =
  /^(chrome|and_(?:ff|chr)|edge|firefox|ios_saf|safari)\s+([\d.]+)/
const targets = new Set()
for (const target of browserslist()) {
  const result = DEMANGLE_TARGET_REGEX.exec(target)
  if (!result) continue
  let [, browser, version] = result
  // rewrite mobile browser versions into what esbuild understands
  if (browser === "ios_saf") browser = "ios"
  else if (browser === "and_chr") browser = "chrome"
  else if (browser === "and_ff") browser = "firefox"
  targets.add(browser + version)
}

const SETTINGS_COMMON = {
  // esbuild settings
  outdir: "dist",
  // minify: true // estrella implicitly toggles minify depending on the `--debug` flag
  bundle: true,
  treeShaking: true,
  splitting: true,
  platform: "browser",
  format: "esm",
  sourcemap: true,
  sourcesContent: true,
  target: [...targets],

  // estrella settings
  tslint: false, // disables estrella's built-in typechecker
  quiet: true // silences estrella's logging, as we use our own logging
}

function directoriesOf(source) {
  return readdirSync(source, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name)
}

function buildModule(name) {
  let dir, index, package

  if (self) {
    dir = process.cwd()
    index = "./src/index.ts"
    package = "./package.json"
  } else {
    dir = realpathSync(`./modules/${name}`)
    index = `${dir}/src/index.ts`
    package = `${dir}/package.json`
  }

  name = basename(dir)

  if (!dev) console.log(styl.blue(`[${name}]`), "Building!")
  let start = performance.now()

  build({
    ...SETTINGS_COMMON,

    // esbuild settings
    absWorkingDir: dir,
    tsconfig: package,
    plugins: [nodeExternalsPlugin({ packagePath: package })],

    // estrella settings
    entry: index,
    cwd: dir,
    // this property uses the Chokidar watch settings interface
    // we need to make sure that Chokidar is watching the module directory
    watch: dev && { cwd: dir },

    // logging handlers

    onStart(_, changed) {
      if (dev && changed && changed.length) {
        start = performance.now()
        console.log(
          styl.blue(`[${name}]`),
          "Rebuilding!",
          styl.orange(`[${changed.join(", ")}]`)
        )
      }
    },

    onEnd() {
      const elapsed = fmtDuration(performance.now() - start)
      console.log(styl.blue(`[${name}]`), "Finished.", styl.green(`${elapsed}`))
    }
  })
}

// check if we're building all or just a single package
if (self) buildModule()
else directoriesOf("./modules").forEach(name => buildModule(name))
