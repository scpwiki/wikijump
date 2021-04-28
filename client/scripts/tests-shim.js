const { JSDOM } = require("jsdom")
const fetch = require("node-fetch")
const { performance } = require("perf_hooks")

globalThis.window = new JSDOM("")
globalThis.fetch = fetch
globalThis.performance = performance
