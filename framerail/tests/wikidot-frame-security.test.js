import assert from "node:assert/strict"
import test from "node:test"

const previousFramerailEnvironment = process.env.FRAMERAIL_ENV
process.env.FRAMERAIL_ENV = "local"
const {
  applyStaticSecurityHeaders,
  applyStaticSecurityHeadersToNodeResponse,
  materializeSiteCsp
} = await import("../src/lib/server/security-headers.js")
if (previousFramerailEnvironment === undefined) delete process.env.FRAMERAIL_ENV
else process.env.FRAMERAIL_ENV = previousFramerailEnvironment

test("permits only the two same-origin Wikidot compatibility frames", () => {
  for (const pathname of [
    "/-/wikidot-interwiki/interwikiFrame.html",
    "/-/wikidot-interwiki/styleFrame.html"
  ]) {
    const response = new Response("")
    applyStaticSecurityHeaders(response, pathname)
    assert.equal(response.headers.get("x-frame-options"), "SAMEORIGIN")
    assert.match(
      response.headers.get("content-security-policy") ?? "",
      /frame-ancestors 'self'/u
    )
  }
})

test("keeps ordinary pages unframeable", () => {
  const response = new Response("", {
    headers: { "content-security-policy": "frame-ancestors 'none'" }
  })
  applyStaticSecurityHeaders(response, "/scp-9506")
  assert.equal(response.headers.get("x-frame-options"), "DENY")
  assert.equal(response.headers.get("content-security-policy"), "frame-ancestors 'none'")
})

test("materializes an exact current-site file origin", () => {
  const response = new Response("", {
    headers: {
      "content-security-policy":
        "default-src 'self'; img-src 'self' https://wikijump-current-site.invalid; style-src 'self' https://wikijump-current-site.invalid"
    }
  })
  materializeSiteCsp(response, "scp-wiki")
  const policy = response.headers.get("content-security-policy") ?? ""
  assert.match(policy, /https:\/\/scp-wiki\.wjfiles\.localhost/u)
  assert.doesNotMatch(policy, /wikijump-current-site\.invalid|\*\.wjfiles/u)
})

test("does not materialize untrusted site slugs", () => {
  const response = new Response("", {
    headers: {
      "content-security-policy": "img-src https://wikijump-current-site.invalid"
    }
  })
  materializeSiteCsp(response, "scp-wiki.evil")
  assert.equal(
    response.headers.get("content-security-policy"),
    "img-src https://wikijump-current-site.invalid"
  )
})

test("replaces only an exact CSP source token", () => {
  const prefixed = `https://evil.example/https://wikijump-current-site.invalid`
  const suffixed = `https://wikijump-current-site.invalid.evil.example`
  const response = new Response("", {
    headers: {
      "content-security-policy": `img-src ${prefixed} https://wikijump-current-site.invalid ${suffixed}`
    }
  })
  materializeSiteCsp(response, "scp-wiki")
  assert.equal(
    response.headers.get("content-security-policy"),
    `img-src ${prefixed} https://scp-wiki.wjfiles.localhost ${suffixed}`
  )
})

test("node compatibility responses receive the same frame policy", () => {
  const headers = new Map()
  const response = {
    /** @param {string} name @param {string} value */
    setHeader(name, value) {
      headers.set(name, value)
    },
    /** @param {string} name */
    removeHeader(name) {
      headers.delete(name)
    }
  }
  applyStaticSecurityHeadersToNodeResponse(
    response,
    "/-/wikidot-interwiki/styleFrame.html"
  )
  assert.equal(headers.get("x-frame-options"), "SAMEORIGIN")
  assert.match(headers.get("content-security-policy"), /frame-ancestors 'self'/u)
})
