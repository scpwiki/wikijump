import { defineElement, observe, pauseObservation } from "../../util"

export class TabviewElement extends HTMLDivElement {
  static tag = "wj-tabs"

  static get observedAttributes() {
    return ["panel-selected"]
  }

  declare observer: MutationObserver

  constructor() {
    super()
    this.observer = observe(this, () => this.update())
  }

  buttons() {
    const list = this.querySelector(".wj-tabs-button-list")
    if (!list) throw new Error("No button list found")
    return Array.from(list.querySelectorAll<HTMLElement>(".wj-tabs-button"))
  }

  tabs(): [HTMLElement, HTMLElement][] {
    const buttons = this.buttons()
    const list = this.querySelector(".wj-tabs-panel-list")
    if (!list) throw new Error("No panel list found")
    return buttons.map((button, idx) => [button, list.children.item(idx) as HTMLElement])
  }

  @pauseObservation
  update() {
    if (!this.hasAttribute("panel-selected")) {
      let selected = 0
      this.buttons().forEach((button, idx) => {
        if (button.getAttribute("aria-selected") === "true") {
          selected = idx
        }
      })
      this.setAttribute("panel-selected", String(selected))
    }

    const selected = parseInt(this.getAttribute("panel-selected")!, 10)

    this.tabs().forEach(([button, panel], idx) => {
      if (idx === selected) {
        button.setAttribute("aria-selected", "true")
        button.setAttribute("tabindex", "0")
        panel.removeAttribute("hidden")
      } else {
        button.setAttribute("aria-selected", "false")
        button.setAttribute("tabindex", "-1")
        panel.setAttribute("hidden", "true")
      }
    })
  }

  // -- LIFECYCLE

  connectedCallback() {
    this.update()
  }

  attributeChangedCallback() {
    this.update()
  }
}

export class TabviewButton extends HTMLButtonElement {
  static tag = "wj-tabs-button"

  constructor() {
    super()
    this.addEventListener("click", () => {
      const tabview = this.closest<HTMLElement>(".wj-tabs")
      if (!tabview) throw new Error("No tabview found")
      tabview.setAttribute("panel-selected", String(this.index))
    })
  }

  get index() {
    const list = this.closest<HTMLElement>(".wj-tabs-button-list")
    if (!list) throw new Error("No button list found")
    return Array.from(list.children).indexOf(this)
  }
}

defineElement(TabviewElement.tag, TabviewElement, { extends: "div" })
defineElement(TabviewButton.tag, TabviewButton, { extends: "button" })
