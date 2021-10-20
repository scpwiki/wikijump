import { addElement, hover } from "@wikijump/dom"

/** Handles hovering, focus for navbar dropdowns. */
export class NavbarDropdownElement extends HTMLElement {
  static tag = "wj-navbar-dropdown"

  constructor() {
    super()

    hover(this, {
      alsoOnFocus: true,
      on: () => (this.details.open = true),
      off: () => (this.details.open = false)
    })

    this.addEventListener("keydown", evt => this.handleKeydown(evt))

    // clicking conflicts with the hover/focus handlers,
    // so we need to add a click handler to the summary element
    this.summary.addEventListener("click", evt => evt.preventDefault())
  }

  /** The dropdown's internal `<details>` element. */
  private get details() {
    const el = this.querySelector("details")
    if (!el) throw new Error("No details element found")
    return el
  }

  /** The dropdown's internal `<summary>` (aka the button) element. */
  private get summary() {
    const summary = this.querySelector("summary")
    if (!summary) throw new Error("No summary element found")
    return summary
  }

  /** Finds every link in this dropdown's list. */
  private findLinks() {
    return Array.from(this.querySelectorAll<HTMLElement>(".wj-navbar-dropdown-link"))
  }

  /** Handler for `keydown` events for the entire component. */
  private handleKeydown(evt: KeyboardEvent) {
    const focused = document.activeElement as HTMLElement | null

    // close menu on escape
    if (evt.key === "Escape") {
      this.details.open = false
      focused?.blur()
    }

    // going up and down in the list
    else if (["ArrowUp", "ArrowDown", "Home", "End"].includes(evt.key)) {
      const links = this.findLinks()
      if (links.length === 0) return

      const index =
        focused && links.includes(focused)
          ? links.indexOf(focused)
          : evt.key === "ArrowDown"
          ? links.length - 1
          : 0

      const frst = links[0]
      const last = links[links.length - 1]
      const next = links[index + 1] ?? frst
      const prev = links[index - 1] ?? last

      // prettier-ignore
      switch(evt.key) {
        case "ArrowUp":   prev.focus(); break
        case "ArrowDown": next.focus(); break
        case "Home":      frst.focus(); break
        case "End":       last.focus(); break
      }

      evt.preventDefault()
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wj-navbar-dropdown": NavbarDropdownElement
  }

  interface Window {
    NavbarDropdownElement: typeof NavbarDropdownElement
  }
}

addElement(NavbarDropdownElement, "NavbarDropdownElement")
