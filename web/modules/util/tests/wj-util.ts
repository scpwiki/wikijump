import { assert, describe, it } from "vitest"
import * as lib from "../src/index"

// TODO: search tests
// TODO: toPoints
// TODO: pointsMatch

describe("@wikijump/util", () => {
  it("hash", () => {
    const output = lib.hash("test")
    assert.equal(output, 3556498)
  })

  it("isEmpty", () => {
    const { isEmpty } = lib
    assert.ok(isEmpty([]))
    assert.ok(isEmpty({}))
    assert.notOk(isEmpty(["foo"]))
    assert.notOk(isEmpty({ foo: "foo" }))
    assert.ok(isEmpty(undefined))
  })

  it("has", () => {
    const { has } = lib
    assert.notOk(has("foo", {}))
    assert.ok(has("foo", { foo: "foo" }))
    assert.notOk(has("foo", { foo: undefined }))
    assert.notOk(has("foo", undefined))
  })

  it("removeUndefined", () => {
    const { removeUndefined } = lib
    // @ts-ignore
    assert.deepEqual(removeUndefined({ foo: "foo", bar: undefined }), { foo: "foo" })
  })

  it("escapeRegExp", () => {
    const { escapeRegExp } = lib
    const str = ".*+?^${}()|[]\\"
    assert.equal(escapeRegExp(str), String.raw`\.\*\+\?\^\$\{\}\(\)\|\[\]\\`)
  })

  it("hasSigil", () => {
    const { hasSigil } = lib
    assert.ok(hasSigil("!foo", "!"))
    assert.ok(hasSigil("!foo", ["$", "!"]))
    assert.notOk(hasSigil("foo", ["$", "!"]))
  })

  it("unSigil", () => {
    const { unSigil } = lib
    assert.equal(unSigil("!foo", "!"), "foo")
    assert.equal(unSigil("$!!foo", ["$", "!"]), "foo")
  })

  it("createID", () => {
    const { createID } = lib
    // there isn't really a good test for this lol
    assert.isString(createID("foo"))
  })

  it("perfy", () => {
    const { perfy } = lib
    const exec = perfy()
    assert.isNumber(exec())
  })

  it("sleep", async () => {
    let flip = false
    const promise = lib.sleep(10).then(() => (flip = true))
    assert.notOk(flip)
    await promise
    assert.ok(flip)
  })

  it("animationFrame", async () => {
    let flip = false
    const promise = lib.animationFrame().then(() => (flip = true))
    assert.notOk(flip)
    await promise
    assert.ok(flip)
  })

  it("throttle", async () => {
    let flip = false
    const func = () => {
      assert.notOk(flip)
      flip = true
    }
    const throttled = lib.throttle(func, 50)
    throttled()
    throttled() // should be ignored, function called too fast
  })

  it("throttle immediate", async () => {
    let count = 0
    const func = () => {
      assert.notEqual(count, 2)
      count++
    }
    const throttled = lib.throttle(func, 50, true)
    throttled()
    throttled() // first call doesn't count against throttling
    throttled() // should be ignored, function called too fast
  })

  it("debounce", async () => {
    let count = 0
    const func = () => count++
    const debounced = lib.debounce(func)
    debounced()
    debounced() // ignored
    debounced() // ignored
    await lib.sleep(10)
    assert.equal(count, 1)
    debounced()
    debounced() // ignored
    debounced() // ignored
    await lib.sleep(10)
    assert.equal(count, 2)
  })

  it("waitFor", async () => {
    let flip = false
    let failed = true

    lib.sleep(50).then(() => (flip = true))
    lib.sleep(150).then(() => assert.notOk(failed))

    let condition = () => flip === true
    await lib.waitFor(condition)
    failed = false
  })

  it("createLock", async () => {
    let busy = false
    const func = async (input: boolean) => {
      assert.equal(input, true)
      assert.notOk(busy)
      busy = true
      await lib.sleep(50)
      busy = false
    }
    const locked = lib.createLock(func)
    locked(true)
    locked(true)
  })

  it("createAnimQueued", async () => {
    let flip = false
    const func = () => {
      assert.notOk(flip)
      flip = true
    }
    const queued = lib.createAnimQueued(func)
    queued()
    queued() // should be ignored, function called too fast
  })

  it("idleCallback", async () => {
    let flip = false
    const promise = lib.idleCallback(() => (flip = true))
    assert.notOk(flip)
    await promise
    assert.ok(flip)
  })

  it("createIdleQueued", async () => {
    let flip = false
    const func = () => {
      assert.notOk(flip)
      flip = true
    }
    const queued = lib.createIdleQueued(func)
    queued()
    queued() // should be ignored, function called too fast
  })

  it("toFragment", () => {
    const html = "<div>foo</div>"
    const fragment = lib.toFragment(html)
    assert.instanceOf(fragment, DocumentFragment)
    assert.equal(fragment.firstElementChild!.outerHTML, html)
  })

  it("html", () => {
    const fragment = lib.html`<div>${["foo", "bar"]} ${"foo"}</div>`
    assert.instanceOf(fragment, DocumentFragment)
    assert.equal(fragment.firstElementChild!.outerHTML, "<div>foobar foo</div>")
  })
})
