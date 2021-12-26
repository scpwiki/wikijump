import { fireEvent } from "@testing-library/dom"
import { html } from "@wikijump/util"
import { beforeEach, describe, expect, it } from "vitest"
import * as lib from "../src/index"

describe("@wikijump/dom - focus", () => {
  let div: HTMLDivElement

  const fragment = html`
    <div id="div">
      <button type="button" id="start">1</button>
      <button type="button" id="minus_tabindex" tabindex="-1"><2/button>
      <button type="button" id="is_disabled" disabled>3</button>
      <button type="button" id="end">4</button>
    </div>
  `

  const btn = (id: string) => document.getElementById(id) as HTMLButtonElement

  beforeEach(() => {
    // @ts-ignore
    div = fragment.cloneNode(true).querySelector("#div")
    document.body.innerHTML = ""
    document.body.appendChild(div)
  })

  describe("getFoci", () => {
    it("gets a list", () => {
      const foci = lib.getFoci(div)
      expect(foci).toHaveLength(2)
      expect(foci).not.toContain(btn("minus_tabindex"))
      expect(foci).not.toContain(btn("is_disabled"))
    })

    it("gets list with tabindex=-1", () => {
      const foci = lib.getFoci(div, true)
      expect(foci).toHaveLength(3)
      expect(foci).toContain(btn("minus_tabindex"))
    })
  })

  describe("FocusGroup", () => {
    it("handles home/end", () => {
      const fg = new lib.FocusGroup(div, "vertical")
      btn("start").focus()
      fireEvent.keyDown(div, { key: "End" })
      expect(document.activeElement).toBe(btn("end"))
      fireEvent.keyDown(div, { key: "Home" })
      expect(document.activeElement).toBe(btn("start"))
      fg.destroy()
    })

    it("handles arrow keys (vertical)", () => {
      const fg = new lib.FocusGroup(div, "vertical")
      btn("start").focus()
      fireEvent.keyDown(div, { key: "ArrowUp" })
      expect(document.activeElement).toBe(btn("end"))
      fireEvent.keyDown(div, { key: "ArrowDown" })
      expect(document.activeElement).toBe(btn("start"))
      fireEvent.keyDown(div, { key: "ArrowDown" })
      expect(document.activeElement).toBe(btn("minus_tabindex"))
      fg.destroy()
    })

    it("handles arrow keys (horizontal)", () => {
      const fg = new lib.FocusGroup(div, "horizontal")
      btn("start").focus()
      fireEvent.keyDown(div, { key: "ArrowLeft" })
      expect(document.activeElement).toBe(btn("end"))
      fireEvent.keyDown(div, { key: "ArrowRight" })
      expect(document.activeElement).toBe(btn("start"))
      fireEvent.keyDown(div, { key: "ArrowRight" })
      expect(document.activeElement).toBe(btn("minus_tabindex"))
      fg.destroy()
    })

    it("updates", () => {
      const fg = new lib.FocusGroup(div, "vertical")
      btn("start").focus()
      fireEvent.keyDown(div, { key: "ArrowDown" })
      expect(document.activeElement).toBe(btn("minus_tabindex"))
      fg.update("horizontal")
      fireEvent.keyDown(div, { key: "ArrowLeft" })
      expect(document.activeElement).toBe(btn("start"))
      fg.destroy()
    })
  })

  describe("FocusObserver", () => {
    it("calls on focus", () => {
      let called = false
      const fo = new lib.FocusObserver(div, { focus: () => (called = true) })
      btn("start").focus()
      expect(called).toBe(true)
      fo.destroy()
    })

    it("calls on blur", () => {
      let called = false
      const fo = new lib.FocusObserver(div, { blur: () => (called = true) })
      btn("start").focus()
      btn("start").blur()
      expect(called).toBe(true)
      fo.destroy()
    })

    it("does not call with refocus", () => {
      let count = 0
      const fo = new lib.FocusObserver(div, { focus: () => (count += 1) })
      btn("start").focus()
      expect(count).toBe(1)
      btn("end").focus()
      expect(count).toBe(1)
      btn("end").blur()
      btn("start").focus()
      expect(count).toBe(2)
      fo.destroy()
    })

    it("updates", () => {
      let called1 = false
      let called2 = false
      const fo = new lib.FocusObserver(div, { focus: () => (called1 = true) })
      btn("start").focus()
      expect(called1).toBe(true)
      btn("start").blur()
      fo.update({ focus: () => (called2 = true) })
      btn("end").focus()
      expect(called2).toBe(true)
      fo.destroy()
    })
  })
})
