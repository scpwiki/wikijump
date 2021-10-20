/* eslint-disable @typescript-eslint/unbound-method */
import { Media, tip } from "@wikijump/components"
import { addElement, BaseButton, Gesture, SwipeObserver, SwipeOpts } from "@wikijump/dom"

/** Slidable sidebar element. */
export class SidebarElement extends HTMLElement {
  static tag = "wj-sidebar"

  static get observedAttributes() {
    return ["open"]
  }

  /** Function that destroys the {@link Media} subscription. */
  private declare mediaDestroy: () => void

  /** The {@link SwipeObserver} used to recognize swipes. */
  private declare observer?: SwipeObserver

  /** The `#app` element. */
  private declare app: HTMLElement

  /** The `#sidebar_sticky` child element. */
  private declare sticky: HTMLElement

  /** The previous focus before the sidebar was opened. */
  private previousFocus: HTMLElement | null = null

  /** {@link SwipeObserver} configuration. */
  private config: SwipeOpts = {
    direction: ["left", "right"],
    threshold: 70,
    minThreshold: 25,
    immediate: false,
    timeout: false,
    callback: (_node, gst) => {
      if (gst.direction === "right") this.show()
      else this.close()
    },
    eventCallback: (_node, gesture) => this.move(gesture)
  }

  constructor() {
    super()

    const app = document.querySelector("#app") as HTMLElement
    if (!app) throw new Error("No app element found")
    this.app = app

    const sticky = document.querySelector("#sidebar_sticky") as HTMLElement
    if (!sticky) throw new Error("No sticky element found")
    this.sticky = sticky

    this.bodyClick = this.bodyClick.bind(this)

    this.mediaDestroy = Media.subscribe(({ breakpoint }) => {
      if (breakpoint === "small" || breakpoint === "narrow") {
        this.startListening()
      } else {
        this.stopListening()
      }
    })
  }

  /** True if the sidebar is open. */
  get open() {
    return this.hasAttribute("open")
  }

  /** Reveals the sidebar. */
  show() {
    this.setAttribute("open", "")
  }

  /** Closes the sidebar. */
  close() {
    this.removeAttribute("open")
  }

  /**
   * Begins listening for swipes. This is called when the screen size is
   * below a certain threshold.
   */
  private startListening() {
    if (!this.observer) {
      this.observer = new SwipeObserver(this.app, this.config)
      this.app.addEventListener("click", this.bodyClick)
    }

    this.setAttribute("aria-expanded", "false")
  }

  /**
   * Stops listening for swipes. This is called when the screen size is
   * beyond a certain threshold.
   */
  private stopListening() {
    if (this.observer) {
      this.observer.destroy()
      this.observer = undefined
      this.app.removeEventListener("click", this.bodyClick)
    }

    if (this.open) this.close()

    this.removeAttribute("aria-expanded")
  }

  /** Event handler for closing the sidebar when the body is tapped/clicked. */
  private bodyClick(evt: MouseEvent) {
    if (!this.open) return
    // special case for open button
    if (evt.target instanceof SidebarButtonElement) return
    // body elements
    if (evt.target !== this && !this.contains(evt.target as HTMLElement)) {
      this.close()
    }
  }

  /** Handles how the sidebar can be "grabbed" as the user moves their touch. */
  private move(gst: Gesture) {
    if (gst.is("move")) {
      const open = this.open

      const offset = gst.offset({
        min: open ? -this.offsetWidth : 0,
        max: open ? 0 : this.offsetWidth
      })

      let ratio = offset / this.offsetWidth
      if (open) ratio = 1 + ratio

      const start = !open ? "-100% + " : ""

      this.style.transition = "none"
      this.style.boxShadow = `${-5 + ratio * 15}rem 0 10rem rgba(0, 0, 0, 0.25)`
      this.style.transform = `translateX(calc(${start}${offset}px))`

      this.sticky.style.visibility = "visible"
    }
    // reset style back to normal
    else if (gst.is("cancel", "end")) {
      this.style.transition = ""
      this.style.boxShadow = ""
      this.style.transform = ""
      this.sticky.style.visibility = ""
    }
  }

  // -- LIFECYCLE

  attributeChangedCallback(name: string) {
    if (name === "open") {
      const open = this.hasAttribute("open")
      if (open) {
        this.setAttribute("aria-expanded", "true")
        this.previousFocus = document.activeElement as HTMLElement | null
      } else {
        this.setAttribute("aria-expanded", "false")
        if (this.previousFocus) {
          this.previousFocus.focus()
          this.previousFocus = null
        }
      }
    }
  }

  disconnectedCallback() {
    this.stopListening()
    this.mediaDestroy()
  }
}

/** Simple button that opens and closes the sidebar. */
export class SidebarButtonElement extends BaseButton {
  static tag = "wj-sidebar-button"

  constructor() {
    super()
    // enables tooltip using aria-label
    tip(this)
  }

  whenClicked() {
    const sidebar = document.querySelector("#sidebar") as SidebarElement
    if (!sidebar) throw new Error("No sidebar element found")
    if (!sidebar.open) sidebar.show()
  }
}

declare global {
  interface HTMLElementTagNameMap {
    "wj-sidebar": SidebarElement
    "wj-sidebar-button": SidebarButtonElement
  }

  interface Window {
    SidebarElement: typeof SidebarElement
    SidebarButtonElement: typeof SidebarButtonElement
  }
}

addElement(SidebarElement, "SidebarElement")
addElement(SidebarButtonElement, "SidebarButtonElement")
