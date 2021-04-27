import { JSDOM } from "jsdom"
import { SvelteComponent, tick } from "svelte"

const { window } = new JSDOM("")

export function setup() {
  // @ts-ignore
  globalThis.window = window
  globalThis.document = window.document
  globalThis.navigator = window.navigator
  globalThis.getComputedStyle = window.getComputedStyle
  globalThis.requestAnimationFrame = null
}

export function reset() {
  window.document.title = ""
  window.document.head.innerHTML = ""
  window.document.body.innerHTML = ""
}

export interface RenderOpts {
  tag: typeof SvelteComponent
  props?: Record<string, any>
  container?: Element
}

export function render({
  tag,
  props = {},
  container = window.document.body
}: RenderOpts) {
  // @ts-ignore
  tag = tag.default || tag
  const component = new tag({ props, target: container })
  return { container, component }
}

export function fire(elem: Element, event: string, details?: EventInit) {
  let evt = new window.Event(event, details)
  elem.dispatchEvent(evt)
  return tick()
}
