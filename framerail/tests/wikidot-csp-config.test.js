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
  assert.equal(directives["upgrade-insecure-requests"], true)
  assert(directives["style-src"]?.some((source) => source === "https://cdn.jsdelivr.net"))
  assert(!directives["style-src"]?.some((source) => source === "https:"))
  assert(directives["font-src"]?.some((source) => source === "https://cdn.jsdelivr.net"))
  assert(
    directives["img-src"]?.some(
      (source) => source === "https://d3g0gp89917ko0.cloudfront.net"
    )
  )
  for (const origin of [
    "https://scp-wiki.wikidot.com",
    "https://www.wikidot.com",
    "https://scp-jp-storage.wikidot.com",
    "https://scpsandboxcn.wikidot.com"
  ]) {
    assert(directives["img-src"]?.some((source) => source === origin))
  }
  assert(
    directives["img-src"]?.some(
      (source) => source === "https://d2qhngyckgiutd.cloudfront.net"
    )
  )
  for (const origin of [
    "https://scp-wiki.wjfiles.localhost",
    "https://scp-jp.wjfiles.localhost"
  ]) {
    assert(directives["img-src"]?.some((source) => source === origin))
  }
  assert(
    directives["style-src"]?.some((source) => source === "https://nu-scptheme.github.io")
  )
  assert(
    directives["style-src"]?.some((source) => source === "https://fonts.googleapis.com")
  )
  assert(
    !directives["script-src"]?.some(
      (source) => source === "https://d3g0gp89917ko0.cloudfront.net"
    )
  )
  for (const origin of ["https://fonts.gstatic.com", "https://nu-scptheme.github.io"]) {
    assert(directives["font-src"]?.some((source) => source === origin))
  }
  // The imported classified-decoration component `@import`s its Renner
  // stylesheet and WOFF2 from Indestructible Type's font-hosting GitHub
  // Pages site. Both retained shared-corpus copies of
  // `component:classified-decoration-base` carry that import, and current
  // Wikidot loads both assets, so the audited origin has to be allowlisted
  // for CSS and font bytes while staying out of every other directive
  // (issue #2213).
  for (const origin of ["https://indestructibletype-fonthosting.github.io"]) {
    assert(directives["style-src"]?.some((source) => source === origin))
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
  assert(
    directives["img-src"]?.some(
      (source) => source === "https://wikijump-current-site.invalid"
    )
  )
  assert(
    directives["style-src"]?.some(
      (source) => source === "https://wikijump-current-site.invalid"
    )
  )
  assert(
    !directives["img-src"]?.some((source) => source === "https://*.wjfiles.localhost")
  )
  assert(
    !directives["style-src"]?.some((source) => source === "https://*.wjfiles.localhost")
  )
  assert.deepEqual(directives["frame-src"], [
    "self",
    "https://www.youtube.com",
    "https://embed.acast.com"
  ])
  assert(directives["script-src"]?.includes("self"))
})
