import { strict as assert } from "node:assert"
import test from "node:test"

const previousFramerailEnvironment = process.env.FRAMERAIL_ENV
process.env.FRAMERAIL_ENV = "local"
const { default: config } = await import("../svelte.config.js")
if (previousFramerailEnvironment === undefined) {
  delete process.env.FRAMERAIL_ENV
} else {
  process.env.FRAMERAIL_ENV = previousFramerailEnvironment
}
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
  for (const origin of [
    "https://scp-wiki.wikidot.com",
    "https://scp-jp-storage.wikidot.com",
    "https://scpsandboxcn.wikidot.com"
  ]) {
    assert(directives["img-src"]?.some((source) => source === origin))
  }
  assert(
    directives["style-src"]?.some((source) => source === "https://nu-scptheme.github.io")
  )
  assert(
    directives["style-src"]?.some((source) => source === "https://fonts.googleapis.com")
  )
  for (const origin of ["https://fonts.gstatic.com", "https://nu-scptheme.github.io"]) {
    assert(directives["font-src"]?.some((source) => source === origin))
  }
  for (const sources of [
    directives["img-src"],
    directives["script-src"],
    directives["connect-src"],
    directives["frame-src"],
    directives["style-src"],
    directives["font-src"]
  ]) {
    assert(!sources?.some((source) => source === "https://*.wikidot.com"))
  }
  assert(directives["img-src"]?.includes("https://wikijump-current-site.invalid"))
  assert(directives["style-src"]?.includes("https://wikijump-current-site.invalid"))
  assert(!directives["img-src"]?.some((source) => source.includes("*.wjfiles")))
  assert(!directives["style-src"]?.some((source) => source.includes("*.wjfiles")))
  assert.deepEqual(directives["frame-src"], ["self"])
  assert.deepEqual(directives["script-src"], ["self"])
})
