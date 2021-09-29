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

  private get buttons() {
    const list = this.querySelector(".wj-tabs-button-list")
    if (!list) throw new Error("No button list found")
    return Array.from(list.querySelectorAll<HTMLElement>(".wj-tabs-button"))
  }

  private get tabs(): [HTMLElement, HTMLElement][] {
    const list = this.querySelector(".wj-tabs-panel-list")
    if (!list) throw new Error("No panel list found")
    const children = Array.from(list.children) as HTMLElement[]
    return this.buttons.map((button, idx) => [button, children[idx]])
  }

  @pauseObservation
  private update() {
    if (!this.hasAttribute("panel-selected")) {
      let selected = 0
      this.buttons.forEach((button, idx) => {
        if (button.getAttribute("aria-selected") === "true") {
          selected = idx
        }
      })
      this.setAttribute("panel-selected", String(selected))
    }

    const selected = parseInt(this.getAttribute("panel-selected")!, 10)

    this.tabs.forEach(([button, panel], idx) => {
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

    this.addEventListener("keydown", evt => {
      if (["ArrowRight", "ArrowLeft", "Home", "End", "Enter"].includes(evt.key)) {
        const list = this.relativeList()
        // prettier-ignore
        switch(evt.key) {
          case "ArrowRight": list.next.focus()  ; break
          case "ArrowLeft":  list.prev.focus()  ; break
          case "Home":       list.start.focus() ; break
          case "End":        list.end.focus()   ; break
          case "Enter":      this.click()       ; break
        }

        evt.preventDefault()
      }
    })
  }

  private get parent() {
    const parent = this.closest<HTMLElement>(".wj-tabs-button-list")
    if (!parent) throw new Error("No button list found")
    return parent
  }

  private get index() {
    return Array.from(this.parent.children).indexOf(this)
  }

  private relativeList() {
    const children = Array.from(this.parent.children) as HTMLElement[]
    const idx = children.indexOf(this)
    return {
      start: children[0],
      end: children[children.length - 1],
      prev: children[idx - 1],
      next: children[idx + 1]
    }
  }
}

defineElement(TabviewElement.tag, TabviewElement, { extends: "div" })
defineElement(TabviewButton.tag, TabviewButton, { extends: "button" })
