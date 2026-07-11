import { describe, expect, it, vi } from "vitest"

vi.mock("@wikijump/prism", () => ({
  default: {
    highlight: vi.fn()
  }
}))

import "../src/components/code/code"
import type { CodeElement } from "../src/components/code/code"

describe("@wikijump/ftml-components - code", () => {
  it("restores unhighlighted code as text", async () => {
    const element = document.createElement("wj-code") as CodeElement
    const code = document.createElement("code")
    code.textContent = `<img src=x onerror="globalThis.__xss = true">`
    element.language = "javascript"
    element.append(code)
    document.body.append(element)

    await (element as any).update()

    expect(code.querySelector("img")).toBeNull()
    expect(code.textContent).toBe(`<img src=x onerror="globalThis.__xss = true">`)
  })
})
