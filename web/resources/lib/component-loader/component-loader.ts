import { addElement, DisplayObserver } from "@wikijump/dom"
import { detach, insert, noop, type SvelteComponent } from "svelte/internal"
import Skeleton from "../components/Skeleton.svelte"
import ComponentManager, { type ComponentName } from "./component-manager"

/**
 * HTML element that can be used to asynchronously load and render
 * components. This is used to simplify component handling in Blade templates.
 */
export class ComponentLoaderElement extends HTMLElement {
  static tag = "wj-component-loader"

  /** Observer used for checking if the loader is being displayed in the DOM or not. */
  private declare observer: DisplayObserver

  /** The original HTML of the element. */
  html: string | null = null

  /** True if the component has already been rendered. */
  rendered = false

  /**
   * The type of rendered component. `null` if the component has not been
   * rendered yet.
   */
  type: null | "html" | "svelte" = null

  /**
   * If the component rendered is a Svelte component, this will be the
   * rendered component. Otherwise, it will be `null`. This will be `null`
   * until the component is rendered.
   */
  component: null | SvelteComponent = null

  constructor() {
    super()
    this.observer = new DisplayObserver(this, {
      visible: () => this.render(),
      hidden: () => this.destroy()
    })
  }

  /** The name of the component that will be rendered. */
  get load(): ComponentName | null {
    return this.getAttribute("ld-load") as any
  }

  /**
   * Parses the `skeleton` attribute that may be set on the element. This
   * attribute has the form of `type:height?:width?`. If the `type` is
   * `inline`, then it is instead of the form `type:lines?:height?`.
   */
  private parseSkeletonAttribute():
    | { type: "block" | "spinner"; height: string; width: string }
    | { type: "inline"; lines: number; height: string }
    | null {
    const attr = this.getAttribute("ld-skeleton")

    if (attr === null) return null

    const [type, arg1, arg2] = attr.split(":")

    if (type === "block" || type === "spinner") {
      return { type, height: arg1 ?? "2rem", width: arg2 ?? "100%" }
    } else if (type === "inline") {
      return { type, lines: parseInt(arg1 ?? "1", 10), height: arg2 ?? "1em" }
    }

    return null
  }

  /** Mounts a skeleton to the loader element, and returns it. */
  private mountSkeleton() {
    const opts = this.parseSkeletonAttribute()

    if (!opts) return null

    if (opts.type === "block" || opts.type === "spinner") {
      const { type, height, width } = opts
      const element = new Skeleton({ target: this, props: { type, height, width } })
      return element
    } else if (opts.type === "inline") {
      const { type, lines, height } = opts
      const element = new Skeleton({ target: this, props: { type, lines, height } })
      return element
    }

    return null
  }

  /** Begins the loading and rendering of the named component. */
  private async loadComponent() {
    // set this here to prevent race conditions,
    // even though it's a bit early
    this.rendered = true

    let skeleton: Skeleton | null = null

    if (this.hasAttribute("ld-skeleton")) {
      this.innerHTML = ""
      skeleton = this.mountSkeleton()
    }

    // this will error if we load a bad component, so
    // we don't need to do any error handling here
    const component = await ComponentManager.load(this.load!)

    // avoid a possible race:
    // if the loader was taken out of the DOM, we don't want to continue
    if (!this.isConnected) return

    const attributeNames = new Set(this.getAttributeNames())

    // get rid of our attributes
    attributeNames.delete("ld-load")
    attributeNames.delete("ld-skeleton")

    // style is weird, let's not pass it on
    attributeNames.delete("style")

    // dismount the skeleton if it was mounted
    if (skeleton) {
      skeleton.$destroy()
      this.innerHTML = this.html!
    }

    // now we need to handle slotted content
    // this is going to be using WITCHCRAFT
    //
    // there are two supported ways of doing this:
    // 1. HTMLElement based component, which is simple
    // 2. Svelte component, which is very complex
    //
    // the latter involves a method taken from
    // the `svelte-tag` library, which I will be using shamelessly.
    // I do not pretend to fully comprehend it

    // HTML element based component is very simple, thankfully
    if (isHTMLComponent(component)) {
      this.type = "html"

      const element = new component()

      for (const name of attributeNames) {
        element.setAttribute(name, this.getAttribute(name)!)
      }

      element.innerHTML = this.innerHTML
      this.innerHTML = ""
      this.appendChild(element)
    }

    // svelte is not so simple
    // the component given will actually be a Proxy,
    // so we just have to presume that it is a Svelte component
    // I can't think of any way to safely check what the component is
    else {
      this.type = "svelte"

      // we are going to get all the props
      // and parse them, because Svelte expects all kinds of props,
      // not just strings/html-attributes

      const props: Record<string, any> = {}

      for (const name of attributeNames) {
        let value: any = this.getAttribute(name)

        // keywords
        if (value === "undefined") value = undefined
        else if (value === "null") value = null
        else if (value === "true") value = true
        else if (value === "false") value = false
        // special case: boolean attribute
        // e.g. <div hidden>
        else if (value === "" || value === null) value = true
        // integers or floats
        else if (value.match(/^\d+$/)) value = parseInt(value, 10)
        else if (value.match(/^\d+\.\d+$/)) value = parseFloat(value)
        // JSON based parsing, e.g. "{}" or "[]"
        else if (value.match(/^\{.*\}$/)) value = JSON.parse(value)
        else if (value.match(/^\[.*\]$/)) value = JSON.parse(value)

        // at this point, we've done all we can to parse the value
        props[name] = value
      }

      // here is the svelte witchcraft
      props.$$scope = {}
      const slots = takeSlots(this)
      props.$$slots = createSvelteSlotHandler(slots)

      // now we can _finally_ mount the component
      // @ts-ignore - SvelteComponent doesn't type its constructor
      const rendered = new component({ target: this, props })

      this.component = rendered
    }
  }

  /** Renders the named component. */
  private render() {
    if (this.rendered) return

    if (!this.getAttribute("ld-load")) {
      throw new Error("Component name is required")
    }

    this.loadComponent()
  }

  /** Destroys the previously rendered component. */
  private destroy() {
    if (this.rendered && this.type === "svelte" && this.component) {
      // prettier-ignore
      try { this.component.$destroy() } catch {}
    } else if (this.rendered && this.type === "html") {
      this.innerHTML = ""
    }

    this.rendered = false
    this.type = null
    this.component = null
  }

  // -- LIFECYCLE

  connectedCallback() {
    // when connected, make sure our innerHTML is always
    // what it was when we first mounted
    if (this.html) this.innerHTML = this.html
    else this.html = this.innerHTML

    if (this.observer.visible) this.render()
  }

  disconnectedCallback() {
    if (this.rendered) this.destroy()
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wj-component-loader": ComponentLoaderElement
  }

  interface Window {
    ComponentLoaderElement: typeof ComponentLoaderElement
  }
}

addElement(ComponentLoaderElement, "ComponentLoaderElement")

// utility functions for getting this all to work

/**
 * Checks if the given class extends from `HTMLElement`.
 *
 * @param component - The class to check.
 */
function isHTMLComponent(component: any): component is typeof HTMLElement {
  return component.prototype instanceof HTMLElement
}

/**
 * Takes an `Element` and decontructs everything inside of it into a
 * `DocumentFragment`, then returns the fragment. All children of the given
 * element will be removed from the element.
 *
 * Taken from `svelte-tag`.
 *
 * @param from - The element to be decontructed.
 */
function unwrap(from: Element) {
  let node = document.createDocumentFragment()
  while (from.firstChild) {
    node.appendChild(from.removeChild(from.firstChild))
  }
  return node
}

/**
 * Gets all of the slots inside of this element, destructively.
 *
 * Taken from `svelte-tag`.
 *
 * @param from - The element to deconstruct the slots from.
 */
function takeSlots(from: Element) {
  const namedSlots = from.querySelectorAll("[slot]")

  let slots: Record<string, DocumentFragment> = {}

  namedSlots.forEach(n => {
    slots[n.slot] = unwrap(n)
    from.removeChild(n)
  })

  // anything left over is the default slot
  if (from.innerHTML.length) {
    slots.default = unwrap(from)
    from.innerHTML = ""
  }

  return slots
}

/**
 * Creates a Svelte compatible slot handler object. Witchcraft!
 *
 * Taken from `svelte-tag`.
 *
 * @param slots - The slots to be used.
 */
function createSvelteSlotHandler(slots: Record<string, DocumentFragment>) {
  const svelteSlots: Record<string, [ReturnType<typeof createSvelteSlotFunction>]> = {}

  for (const slotName in slots) {
    svelteSlots[slotName] = [createSvelteSlotFunction(slots[slotName])]
  }

  return svelteSlots
}

/**
 * Creates a Svelte compatible slot function. Witchcraft!
 *
 * Taken from `svelte-tag`.
 *
 * @param fragment - The document fragment to be used as the slot.
 */
function createSvelteSlotFunction(fragment: DocumentFragment) {
  return function () {
    return {
      c: noop,
      m: function mount(target: Node, anchor?: Node) {
        insert(target, fragment.cloneNode(true), anchor)
      },
      d: function destroy(detaching: boolean) {
        // @ts-ignore
        if (detaching && fragment.innerHTML) {
          detach(fragment)
        }
      },
      l: noop
    }
  }
}
