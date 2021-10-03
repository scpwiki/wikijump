const yaml = require("js-yaml")

const fileRegex = /\.(yaml|yml)$/

module.exports = function viteYAMLPlugin() {
  return {
    name: "yaml",

    transform(src, id) {
      if (fileRegex.test(id)) {
        const obj = yaml.load(src)
        if (!obj || typeof obj !== "object") {
          throw new Error(
            "Invalid YAML provided to plugin. YAML provided must resolve to an object."
          )
        }

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
