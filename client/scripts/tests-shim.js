const { JSDOM } = require("jsdom")
const fetch = require("node-fetch")
const { performance } = require("perf_hooks")

const DOM = new JSDOM("")

globalThis.window = DOM.window
globalThis.document = DOM.window.document
globalThis.fetch = fetch
globalThis.performance = performance
globalThis.DOMParser = DOM.window.DOMParser
