/*
 * Build and development tool for the legacy frontend.
 *
 * To use, do either:
 * - `node build.js`     | Build
 * - `node build.js dev` | Watch mode
 */

const path = require("path")
const { execSync: exec } = require("child_process")
const esbuild = require("esbuild")

const DIR = path.resolve(__dirname, "../")

/** @type esbuild.BuildOptions */
const SETTINGS_COMMON = {
  absWorkingDir: DIR,
  tsconfig: "./tsconfig.legacy.json",
  entryPoints: ["web/files--common/javascript/index.ts"],
  outfile: "web/files--common/dist/bundle.js",
  bundle: true,
  splitting: false,
  platform: "browser",
  format: "iife",
  target: "es6"
}

/** @type esbuild.BuildOptions */
const SETTINGS_BUILD = {
  ...SETTINGS_COMMON,

  minify: true
}

/** @type esbuild.BuildOptions */
const SETTINGS_DEV = {
  ...SETTINGS_COMMON,

  logLevel: "info",
  sourcemap: true,
  watch: true
}

const mode = process.argv[2].trim()

// this is a bit of a wacky place to put it, but in order to gracefully stop
// the docker container we'll need to do it somewhere with access to hooks
// so we'll do it here
if (mode === "dev") {
  process.on("SIGINT", () => {
    process.chdir(DIR)
    exec("pnpm compose stop", { cwd: DIR, shell: true })
  })
}

buildLegacyBundle(mode)

async function buildLegacyBundle(mode) {
  try {
    if (mode === "build") console.log("[legacy] Building legacy bundle...")

    const server = await esbuild.build(mode === "dev" ? SETTINGS_DEV : SETTINGS_BUILD)

    // if we're in dev, `watch` option starts a dev server
    if (mode === "dev") {
      process.on("beforeExit", () => server.stop())
    }

    if (mode === "build") console.log("[legacy] Legacy bundle built.")
    else if (mode === "dev") console.log("[legacy] Legacy development server started.")

    // catch errors in general rather than catching on individual promises
  } catch (err) {
    if (err) console.error("[legacy] Process failed:", err)
    process.exit(1)
  }
}
