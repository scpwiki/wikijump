const fileRegex = /\.ftl$/

module.exports = function viteTOMLPlugin() {
  return {
    name: "ftl",

    transform(src, id) {
      if (fileRegex.test(id)) {
        return {
          // stringify twice so that we get an escaped string
          // e.g. "{\"foo\": \"bar\"}"
          code: `export default JSON.parse(${JSON.stringify(JSON.stringify(src))});`,
          map: { mappings: "" }
        }
      }
    }
  }
}
