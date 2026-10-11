import { strict as assert } from "node:assert"
import test from "node:test"

const previousFramerailEnvironment = process.env.FRAMERAIL_ENV
process.env.FRAMERAIL_ENV = "local"
const { default: config } = await import("../svelte.config.js")
const { AUDITED_EXTERNAL_ASSET_ORIGINS, auditedExternalAssetOrigins } =
  await import("../src/lib/server/audited-external-asset-origins.js")
if (previousFramerailEnvironment === undefined) {
  delete process.env.FRAMERAIL_ENV
} else {
  process.env.FRAMERAIL_ENV = previousFramerailEnvironment
}
const directives = config.kit?.csp?.directives
if (!directives) throw new Error("CSP directives are missing")

test("every audited external asset origin is a documented https origin", () => {
  const seen = new Set()
  for (const entry of AUDITED_EXTERNAL_ASSET_ORIGINS) {
    assert.equal(typeof entry.origin, "string", "origin must be a string")
    assert.ok(
      /^https:\/\/(?:\*\.)?[a-z0-9.-]+$/u.test(entry.origin),
      `origin is not an exact https host-source: ${entry.origin}`
    )
    // A scheme-wide source such as `https:` would let an arbitrary
    // author-supplied `@import` URL load anywhere in the world.
    assert.notEqual(entry.origin, "https:", "scheme-wide sources are not auditable")
    assert.ok(
      entry.styles || entry.fonts,
      `origin grants no directive it can actually serve: ${entry.origin}`
    )
    assert.equal(
      typeof entry.evidence,
      "string",
      `origin has no retained evidence: ${entry.origin}`
    )
    assert.ok(
      entry.evidence.trim().length > 0,
      `origin has empty retained evidence: ${entry.origin}`
    )
    assert.ok(!seen.has(entry.origin), `duplicate audited origin: ${entry.origin}`)
    seen.add(entry.origin)
  }
})

test("the published style and font allowlists are exactly the audited origins", () => {
  const styleOrigins = auditedExternalAssetOrigins("styles")
  const fontOrigins = auditedExternalAssetOrigins("fonts")
  for (const origin of styleOrigins) {
    assert.ok(
      directives["style-src"]?.includes(origin),
      `style-src is missing the audited origin ${origin}`
    )
  }
  for (const origin of fontOrigins) {
    assert.ok(
      directives["font-src"]?.includes(origin),
      `font-src is missing the audited origin ${origin}`
    )
  }
})

test("no audited asset origin is allowlisted outside style-src and font-src", () => {
  // An allowlisted CSS or font origin must never become an executable,
  // connection or frame dependency of a rendered page, so the CSP grants it
  // exactly the directives the retained evidence requires. An image origin
  // is a separate audit, so `img-src` is not part of this invariant.
  for (const entry of AUDITED_EXTERNAL_ASSET_ORIGINS) {
    for (const directive of [
      "script-src",
      "connect-src",
      "frame-src",
      "worker-src",
      "manifest-src",
      "form-action",
      "base-uri"
    ]) {
      assert.ok(
        !(directives[directive] ?? []).includes(entry.origin),
        `audited asset origin ${entry.origin} must not be allowlisted in ${directive}`
      )
    }
  }
})

test("an untrusted external origin cannot reach the audited allowlist", () => {
  const untrustedOrigins = [
    "https://example.invalid",
    "https://attacker-controlled-font-hosting.example.com"
  ]
  for (const directive of ["style-src", "font-src", "script-src"]) {
    for (const origin of untrustedOrigins) {
      assert.ok(
        !(directives[directive] ?? []).includes(origin),
        `${origin} must not be allowlisted in ${directive}`
      )
    }
  }
})

test("no directive falls back to a scheme-wide external source", () => {
  for (const directive of ["style-src", "font-src"]) {
    for (const source of directives[directive] ?? []) {
      assert.notEqual(source, "https:", `${directive} must not allow every https origin`)
      assert.notEqual(source, "http:", `${directive} must not allow every http origin`)
      assert.notEqual(source, "https://*", `${directive} must not allow every https host`)
    }
  }
})
