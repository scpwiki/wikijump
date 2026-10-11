import assert from "node:assert/strict"
import test from "node:test"

import { decodeWikidotEmailObfuscation } from "../src/lib/wikidot/wikidot-email-obfuscation.js"

test("decodes a plain Wikidot email address", () => {
  assert.deepEqual(decodeWikidotEmailObfuscation("vog.isfn|ofni#vog.isfn|ofni"), {
    address: "info@nfsi.gov",
    label: "info@nfsi.gov"
  })
})

test("keeps the authored label separate from the decoded mailto target", () => {
  assert.deepEqual(decodeWikidotEmailObfuscation("moc.elpmaxe|troppus#em liame"), {
    address: "support@example.com",
    label: "email me"
  })
})

test("decodes apostrophes after browser HTML entity decoding", () => {
  assert.deepEqual(
    decodeWikidotEmailObfuscation("moc.elpmaxe|arah'o#moc.elpmaxe|arah'o"),
    { address: "o'hara@example.com", label: "o'hara@example.com" }
  )
})

test("leaves malformed or non-email encodings unrecognized", () => {
  for (const value of [
    "info@nfsi.gov",
    "vog.isfn|ofni",
    "no-domain|ofni#ofni",
    "invalid|value#label",
    "vog.isfn|ofni#"
  ]) {
    assert.equal(decodeWikidotEmailObfuscation(value), null, value)
  }
})
