import assert from "node:assert/strict"
import test from "node:test"

import { translateMfaCodeLabel } from "../src/lib/server/load/login-translation.ts"

test("uses the translated MFA label when the active catalog provides it", async () => {
  let request
  const label = await translateMfaCodeLabel(["ja", "en"], async (locales, keys) => {
    request = { locales, keys }
    return { "mfa-code": "MFA コード" }
  })

  assert.equal(label, "MFA コード")
  assert.deepEqual(request, { locales: ["ja", "en"], keys: { "mfa-code": {} } })
})

test("falls back only when an older catalog lacks the optional MFA key", async () => {
  const label = await translateMfaCodeLabel(["en"], async () => {
    throw Object.assign(new Error("Message key not found for this locale"), {
      code: 5002
    })
  })

  assert.equal(label, "MFA code")
})

test("does not hide transport or unrelated translation failures", async () => {
  const failure = Object.assign(new Error("Deepwell unavailable"), { code: 502 })

  await assert.rejects(
    translateMfaCodeLabel(["en"], async () => {
      throw failure
    }),
    failure
  )
})
