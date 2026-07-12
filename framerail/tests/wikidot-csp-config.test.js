import { strict as assert } from "node:assert"
import test from "node:test"

process.env.FRAMERAIL_ENV = "local"

const { default: config } = await import("../svelte.config.js")
const directives = config.kit?.csp?.directives
if (!directives) throw new Error("CSP directives are missing")

test("allows captured Wikidot legacy asset origins in local CSP", () => {
  assert(directives["style-src"]?.some((source) => source === "https://cdn.jsdelivr.net"))
  assert(directives["font-src"]?.some((source) => source === "https://cdn.jsdelivr.net"))
  assert(
    directives["img-src"]?.some(
      (source) => source === "https://d3g0gp89917ko0.cloudfront.net"
    )
  )
  assert(
    directives["frame-src"]?.some((source) => source === "https://*.wjfiles.localhost")
  )
  assert.deepEqual(directives["script-src"], ["self"])
})
