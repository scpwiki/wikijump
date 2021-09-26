import { defineElement, observe, pauseObservation } from "../../util"

const NEED_TO_POLYFILL = !hasMathMLSupport()

let hfmathPromise: null | Promise<typeof import("hfmath").hfmath> = null

if (NEED_TO_POLYFILL) {
  hfmathPromise = (async () => {
    return (await import("hfmath")).hfmath
  })()
}

export class MathElement extends HTMLSpanElement {
  static tag = "wj-math-ml"

  private declare display: "inline" | "block"
  private declare root: ShadowRoot

  declare observer: MutationObserver

  constructor() {
    super()
    if (!NEED_TO_POLYFILL) {
      throw new Error("shouldn't have been created if no polyfill was needed")
    }

    this.root = this.attachShadow({ mode: "open" })
    this.observer = observe(this, () => this.update())
  }

  private get sourceLatex() {
    return (
      this.parentElement?.querySelector<HTMLElement>(".wj-math-source")?.innerText ?? ""
    )
  }

  @pauseObservation
  private async update() {
    try {
      const hfmath = await hfmathPromise!
      const svg = new hfmath(this.sourceLatex).svg({
        SCALE_X: this.display === "inline" ? 8 : 10,
        SCALE_Y: this.display === "inline" ? 8 : 10,
        MARGIN_X: 0,
        MARGIN_Y: 0,
        STROKE_W: 0.5
      })
      this.root.innerHTML = svg
      const element = this.root.querySelector("svg")!
      element.setAttribute("style", "vertical-align: text-bottom;")
    } catch {
      // TODO: display some sort of error
    }
  }

  connectedCallback() {
    this.display = this.parentElement?.tagName === "DIV" ? "block" : "inline"
    this.update()
  }
}

if (NEED_TO_POLYFILL) {
  defineElement(MathElement.tag, MathElement, { extends: "span" })
}

// function from https://developer.mozilla.org/en-US/docs/Web/MathML/Authoring
/** Returns if the browser has support for MathML. */
export function hasMathMLSupport() {
  let div = document.createElement("div")
  let box: DOMRect
  div.innerHTML = "<math><mspace height='23px' width='77px'/></math>"
  document.body.appendChild(div)
  // @ts-ignore
  box = div.firstChild.firstChild.getBoundingClientRect()
  document.body.removeChild(div)
  return Math.abs(box.height - 23) <= 1 && Math.abs(box.width - 77) <= 1
}
