import { fireEvent } from "@testing-library/dom"
import { html, sleep } from "@wikijump/util"
import { beforeEach, describe, expect, it } from "vitest"
import * as lib from "../src/index"

describe("@wikijump/dom - misc", () => {
  describe("HeldObserver", () => {
    let button: HTMLButtonElement

    const fragment = html`<button type="button"></button>`

    beforeEach(() => {
      // @ts-ignore
      button = fragment.cloneNode(true).querySelector("button")
      document.body.innerHTML = ""
      document.body.appendChild(button)
    })

    it("handles press and release", () => {
      let pressed = false
      let released = false
      const ho = new lib.HeldObserver(button, {
        pressed: () => (pressed = true),
        released: () => (released = true)
      })
      fireEvent.pointerDown(button)
      expect(pressed).toBe(true)
      expect(released).toBe(false)
      fireEvent.pointerUp(button)
      expect(pressed).toBe(true)
      expect(released).toBe(true)
      ho.destroy()
    })

    it("updates", () => {
      let called1 = false
      let called2 = false
      const ho = new lib.HeldObserver(button, { pressed: () => (called1 = true) })
      fireEvent.pointerDown(button)
      expect(called1).toBe(true)
      fireEvent.pointerUp(button)
      ho.update({ pressed: () => (called2 = true) })
      fireEvent.pointerDown(button)
      expect(called2).toBe(true)
      ho.destroy()
    })
  })

  it("observe", async () => {
    const div = document.createElement("div")
    document.body.append(div)
    let called = false
    const ob = lib.observe(div, () => (called = true))
    expect(ob).toBeInstanceOf(MutationObserver)
    expect(called).toBe(false)
    div.append(document.createElement("div"))
    // have to wait for the observer to fire
    await sleep(0)
    expect(called).toBe(true)
    document.body.removeChild(div)
    ob.disconnect()
  })

  it("addElement", () => {
    const CustomElement = class extends HTMLElement {
      static tag = "custom-element"
    }
    lib.addElement(CustomElement, "CustomElement")
    const ce = document.createElement("custom-element")
    expect(ce).toBeInstanceOf(CustomElement)
    expect("CustomElement" in globalThis).toBe(true)
  })

  it("upgrade", () => {
    const fragment = html`
      <div>
        <unloaded-element></unloaded-element>
      </div>
    `
    const div = fragment.querySelector<HTMLDivElement>("div")!
    const ue = div.querySelector<HTMLUnknownElement>("unloaded-element")!
    const NewElement = class extends HTMLElement {
      static tag = "unloaded-element"
    }
    lib.addElement(NewElement)
    expect(ue).not.toBeInstanceOf(NewElement)
    lib.upgrade(div, NewElement)
    expect(ue).toBeInstanceOf(NewElement)
  })

  it("UserAgent", () => {
    expect(lib.UserAgent.isMobile).toBe(false)
    expect(typeof lib.UserAgent.mouseX).toBe("number")
    expect(typeof lib.UserAgent.mouseY).toBe("number")
    expect(typeof lib.UserAgent.scroll).toBe("number")
  })

  it("inputsValid", () => {
    const fragment = html`
      <div>
        <input type="text" value="foo" />
        <input type="text" value="bar" />
        <input type="text" value="baz" />
      </div>
    `
    const div = fragment.querySelector<HTMLDivElement>("div")!
    const inputs = Array.from(div.querySelectorAll<HTMLInputElement>("input"))
    expect(lib.inputsValid(...inputs)).toBe(true)
    inputs[0].required = true
    inputs[0].value = ""
    expect(lib.inputsValid(...inputs)).toBe(false)
    inputs[0].required = false
    inputs[0].value = "foo"
    inputs[0].disabled = true
    expect(lib.inputsValid(...inputs)).toBe(false)
    inputs[0].disabled = false
    inputs[0].readOnly = true
    expect(lib.inputsValid(...inputs)).toBe(false)
    inputs[0].readOnly = false
    expect(lib.inputsValid(...inputs)).toBe(true)
  })
})
