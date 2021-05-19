import * as uvu from "uvu"
import * as assert from "uvu/assert"
import Tabview from "../src/Tabview.svelte"
import { fire, render, reset, setup } from "./setup/setup"

const test = uvu.suite("Tabview-stub")

test.before(setup)
test.before.each(reset)

function generateStub(tabs = 1) {
  const container = document.createElement("div")
  const children: HTMLElement[] = []
  for (let i = 0; i < tabs; i++) {
    const content = document.createElement("div")
    content.className = "wj-tabview-content"
    content.innerHTML = "test content"
    children.push(content)
  }
  container.append(...children)
  return container
}

test("can render stub", () => {
  const { container } = render({ tag: Tabview, container: generateStub() })

  assert.snapshot(
    container.innerHTML,
    `<div class="tabview svelte"><div class="tabs"><div class="tab-selector svelte"> </div></div> <div class="tab-contents"><div class="tab-content selected svelte">test content </div></div></div>`
  )
})

test("can select tab", async () => {
  const { container } = render({ tag: Tabview, container: generateStub(2) })

  // second button
  const button = container.querySelectorAll(".tab-selector")[1]!
  await fire(button, "click")
  assert.snapshot(
    container.innerHTML,
    `<div class="tabview svelte"><div class="tabs"><div class="tab-selector svelte"> </div><div class="tab-selector svelte"> </div></div> <div class="tab-contents"><div class="tab-content  svelte">test content </div><div class="tab-content selected svelte">test content </div></div></div>`
  )
})

test.run()
