import { assert } from "@esm-bundle/chai"
import { fireEvent, render } from "@testing-library/svelte"
import { TabviewStub } from "../src/index"

describe("tabview-stub", () => {
  it("can render stub", async () => {
    const container = generateStub()
    render(TabviewStub, { target: container })
    assert.equal(
      container.innerHTML,
      `<div class="tabview svelte"><div class="tabs"><div class="tab-selector svelte"> </div></div> <div class="tab-contents"><div class="tab-content selected svelte">test content </div></div></div><!--<Tabview>-->`
    )
  })

  it("can select second tab", async () => {
    const container = generateStub(2)
    render(TabviewStub, { target: container })

    // second button
    const button = container.querySelectorAll(".tab-selector")[1]!
    await fireEvent.click(button)

    assert.equal(
      container.innerHTML,
      `<div class="tabview svelte"><div class="tabs"><div class="tab-selector svelte"> </div><div class="tab-selector svelte"> </div></div> <div class="tab-contents"><div class="tab-content  svelte">test content </div><div class="tab-content selected svelte">test content </div></div></div><!--<Tabview>-->`
    )
  })
})

function generateStub(tabs = 1) {
  const container = document.createElement("div")
  const children: HTMLElement[] = []
  for (let i = 0; i < tabs; i++) {
    const content = document.createElement("div")
    content.className = "@wikijump/tabview-content"
    content.innerHTML = "test content"
    children.push(content)
  }
  container.append(...children)
  return container
}
