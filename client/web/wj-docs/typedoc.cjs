const globby = require("globby")
const fs = require("fs")
const path = require("path")
const TypeDoc = require("typedoc")

// change working directory to repo root if it isn't already
process.chdir(path.resolve(__dirname, "../../"))

const OUTPUT_DIR = "web/wj-docs/dist"

async function main() {
  const modules = fs
    .readdirSync("modules")
    .filter(dir => fs.statSync(`modules/${dir}`).isDirectory())
    .filter(dir => !["wj-css"].some(ignore => dir.endsWith(ignore)))
    .map(dir => `modules/${dir}`)

  const app = new TypeDoc.Application()

  app.options.addReader(new TypeDoc.TSConfigReader())

  app.bootstrap({
    entryPoints: modules,
    entryPointStrategy: "packages",
    name: "Wikijump",
    tsconfig: "tsconfig.json",
    exclude: [
      "**/tests/**",
      "**/node_modules/**",
      "**/dist/**/",
      "**/*.js",
      "**/*.mjs",
      "**/*.cjs",
      "web/**/"
    ],
    excludeExternals: true
  })

  const project = app.convert()

  if (project) {
    await app.generateDocs(project, OUTPUT_DIR)
  }
}

main().catch(console.error)
