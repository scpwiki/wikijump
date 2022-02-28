import { addElement, BaseTooltipButton, observe, pauseObservation } from "@wikijump/dom"
import { animationFrame } from "@wikijump/util"

const NEED_TO_POLYFILL = !hasMathMLSupport()

let hfmathPromise: null | Promise<typeof import("hfmath").hfmath> = null

if (NEED_TO_POLYFILL) {
  hfmathPromise = (async () => {
    return (await import("hfmath")).hfmath
  })()
}

/**
 * FTML `[[math]]` or `[[$ inline $]]` element. This element is only
 * created when polyfilling for MathML is needed.
 */
export class MathElement extends HTMLElement {
  static tag = "wj-math-ml"

  /** Display mode of the element. */
  private declare display: "inline" | "block"

  /** `ShadowRoot` of the node. */
  private declare root: ShadowRoot

  /** The element in which the polyfilled SVG element is placed inside of. */
  private declare container: HTMLElement

  /** Observer for watching changes to the contents of the element. */
  declare observer: MutationObserver

  constructor() {
    super()
    if (!NEED_TO_POLYFILL) {
      throw new Error("shouldn't have been created if no polyfill was needed")
    }

    this.root = this.attachShadow({ mode: "open" })

    // polyfilled SVG element goes inside of this container
    this.container = document.createElement("span")
    this.container.setAttribute("style", "display: inline-block;")
    this.container.setAttribute("aria-hidden", "true")
    this.root.appendChild(this.container)

    // MathML element automatically goes into this slot
    this.root.append(document.createElement("slot"))

    this.observer = observe(this, () => this.update())
  }

  /** The source LaTeX string that this math element was rendered from. */
  private get sourceLatex() {
    return this.parentElement?.querySelector<HTMLElement>(".wj-math-source")?.innerText
  }

  /** Ran whenever the element is mutated. */
  @pauseObservation
  private async update() {
    // we make sure to keep this class
    // it's how we style the MathML element to be visually hidden
    // but still accessible to screen readers
    this.classList.add("wj-math-ml-polyfilled")

    const latex = await animationFrame(() => this.sourceLatex)
    if (!latex) return

    try {
      const hfmath = await hfmathPromise!
      await animationFrame(() => {
        const svg = new hfmath(latex).svg({
          SCALE_X: 7.5,
          SCALE_Y: 7.5,
          MARGIN_X: 0,
          MARGIN_Y: 0
        })
        this.container.innerHTML = svg
        const element = this.container.querySelector("svg")!
        // align SVG with surrounding text, set color to the current text color
        element.setAttribute(
          "style",
          "vertical-align: text-bottom; stroke: currentColor;"
        )
      })
    } catch (err) {
      // display an error message if something goes wrong
      const message = err instanceof Error ? err.message : "unknown error"
      const error = document.createElement("span")
      error.setAttribute("class", `wj-error-${this.display}`)
      error.innerText = message
      this.container.innerHTML = ""
      this.container.append(error)
    }
  }

  // -- LIFECYCLE

  connectedCallback() {
    this.display = this.parentElement?.tagName === "DIV" ? "block" : "inline"
    this.update()
  }
}

/**
 * FTML `[[eqref]]` block. Handles scrolling to an equation when clicked,
 * and an equation tooltip that shows on hover.
 */
export class EquationRefMarkerElement extends BaseTooltipButton {
  static tag = "wj-equation-ref-marker"

  get parent() {
    if (!this.parentElement) throw new Error("No parent element")
    return this.parentElement
  }

  get tooltip() {
    const element = this.parent.querySelector(".wj-equation-ref-tooltip")
    if (!element) throw new Error("No tooltip element")
    return element as HTMLElement
  }

  whenClicked() {
    const equation = this.getUpdatedEquation()
    if (!equation) return
    equation.scrollIntoView({ block: "center" })
    equation.focus()
  }

  whenHovered() {
    if (!this.getUpdatedEquation()) return false
  }

  /**
   * Tries to find this reference's equation. If it does, it will update
   * the tooltip and return the equation element.
   */
  private getUpdatedEquation() {
    const label = this.getAttribute("data-name")

    if (!label) {
      this.classList.toggle("is-no-equation", true)
      return null
    }

    const eq = this.closest(".wj-body")?.querySelector(`.wj-math[data-name="${label}"]`)

    if (!eq) {
      this.classList.toggle("is-no-equation", true)
      return null
    }

    // this is a bit wacky, but it works for now.
    // the `wj-math-ml` component needs the `wj-math-source` element
    const source = eq.querySelector(".wj-math-source")!.cloneNode(true)
    const math = eq.querySelector(".wj-math-ml")!.cloneNode(true)
    this.tooltip.replaceChildren(source, math)

    this.classList.toggle("is-no-equation", false)

    return eq as HTMLElement
  }

  // -- LIFECYCLE

  connectedCallback() {
    super.connectedCallback()
    this.getUpdatedEquation()
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wj-math-ml": MathElement
    "wj-equation-ref-marker": EquationRefMarkerElement
  }

  interface Window {
    MathElement: typeof MathElement
    EquationRefMarkerElement: typeof EquationRefMarkerElement
  }
}

if (NEED_TO_POLYFILL) addElement(MathElement, "MathElement")
addElement(EquationRefMarkerElement, "EquationRefMarkerElement")

// function from https://developer.mozilla.org/en-US/docs/Web/MathML/Authoring
/** Returns if the browser has support for MathML. */
function hasMathMLSupport() {
  let div = document.createElement("div")
  let box: DOMRect
  div.innerHTML = "<math><mspace height='23px' width='77px'/></math>"
  document.body.appendChild(div)
  // @ts-ignore
  box = div.firstChild.firstChild.getBoundingClientRect()
  document.body.removeChild(div)
  return Math.abs(box.height - 23) <= 1 && Math.abs(box.width - 77) <= 1
}
