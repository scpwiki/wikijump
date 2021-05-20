const toml = require("@ltd/j-toml")

const fileRegex = /\.toml$/

module.exports = function viteTOMLPlugin() {
  return {
    name: "toml",

    transform(src, id) {
      if (fileRegex.test(id)) {
        const obj = toml.parse(src, 1.0, "\n", false, { order: true, null: true })

        return {
          // stringify twice so that we get an escaped string
          // e.g. "{\"foo\": \"bar\"}"
          code: `export default JSON.parse(${JSON.stringify(JSON.stringify(obj))});`,
          map: { mappings: "" }
        }
      }
    }
  }
}
