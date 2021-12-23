import { assert, describe, it } from "vitest"
import { LifecycleElement } from "../src/svelte/svelte-lifecycle-element"

// testing the other adapters might not really be possible, at least in this module
// they ofc require a svelte component to work, and even after you'd need an entire
// CodeMirror editor to test it

describe("SheafAdapters", () => {
  it("disconnect element calls callback", async () => {
    const element = new LifecycleElement()
    document.documentElement.append(element)

    let disconnected = false
    element.addEventListener("disconnected", () => (disconnected = true))

    document.documentElement.removeChild(element)

    assert.equal(disconnected, true, "Disconnect element did not fire callback")
  })
})
