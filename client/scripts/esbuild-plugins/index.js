const compileWorkers = require("./esbuild-compile-worker")
const importTOML = require("./esbuild-import-toml")

module.exports = [compileWorkers, importTOML]
