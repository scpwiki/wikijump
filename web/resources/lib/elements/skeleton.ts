import { addElement } from "@wikijump/dom"
import { html } from "@wikijump/util"

export class SkeletonBlockElement extends HTMLElement {
  static tag = "wj-skeleton-block"

  static get observedAttributes() {
    return ["height", "width"]
  }

  constructor() {
    super()
    this.update()
  }

  get height() {
    return this.getAttribute("height") ?? "auto"
  }

  get width() {
    return this.getAttribute("width") ?? "auto"
  }

  private update() {
    this.innerHTML = ""
    const template = html`
      <div
        class="skeleton is-block"
        style="height: ${this.height}; width: ${this.width}"
      ></div>
    `
    this.appendChild(template.cloneNode(true))
  }

  // -- LIFECYCLE

  attributeChangedCallback(name: string) {
    if (name === "height" || name === "width") this.update()
  }
}

export class SkeletonInlineElement extends HTMLElement {
  static tag = "wj-skeleton-inline"

  static get observedAttributes() {
    return ["lines", "height"]
  }

  constructor() {
    super()
    this.update()
  }

  get lines() {
    return parseInt(this.getAttribute("lines") ?? "1", 10)
  }

  get height() {
    return this.getAttribute("height") ?? "1em"
  }

  private getLineArray() {
    const arr: string[] = []
    const height = this.height
    for (let i = 0; i < this.lines; i++) {
      arr.push(`
        <span
          class="skeleton is-line"
          style="height: ${height};">
        </span>
      `)
    }
    return arr
  }

  private update() {
    this.innerHTML = ""
    const template = html`${this.getLineArray()}`
    this.appendChild(template.cloneNode(true))
  }

  // -- LIFECYCLE

  attributeChangedCallback(name: string) {
    if (name === "lines" || name === "height") this.update()
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wj-skeleton-block": SkeletonBlockElement
    "wj-skeleton-inline": SkeletonInlineElement
  }

  interface Window {
    SkeletonBlockElement: typeof SkeletonBlockElement
    SkeletonInlineElement: typeof SkeletonInlineElement
  }
}

addElement(SkeletonBlockElement, "SkeletonBlockElement")
addElement(SkeletonInlineElement, "SkeletonInlineElement")
