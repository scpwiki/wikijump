import { addElement, BaseButton, observe, pauseObservation } from "@wikijump/dom"

/**
 * FTML `[[tabview]]` element. Handles ARIA state and visibility of tab
 * panels through the `panel-selected` attribute.
 */
export class TabviewElement extends HTMLElement {
  static tag = "wj-tabs"

  static get observedAttributes() {
    return ["panel-selected"]
  }

  /** Observer for watching changes to the contents of the element. */
  declare observer: MutationObserver

  constructor() {
    super()
    this.observer = observe(this, () => this.update())
  }

  /** The list of tab buttons in this element. */
  private get buttons() {
    const list = this.querySelector(".wj-tabs-button-list")
    if (!list) throw new Error("No button list found")
    return Array.from(list.querySelectorAll<HTMLElement>(".wj-tabs-button"))
  }

  /**
   * An array of arrays, with each array element being a tab button and its
   * corresponding tab panel element.
   */
  private get tabs(): [HTMLElement, HTMLElement][] {
    const list = this.querySelector(".wj-tabs-panel-list")
    if (!list) throw new Error("No panel list found")
    const children = Array.from(list.children) as HTMLElement[]
    return this.buttons.map((button, idx) => [button, children[idx]])
  }

  /**
   * Called whenever the tabs element has mutated or has had the selected
   * panel attribute changed.
   */
  @pauseObservation
  private update() {
    // if we don't have a panel-selected attribute,
    // we'll need to try to find it from the buttons
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

/**
 * FTML `[[tabview]]` tab button. Handles keyboard support and changing the
 * selected tab when clicked.
 */
export class TabviewButtonElement extends BaseButton {
  static tag = "wj-tabs-button"

  /** Parent button list element. */
  private get parent() {
    const parent = this.closest<HTMLElement>(".wj-tabs-button-list")
    if (!parent) throw new Error("No button list found")
    return parent
  }

  /** This button's index. */
  private get index() {
    return Array.from(this.parent.children).indexOf(this)
  }

  /**
   * Fired when the button is clicked. Changes the parent tabs
   * `panel-selected` attribute to match this button's index.
   */
  whenClicked() {
    const tabview = this.closest<HTMLElement>(".wj-tabs")
    if (!tabview) throw new Error("No tabview found")
    tabview.setAttribute("panel-selected", String(this.index))
  }

  /**
   * Fired on keydown events. This function handles accessibility support
   * for keyboards.
   */
  whenKeydown(evt: KeyboardEvent) {
    if (["ArrowRight", "ArrowLeft", "Home", "End"].includes(evt.key)) {
      const list = this.relativeList()

      // prettier-ignore
      switch(evt.key) {
        case "ArrowRight": list.next.focus()  ; break
        case "ArrowLeft":  list.prev.focus()  ; break
        case "Home":       list.start.focus() ; break
        case "End":        list.end.focus()   ; break
      }

      evt.preventDefault()
    }
  }

  /**
   * Returns an object that contains info about the parent list, e.g. the
   * button after this one.
   */
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

declare global {
  interface HTMLElementTagNameMap {
    "wj-tabs": TabviewElement
    "wj-tabs-button": TabviewButtonElement
  }

  interface Window {
    TabviewElement: typeof TabviewElement
    TabviewButtonElement: typeof TabviewButtonElement
  }
}

addElement(TabviewElement, "TabviewElement")
addElement(TabviewButtonElement, "TabviewButtonElement")
