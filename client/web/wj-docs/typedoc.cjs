const globby = require("globby")
const path = require("path")
const TypeDoc = require("typedoc")

// change working directory to repo root if it isn't already
process.chdir(path.resolve(__dirname, "../../"))

const OUTPUT_DIR = "web/wj-docs/dist"
const TS_CONFIG = "tsconfig.typedoc.json"

async function main() {
  const modules = [
    ...(await globby("modules/*", { onlyDirectories: true, ignore: ["modules/wj-css"] }))
  ]

  const app = new TypeDoc.Application()

  app.options.addReader(new TypeDoc.TSConfigReader())

  app.bootstrap({
    packages: modules,
    name: "Wikijump",
    tsconfig: TS_CONFIG,
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
