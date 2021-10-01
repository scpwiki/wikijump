import * as Popper from "@popperjs/core"
import { clearTimeout, timeout, Timeout } from "@wikijump/util"
import { BaseButton } from "./base-button"
import { hover } from "./util"

// TODO: proper mobile support (need more infrastructure for mobile support)

export abstract class BaseTooltipButton extends BaseButton {
  abstract get parent(): HTMLElement
  abstract get tooltip(): HTMLElement
  protected whenHovered?(): void | boolean
  protected whenUnhovered?(): void

  /** Timer to keep track of the delay for revealing the tooltip. */
  private declare onTimer?: Timeout

  /** Timer to keep track of the delay for hiding the tooltip. */
  private declare offTimer?: Timeout

  /** The Popper.js instance for handling placement of the tooltip. */
  private declare popperInstance?: Popper.Instance

  /** Fired when the element is hovered over. */
  private baseWhenHovered() {
    if (this.disabled) return

    if (this.whenHovered) {
      const result = this.whenHovered()
      if (result === false) return
    }

    this.tooltip.classList.add("is-hovered")

    if (!this.popperInstance) {
      // @ts-ignore Popper has some bad typings (Element !== HTMLElement)
      this.popperInstance = Popper.createPopper(this.parent, this.tooltip, {
        placement: "bottom"
      })
    }
  }

  /** Fired when the element is no longer being hovered over. */
  private baseWhenUnhovered() {
    if (this.disabled && !this.tooltip.classList.contains("is-hovered")) return

    if (this.whenUnhovered) this.whenUnhovered()

    this.tooltip.classList.remove("is-hovered")

    if (this.popperInstance) {
      // we'll only destroy the instance after
      // a timeout, to give room for a fade animation
      this.offTimer = timeout(100, () => {
        this.popperInstance!.destroy()
        this.popperInstance = undefined
      })
    }
  }

  connectedCallback() {
    super.connectedCallback()

    hover(this.parent, {
      on: () => {
        clearTimeout(this.offTimer)
        this.onTimer = timeout(50, () => this.baseWhenHovered())
      },
      off: () => {
        clearTimeout(this.onTimer)
        this.offTimer = timeout(50, () => this.baseWhenUnhovered())
      }
    })
  }
}
