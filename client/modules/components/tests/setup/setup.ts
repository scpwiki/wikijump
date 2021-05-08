import { SvelteComponent, tick } from "svelte"

export function setup() {}

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
