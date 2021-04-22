const { build, cliopts, basename, stdoutStyle: styl, fmtDuration } = require("estrella")
const { readdirSync, realpathSync } = require("fs")
const { performance } = require("perf_hooks")
const browserslist = require("browserslist")

const { nodeExternalsPlugin } = require("esbuild-node-externals")
const compileWorkersPlugin = require("./esbuild-compile-worker")

// -- CONSTANTS, COMMAND LINE ARGUMENTS

const [opts] = cliopts.parse(
  ["self", "Compiles the calling module package only."],
  ["tests", "Compile the module tests instead."]
)

const DEV = Boolean(cliopts.watch)
const SELF = opts.self ?? false
const TESTS = opts.tests ?? false

if (DEV && TESTS) throw new Error("Can't use -watch and -tests at the same time!")

const fmt = TESTS ? styl.orange : styl.blue

/** Common settings for all modules that are built. */
const SETTINGS_COMMON = {
  // esbuild settings
  outdir: "dist",
  // minify: true // estrella implicitly toggles minify depending on the `--debug` flag
  bundle: true,
  treeShaking: true,
  splitting: false,
  format: "esm",
  platform: "browser",
  sourcemap: true,
  sourcesContent: true,

  // estrella settings
  tslint: false, // disables estrella's built-in tsc typechecking
  quiet: true // silences estrella's logging, as we use our own logging
}

// prettier-ignore
const DEMANGLE_TARGET_REGEX =
  /^(chrome|and_(?:ff|chr)|edge|firefox|ios_saf|safari)\s+([\d.]+)/

// -- DEMANGLE BROWSERSLIST

/*
  esbuild expects the browsers list provided to it look like this:
  chrome89, firefox78, etc.
  Browserslist looks like this:
  chrome 89, firefox 78, etc.
  Additionally, esbuild only accepts certain browsers.
  So, this chunk of code converts the browserslist list into an esbuild list.
  It also renames some of the mobile browsers to match esbuild's version.
*/

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

// -- BUILD

// check if we're building all or just a single package
if (SELF) buildModule()
else directoriesOf("./modules").forEach(name => buildModule(name))

// -- FUNCTIONS

/** Builds the module (a package in `modules/`).
 *  If `-tests` was specified, it'll compile the module's tests, instead. */
function buildModule(name) {
  const dir = SELF ? process.cwd() : realpathSync(`./modules/${name}`)
  const index = `./src/index.ts`
  const package = `./package.json`

  let tests
  if (TESTS) {
    try {
      tests = filesOf(`${dir}/tests`).map(name => `./tests/${name}`)
      if (tests.length === 0) {
        console.log(fmt(`[${name}]`), "No tests, skipping.")
        return
      }
    } catch {
      // we'll be here if the `tests` folder doesn't exist
      console.log(fmt(`[${name}]`), "No tests, skipping.")
      return
    }
  }

  name = basename(dir)

  const NODE = name.startsWith("node-")

  if (!DEV) console.log(fmt(`[${name}]`), "Building!")
  let start = performance.now()

  build({
    // -- SETTINGS

    ...SETTINGS_COMMON,

    // esbuild
    absWorkingDir: dir,
    tsconfig: `${dir}/tsconfig.json`,
    plugins: [
      nodeExternalsPlugin({ packagePath: `${dir}/package.json` }),
      compileWorkersPlugin
    ],
    loader: { ".wasm": "file" },
    target: [...targets],

    // estrella
    entry: index,
    cwd: dir,
    // this property uses the Chokidar watch settings interface
    // we need to make sure that Chokidar is watching the module directory
    watch: DEV && { cwd: dir },

    // -- MODES

    // node compiling
    // prettier-ignore
    ...(!NODE ? {} : {
      bundle: false,
      splitting: false,
      format: "cjs",
      platform: "node",
      target: undefined
    }),

    // test compiling
    //prettier-ignore
    ...(!TESTS ? {} : {
      entry: [...tests],
      outdir: `${dir}/tests/dist`,
      bundle: true,
      minify: false,
      splitting: false,
      format: "cjs",
      platform: "node",
      sourcemap: false,
      target: undefined,
      outExtension: { ".js": ".cjs" },
      define: {
        "window": "globalThis"
      }
    }),

    // -- LOGGING HANDLERS

    onStart(_, changed) {
      if (DEV && changed && changed.length) {
        start = performance.now()
        console.log(
          fmt(`[${name}]`),
          "Rebuilding!",
          styl.orange(`[${changed.join(", ")}]`)
        )
      }
    },

    onEnd() {
      const elapsed = fmtDuration(performance.now() - start)
      console.log(fmt(`[${name}]`), "Finished.", styl.green(`${elapsed}`))
    }
  })
}

/** Gets the directories (shallow) of a folder. */
function directoriesOf(source) {
  return readdirSync(source, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name)
}

/** Gets the files (shallow) of a folder. */
function filesOf(source) {
  return readdirSync(source, { withFileTypes: true })
    .filter(dirent => dirent.isFile())
    .map(dirent => dirent.name)
}
