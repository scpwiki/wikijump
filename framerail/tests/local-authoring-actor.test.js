import assert from "node:assert/strict"
import test from "node:test"

import { resolvePageMutationUserId } from "../src/lib/server/load/local-authoring-actor.ts"

const LOCAL_SITE = "scpaiueouiuiuiui"

test("authenticated sessions retain their actor in every deployment environment", () => {
  assert.equal(resolvePageMutationUserId(42, LOCAL_SITE, 7, 7, "prod"), 42)
})

test("the local admin actor is available only for a bound local-site mutation", () => {
  assert.equal(resolvePageMutationUserId(undefined, LOCAL_SITE, 7, 7, "local"), -1)
  assert.equal(resolvePageMutationUserId(undefined, LOCAL_SITE, 7, 7, "prod"), undefined)
  assert.equal(resolvePageMutationUserId(undefined, LOCAL_SITE, 7, 8, "local"), undefined)
  assert.equal(resolvePageMutationUserId(undefined, "scp-wiki", 7, 7, "local"), undefined)
})
