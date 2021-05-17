import * as uvu from "uvu"
import * as assert from "uvu/assert"

import * as lib from "../src/index"

const Utils = uvu.suite("wj-utils")

// TODO: search tests
// TODO: toPoints
// TODO: pointsMatch

Utils("hash", () => {
  const output = lib.hash("test")
  assert.is(output, 3556498)
})

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

Utils("perfy", () => {
  const { perfy } = lib
  const exec = perfy()
  assert.type(exec(), "number")
})

Utils("sleep", async () => {
  let flip = false
  const promise = lib.sleep(10).then(() => (flip = true))
  assert.not(flip)
  await promise
  assert.ok(flip)
})

Utils("animationFrame", async () => {
  let flip = false
  const promise = lib.animationFrame().then(() => (flip = true))
  assert.not(flip)
  await promise
  assert.ok(flip)
})

Utils("throttle", async () => {
  let flip = false
  const func = () => {
    assert.not(flip)
    flip = true
  }
  const throttled = lib.throttle(func, 50)
  throttled()
  throttled() // should be ignored, function called too fast
})

Utils("throttle immediate", async () => {
  let count = 0
  const func = () => {
    assert.is.not(count, 2)
    count++
  }
  const throttled = lib.throttle(func, 50, true)
  throttled()
  throttled() // first call doesn't count against throttling
  throttled() // should be ignored, function called too fast
})

Utils("debounce", async () => {
  let count = 0
  const func = () => count++
  const debounced = lib.debounce(func)
  debounced()
  debounced() // ignored
  debounced() // ignored
  await lib.sleep(10)
  assert.is(count, 1)
  debounced()
  debounced() // ignored
  debounced() // ignored
  await lib.sleep(10)
  assert.is(count, 2)
})

Utils("waitFor", async () => {
  let flip = false
  let failed = true

  lib.sleep(50).then(() => (flip = true))
  lib.sleep(150).then(() => assert.not(failed))

  let condition = () => flip === true
  await lib.waitFor(condition)
  failed = false
})

Utils("createLock", async () => {
  const func = async (input: boolean) => {
    assert.is(input, true)
    await lib.sleep(50)
    return Math.random()
  }
  const locked = lib.createLock(func)
  const v1 = locked(true)
  const v2 = locked(true)
  assert.is(await v1, await v2)
})

Utils("createAnimQueued", async () => {
  let flip = false
  const func = () => {
    assert.not(flip)
    flip = true
  }
  const queued = lib.createAnimQueued(func)
  queued()
  queued() // should be ignored, function called too fast
})

Utils("idleCallback", async () => {
  let flip = false
  const promise = lib.idleCallback(() => (flip = true))
  assert.not(flip)
  await promise
  assert.ok(flip)
})

Utils("createIdleQueued", async () => {
  let flip = false
  const func = () => {
    assert.not(flip)
    flip = true
  }
  const queued = lib.createIdleQueued(func)
  queued()
  queued() // should be ignored, function called too fast
})

Utils("toFragment", () => {
  const html = "<div>foo</div>"
  const fragment = lib.toFragment(html)
  assert.instance(fragment, DocumentFragment)
  assert.snapshot(fragment.firstElementChild!.outerHTML, html)
})

Utils("html", () => {
  const fragment = lib.html`<div>${["foo", "bar"]} ${"foo"}</div>`
  assert.instance(fragment, DocumentFragment)
  assert.snapshot(fragment.firstElementChild!.outerHTML, "<div>foobar foo</div>")
})

Utils.run()
