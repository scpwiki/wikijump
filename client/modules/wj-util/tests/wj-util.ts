import * as uvu from "uvu"
import * as assert from "uvu/assert"

import * as lib from "../src/index"

const Utils = uvu.suite("wj-utils")

Utils("hash", () => {
  const output = lib.hash("test")
  assert.is(output, 3556498)
})

// TODO: search tests

Utils("isEmpty", () => {
  const { isEmpty } = lib
  assert.ok(isEmpty([]))
  assert.ok(isEmpty({}))
  assert.not(isEmpty(["foo"]))
  assert.not(isEmpty({ foo: "foo" }))
  assert.ok(isEmpty(undefined))
})

Utils("has", () => {
  const { has } = lib
  assert.not(has("foo", {}))
  assert.ok(has("foo", { foo: "foo" }))
  assert.not(has("foo", { foo: undefined }))
  assert.not(has("foo", undefined))
})

Utils("removeUndefined", () => {
  const { removeUndefined } = lib
  assert.equal(removeUndefined({ foo: "foo", bar: undefined }), { foo: "foo" })
})

Utils("escapeRegExp", () => {
  const { escapeRegExp } = lib
  const str = ".*+?^${}()|[]\\"
  assert.is(escapeRegExp(str), String.raw`\.\*\+\?\^\$\{\}\(\)\|\[\]\\`)
})

Utils("hasSigil", () => {
  const { hasSigil } = lib
  assert.ok(hasSigil("!foo", "!"))
  assert.ok(hasSigil("!foo", ["$", "!"]))
  assert.not(hasSigil("foo", ["$", "!"]))
})

Utils("unSigil", () => {
  const { unSigil } = lib
  assert.is(unSigil("!foo", "!"), "foo")
  assert.is(unSigil("$!!foo", ["$", "!"]), "foo")
})

Utils("createID", () => {
  const { createID } = lib
  // there isn't really a good test for this lol
  assert.type(createID("foo"), "string")
})

// TODO: toPoints
// TODO: pointsMatch

import { performance } from "perf_hooks"
// a bit hacky, but we have to make sure perfy works in node
globalThis.performance = performance as any

Utils("perfy", () => {
  const { perfy } = lib
  const exec = perfy()
  assert.type(exec(), "number")
})

Utils.run()
