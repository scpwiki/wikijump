const vite = require("vite")
const path = require("path")
const fs = require("fs-extra")
const { linebreak, separator, info, warn, error } = require("./pretty-logs.js")
const { BASE_CONFIG } = require("./vite-utils.js")

/*
 * This script allows for the creation of NPM packages from a module in the
 * monorepo. Specifically, it bundles the module and copies the
 * package.json (and friends) into the modules "dist" folder. This is where
 * the package will be published from.
 *
 * This allows for the actual package.json to still point to the module's
 * source, so that the monorepo itself doesn't need to have every module
 * built to work. In general, the monorepo uses no built modules when
 * developing or building websites.
 *
 * Packages are published with their sources, which has two main advantages:
 * 1. The `types` field in the package.json is set to the module's source
 * 2. The package is built with sourcemaps that reference the source
 *
 * The disadvantage is the package is larger, but it's a good tradeoff.
 */

// node vite-package module-name
//                       ^
const package = process.argv[2]?.trim()?.toLowerCase()

if (!package) {
  console.error("No module specified")
  process.exit(1)
}

info(`Building package for "${package}"`)
linebreak()

const ROOT = path.resolve(__dirname, "../")
const DIR = path.resolve(__dirname, `../modules/${package}`)
process.chdir(DIR)

const json = require(`${DIR}/package.json`)

if (!json.name || !json.version || !json.main) {
  throw new Error(`Invalid package.json for ${package}`)
}

// imports matching a dependency in this list will be left out of the bundle
let external = Object.keys(json.dependencies || {})

// push other dependencies to external just in case
if (json.devDependencies) external.push(...Object.keys(json.devDependencies))
if (json.optionalDependencies) external.push(...Object.keys(json.optionalDependencies))
if (json.peerDependencies) external.push(...Object.keys(json.peerDependencies))

// converts the externals list to regexs that catch edge cases
// e.g. import from "foo/bar"
// e.g. import from "foo/bar.txt?url"
// in the latter case, we shouldn't externalize the import
// as that has to be handled by Vite
external = external.map(str => new RegExp(`^${escapeRegExp(str)}[^?]*$`))

// modify default config to be for building modules
// otherwise it's entirely the same build process
const config = BASE_CONFIG
config.command = "build"
config.publicDir = false
config.base = "./"
config.root = "./"
config.build.assetsDir = "./"
config.build.minify = false // let consumers minify their own code
config.build.manifest = false
config.build.rollupOptions = {
  external,
  output: {
    sourcemapExcludeSources: true,
    // fixes the relative sourcemap paths
    // e.g. "../../src/index.ts" -> "../src/index.ts"
    sourcemapPathTransform(path) {
      return path.replace(/^\.+(\\|\/)/, "")
    }
  }
}
config.build.lib = {
  entry: `${json.main}`,
  fileName(format) {
    return `wj-${package}.${format === "es" ? "mjs" : "cjs"}`
  }
}

// modify the package.json for NPM publish

// point to built files
json.types = `./${json.main}`
json.main = `./cjs/wj-${package}.cjs`
json.module = `./esm/wj-${package}.mjs`
json.exports ??= {}
json.exports["."] = {
  import: `./esm/wj-${package}.mjs`,
  require: `./cjs/wj-${package}.cjs`
}

// delete fields that would mess with things
delete json.private
delete json.type
delete json.eslintConfig

// make workspace dependencies have their actual versions
// e.g. "workspace:*" -> "^0.5.0"
if (json.dependencies) {
  for (const [name, version] of Object.entries(json.dependencies)) {
    if (version.startsWith("workspace:")) {
      const stripped = name.replace("@wikijump/", "")
      const latest = require(`${ROOT}/modules/${stripped}/package.json`).version
      if (!latest) throw new Error(`No version found for ${name}`)
      if (latest === "0.0.0") {
        linebreak()
        warn(
          `Linked dependency "${name}" has a version of v0.0.0`,
          "That probably means that it hasn't been published",
          `Package "${json.name}" may have unresolvable dependencies`
        )
      }
      json.dependencies[name] = `^${latest}`
    }
  }
}

if (json.version === "0.0.0") {
  linebreak()
  error(
    `Package "${json.name}" has a version of v0.0.0`,
    "You have to give this package a non-zero version"
  )
  process.exit(1)
}

// add additional metadata regarding repository
json.repository = {
  type: "git",
  url: "git+https://github.com/scpwiki/wikijump.git",
  directory: `web/modules/${package}`
}
json.bugs = "https://scuttle.atlassian.net/servicedesk/customer/portal/2"
json.homepage = `https://github.com/scpwiki/wikijump/tree/develop/web/modules/${package}`

// build module, finally
;(async () => {
  // have to clear folder because Vite won't do it due to the outDir workaround
  if (await fs.pathExists(`${DIR}/dist`)) await fs.remove(`${DIR}/dist`)

  // have to build twice, due to Vite bug
  // Vite doesn't separate the CJS and ESM builds from each other
  // so they would overwrite each other's files

  info("Building ESM...")
  separator()
  config.build.outDir = "./dist/esm"
  config.build.lib.formats = ["es"]
  await vite.build(config)

  linebreak()

  info("Building CJS...")
  separator()
  config.build.outDir = "./dist/cjs"
  config.build.lib.formats = ["cjs"]
  await vite.build(config)

  linebreak()

  info("Copying module source files...")
  await copy(`${DIR}/src`, `${DIR}/dist/src`)
  await copy(`${DIR}/vendor`, `${DIR}/dist/vendor`)
  await copy(`${DIR}/bin`, `${DIR}/dist/bin`)

  info("Writing package metadata...")
  await fs.writeFile(`${DIR}/dist/package.json`, JSON.stringify(json, null, 2))
  await copy(`${DIR}/README.md`, `${DIR}/dist/README.md`)
  await copy(`${ROOT}/../LICENSE.md`, `${DIR}/dist/LICENSE.md`)
  await copy(`${DIR}/CHANGELOG.md`, `${DIR}/dist/CHANGELOG.md`)

  info(`Finished packaging "${package}"`)
  linebreak()
})()

async function copy(from, to) {
  if (await fs.pathExists(from)) {
    await fs.copy(from, to)
  }
}

function escapeRegExp(str) {
  return str.replace(/[.*+?^${}()|\[\]\\]/g, "\\$&")
}
