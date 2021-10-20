import * as Popper from "@popperjs/core"
import { clearTimeout, timeout, Timeout } from "@wikijump/util"
import { BaseButton } from "./base-button"
import { HoverObserver } from "./hover"

// TODO: proper mobile support (need more infrastructure for mobile support)

/**
 * Abstract custom element which extends the {@link BaseButton} element.
 * This element handles revealing a tooltip when it is hovered over. It
 * uses the `parent` and `tooltip` getters to do this. It will also call
 * the `whenHovered` and `whenUnhovered` methods, if they exist.
 */
export abstract class BaseTooltipButton extends BaseButton {
  /** The parent element. This is what the tooltip will be placed relative to. */
  abstract get parent(): HTMLElement

  /** The tooltip element. */
  abstract get tooltip(): HTMLElement

  /**
   * Fired when the element is hovered over. Can return false to cancel
   * revealing the tooltip.
   */
  protected whenHovered?(): void | boolean

  /** Fired when the element is unhovered. */
  protected whenUnhovered?(): void

  /** Timer to keep track of the delay for revealing the tooltip. */
  private declare onTimer?: Timeout

  /** Timer to keep track of the delay for hiding the tooltip. */
  private declare offTimer?: Timeout

  /** The Popper.js instance for handling placement of the tooltip. */
  private declare popperInstance?: Popper.Instance

  /** Internal observer for hover events. */
  private declare hoverObserver: HoverObserver

  /** Internal handler that is fired when the element is hovered over. */
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

  /** Internal event that is fired when the element is no longer being hovered over. */
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

    this.hoverObserver = new HoverObserver(this.parent, {
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

  disconnectedCallback() {
    this.hoverObserver.destroy()
    clearTimeout(this.onTimer)
    clearTimeout(this.offTimer)
  }
}
