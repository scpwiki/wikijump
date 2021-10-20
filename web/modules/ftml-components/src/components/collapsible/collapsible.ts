import { addElement, BaseButton } from "@wikijump/dom"

/**
 * Button that shows up at the bottom of a FTML `[[collapsible]]` block.
 * Closes the collapsible when clicked.
 */
export class CollapsibleBottomButtonElement extends BaseButton {
  static tag = "wj-collapsible-button-bottom"

  get details() {
    const details = this.closest("details.wj-collapsible")
    if (!details) throw new Error("No details found")
    return details as HTMLElement
  }

  whenClicked() {
    this.details.removeAttribute("open")
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wj-collapsible-button-bottom": CollapsibleBottomButtonElement
  }

  interface Window {
    CollapsibleBottomButtonElement: typeof CollapsibleBottomButtonElement
  }
}

addElement(CollapsibleBottomButtonElement, "CollapsibleBottomButtonElement")
