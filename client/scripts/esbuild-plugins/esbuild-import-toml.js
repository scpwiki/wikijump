const path = require("path")
const toml = require("@ltd/j-toml")
const fs = require("fs/promises")

module.exports = {
  name: "import-toml",
  setup(build) {
    build.onResolve({ filter: /\.toml$/ }, args => {
      // path can't be resolved, ignore
      if (args.resolveDir === "") return

      const pathTOML = path.join(path.dirname(args.importer), args.path)

      return {
        path: args.path,
        namespace: "import-toml",
        pluginData: { pathTOML }
      }
    })

    build.onLoad({ filter: /.*/, namespace: "import-toml" }, async args => {
      const { pathTOML } = args.pluginData

      const tomlStr = await fs.readFile(pathTOML, "utf-8")
      const parsed = toml.parse(tomlStr, 1.0, "\n", false, { order: true, null: true })
      // we'll nicely format the JSON so that it's fairly readable
      // in something like a web debugger
      const stringified = JSON.stringify(parsed, undefined, 2)

      return {
        contents: stringified,
        loader: "json"
      }
    })
  }
}
