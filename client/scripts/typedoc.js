const globby = require("globby")
const path = require("path")
const TypeDoc = require("typedoc")

// change working directory to repo root if it isn't already
process.chdir(path.resolve(__dirname, "../"))

const OUTPUT_DIR = "web/wj-docs/dist"
const TS_CONFIG = "tsconfig.typedoc.json"

async function main() {
  const packages = [
    "components",
    ...(await globby("modules/*", { onlyDirectories: true }))
  ]

  const app = new TypeDoc.Application()

  app.options.addReader(new TypeDoc.TSConfigReader())

  app.bootstrap({
    name: "Wikijump",
    tsconfig: TS_CONFIG,
    entryPoints: packages,
    exclude: ["**/tests/**", "**/node_modules/**", "*.js"],
    // added by plugin (@strictsoftware/typedoc-plugin-monorepo)
    "external-modulemap": ".*(modules/([^/]+)/|components/).*"
  })

  const project = app.convert()

  if (project) {
    await app.generateDocs(project, OUTPUT_DIR)
  }
}

main().catch(console.error)
