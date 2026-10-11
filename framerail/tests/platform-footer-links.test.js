import { strict as assert } from "node:assert"
import test from "node:test"

import {
  DEFAULT_NATIVE_FOOTER_LINKS,
  NATIVE_FOOTER_DESTINATION_ENVIRONMENT_VARIABLES,
  NATIVE_FOOTER_DESTINATION_KEYS,
  normalizePlatformFooterDestination,
  readNativeFooterDestinationOverrides,
  resolveNativeFooterLinks
} from "../src/lib/platform-footer-links.js"

const KEYS_AND_DEFAULT_HREFS = [
  ["terms-conditions", "https://wikijump.com/legal/terms-of-service"],
  ["privacy", "https://wikijump.com/legal/privacy-policy"],
  ["docs", "https://wikijump.com/docs"],
  ["security", "https://wikijump.com/legal/security"]
]

test("keeps the native footer order and its internationalization keys", () => {
  assert.deepEqual(NATIVE_FOOTER_DESTINATION_KEYS, [
    "terms-conditions",
    "privacy",
    "docs",
    "security"
  ])
  assert.deepEqual(
    DEFAULT_NATIVE_FOOTER_LINKS.map((link) => link.key),
    NATIVE_FOOTER_DESTINATION_KEYS
  )
})

test("binds every native footer item to an explicit absolute destination", () => {
  assert.deepEqual(
    DEFAULT_NATIVE_FOOTER_LINKS.map((link) => [link.key, link.href]),
    KEYS_AND_DEFAULT_HREFS
  )

  for (const link of DEFAULT_NATIVE_FOOTER_LINKS) {
    const url = new URL(link.href)
    // Legal and docs links never resolve against the wiki being viewed, and
    // never collapse onto one shared homepage.
    assert.equal(url.protocol, "https:")
    assert.notEqual(url.href, "https://wikijump.com/", `${link.key} is the homepage`)
    assert.notEqual(
      url.pathname,
      "/",
      `${link.key} must not resolve to a platform homepage`
    )
  }

  const hrefs = DEFAULT_NATIVE_FOOTER_LINKS.map((link) => link.href)
  assert.equal(new Set(hrefs).size, hrefs.length)
})

test("resolves the documented destinations when nothing is configured", () => {
  assert.deepEqual(
    resolveNativeFooterLinks(),
    KEYS_AND_DEFAULT_HREFS.map(([key, href]) => ({ key, href }))
  )
  assert.deepEqual(
    resolveNativeFooterLinks({}),
    KEYS_AND_DEFAULT_HREFS.map(([key, href]) => ({ key, href }))
  )
  assert.deepEqual(
    resolveNativeFooterLinks({ docs: "   ", privacy: null, security: undefined }),
    KEYS_AND_DEFAULT_HREFS.map(([key, href]) => ({ key, href }))
  )
})

test("lets a deployment point every item at its own instance resources", () => {
  const configured = {
    "terms-conditions": "https://legacy.example.org/terms",
    privacy: "https://legacy.example.org/privacy",
    docs: "https://legacy.example.org/manual",
    security: "https://legacy.example.org/.well-known/security.txt"
  }
  assert.deepEqual(resolveNativeFooterLinks(configured), [
    { key: "terms-conditions", href: "https://legacy.example.org/terms" },
    { key: "privacy", href: "https://legacy.example.org/privacy" },
    { key: "docs", href: "https://legacy.example.org/manual" },
    { key: "security", href: "https://legacy.example.org/.well-known/security.txt" }
  ])

  // One configured item does not disturb the others.
  const partial = resolveNativeFooterLinks({ docs: "https://legacy.example.org/manual" })
  assert.equal(partial[2].href, "https://legacy.example.org/manual")
  assert.equal(partial[0].href, DEFAULT_NATIVE_FOOTER_LINKS[0].href)
  assert.equal(partial[1].href, DEFAULT_NATIVE_FOOTER_LINKS[1].href)
  assert.equal(partial[3].href, DEFAULT_NATIVE_FOOTER_LINKS[3].href)
})

test("normalizes trailing whitespace and explicit default ports", () => {
  assert.equal(
    normalizePlatformFooterDestination("  https://legacy.example.org/terms\n"),
    "https://legacy.example.org/terms"
  )
  assert.equal(
    normalizePlatformFooterDestination("https://legacy.example.org:443/terms"),
    "https://legacy.example.org/terms"
  )
})

test("fails closed on destinations that could resolve against a viewed wiki", () => {
  for (const unsafe of [
    null,
    undefined,
    "",
    "   ",
    "/legal/terms-of-service",
    "legal/terms-of-service",
    "//wikijump.com/legal/terms-of-service",
    "#terms",
    "?page=terms",
    "javascript:alert(1)",
    "data:text/html,doc",
    "http://legacy.example.org/terms",
    "ftp://legacy.example.org/terms",
    "https://user:secret@legacy.example.org/terms",
    "https://legacy.example.org:not-a-port/terms",
    "https://legacy.example.org/over\nlong"
  ]) {
    assert.equal(
      normalizePlatformFooterDestination(unsafe),
      null,
      `${unsafe} must be rejected`
    )
  }

  // A rejected configured value falls back to the documented default rather
  // than emitting a link the viewed wiki could influence.
  assert.deepEqual(
    resolveNativeFooterLinks({ privacy: "/privacy", docs: "javascript:;" }),
    DEFAULT_NATIVE_FOOTER_LINKS.map((link) => ({ ...link }))
  )
})

test("never emits a destination with a control character", () => {
  for (const unsafe of [
    "https://legacy.example.org/term\u0000s",
    "https://legacy.example.org/term\u001fs",
    "https://legacy.example.org/term\\s",
    "https://legacy.example.org/term s"
  ]) {
    assert.equal(normalizePlatformFooterDestination(unsafe), null)
  }
})

test("reads the instance overrides from the public platform environment", () => {
  const environment = {
    PUBLIC_WIKIJUMP_FOOTER_TERMS_CONDITIONS_URL: "https://legacy.example.org/terms",
    PUBLIC_WIKIJUMP_FOOTER_PRIVACY_URL: undefined,
    PUBLIC_WIKIJUMP_FOOTER_DOCS_URL: "https://legacy.example.org/manual",
    PUBLIC_WIKIJUMP_FOOTER_SECURITY_URL: "https://legacy.example.org/security",
    PUBLIC_SOMETHING_ELSE: "https://legacy.example.org/ignored",
    SECRET_NOT_PUBLIC: "https://legacy.example.org/ignored"
  }

  assert.deepEqual(readNativeFooterDestinationOverrides(environment), {
    "terms-conditions": "https://legacy.example.org/terms",
    privacy: undefined,
    docs: "https://legacy.example.org/manual",
    security: "https://legacy.example.org/security"
  })
  assert.deepEqual(readNativeFooterDestinationOverrides(), {
    "terms-conditions": undefined,
    privacy: undefined,
    docs: undefined,
    security: undefined
  })

  const keys = Object.keys(NATIVE_FOOTER_DESTINATION_ENVIRONMENT_VARIABLES)
  for (const key of NATIVE_FOOTER_DESTINATION_KEYS) {
    assert.ok(keys.includes(key), `${key} must name an environment variable`)
    assert.match(
      NATIVE_FOOTER_DESTINATION_ENVIRONMENT_VARIABLES[key],
      /^PUBLIC_WIKIJUMP_FOOTER_[A-Z_]+_URL$/u
    )
  }
})
