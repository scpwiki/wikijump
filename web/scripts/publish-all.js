const fs = require("fs")
const path = require("path")
const packageJson = require("package-json")
const {
  chalk,
  info,
  look,
  section,
  linebreak,
  warn,
  question,
  cmd,
  separator
} = require("./pretty-logs")

/*
 * This script is used to publish any modules that have versions higher than what
 * is currently in the NPM registry. It goes through this process:
 * 1. Get the list of modules that have non-zero versions
 * 2. Filter out any module that is up-to-date in the NPM registry
 * 3. Pack, and publish the remaining modules
 */

const ROOT = path.resolve(__dirname, "../")
process.chdir(ROOT)

const modules = fs
  .readdirSync(`${ROOT}/modules`)
  .filter(dir => fs.statSync(`${ROOT}/modules/${dir}`).isDirectory())

;(async () => {
  // getting list of modules

  const outOfDate = []
  for (const module of modules) {
    try {
      const path = `${ROOT}/modules/${module}`
      const json = require(`${path}/package.json`)

      const version = json.version
      if (version === "0.0.0") continue

      const npmVersion = await getLatest(json.name)
      if (version !== npmVersion) outOfDate.push([module, path, version, npmVersion])
    } catch (err) {
      warn(`Could not get latest version for "${module}"`)
    }
  }

  // asking if the user wants to publish

  if (outOfDate.length === 0) {
    info("All modules are up-to-date")
    linebreak()
    return
  }

  look("These modules are ready to be published:")

  // makes a list like:
  // - module v1.0.0 -> v2.0.0
  outOfDate.forEach(([module, , version, npmVersion]) => {
    // prettier-ignore
    console.log(`${chalk.gray("-")} ${chalk.blueBright(module)} v${npmVersion} ${chalk.gray("->")} v${version}`)
  })

  linebreak()
  const answer = await question("Do you want to publish these modules? [y/N] -> ")
  linebreak()

  if (answer.trim().toLowerCase() !== "y") {
    info(`Aborted publish of ${outOfDate.length} module(s)`)
    return
  }

  // packaging

  info(`Starting packaging and publishing of ${outOfDate.length} module(s)`)
  linebreak()

  section("BUILD")
  linebreak()

  for (const [module] of outOfDate) {
    cmd(`node scripts/vite-pack.js ${module}`)
  }

  // publishing

  section("PUBLISH")

  for (const [module, path] of outOfDate) {
    linebreak()
    info(`Publishing "${module}"`)
    separator()
    process.chdir(`${path}/dist`)
    cmd("npm publish --access public")
  }

  linebreak()
  info(`Finished publishing ${outOfDate.length} module(s)`)
  linebreak()
})()

async function getLatest(package) {
  return (await packageJson(package.trim().toLowerCase())).version
}
