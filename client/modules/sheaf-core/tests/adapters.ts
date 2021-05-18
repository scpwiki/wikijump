import * as uvu from "uvu"
import * as assert from "uvu/assert"
import { LifecycleElement } from "../src/adapters/svelte-lifecycle-element"

// testing the other adapters might not really be possible, at least in this module
// they ofc require a svelte component to work, and even after you'd need an entire
// CodeMirror editor to test it
//
// for right now, it's probably not worth it to try to get CodeMirror to fully mount
// in JSDOM, considering how much CodeMirror uses visual interfaces of the DOM
// that JSDOM has trouble with

const Adapters = uvu.suite("SheafAdapters")

Adapters("disconnect element calls callback", async () => {
  const element = new LifecycleElement()
  document.documentElement.append(element)

  let disconnected = false
  element.addEventListener("disconnected", () => (disconnected = true))

  document.documentElement.removeChild(element)

  assert.is(disconnected, true, "Disconnect element did not fire callback")
})

Adapters.run()
