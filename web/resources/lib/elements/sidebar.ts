/* eslint-disable @typescript-eslint/unbound-method */
import { Gesture, Media, SwipeObserver, SwipeOpts, tip } from "@wikijump/components"
import { addElement, BaseButton } from "@wikijump/util"

export class SidebarElement extends HTMLElement {
  static tag = "wj-sidebar"

  private declare mediaDestroy: () => void
  private declare observer?: SwipeObserver
  private declare app: HTMLElement

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

    this.bodyClick = this.bodyClick.bind(this)

    this.mediaDestroy = Media.subscribe(({ breakpoint }) => {
      // create observer if viewport is small
      if (breakpoint === "small" || breakpoint === "narrow") {
        if (!this.observer) {
          this.observer = new SwipeObserver(this.app, this.config)
          this.app.addEventListener("click", this.bodyClick)
        }
      } else {
        if (this.observer) {
          this.observer.destroy()
          this.observer = undefined
          this.app.removeEventListener("click", this.bodyClick)
        }

        if (this.open) this.close()
      }
    })
  }

  get open() {
    return this.classList.contains("is-open")
  }

  show() {
    this.classList.add("is-open")
  }

  close() {
    this.classList.remove("is-open")
  }

  private bodyClick(evt: MouseEvent) {
    if (!this.open) return
    // special case for open button
    if (evt.target instanceof SidebarButtonElement) return
    // body elements
    if (evt.target !== this && !this.contains(evt.target as HTMLElement)) {
      this.classList.remove("is-open")
    }
  }

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
    }
    // reset style back to normal
    else if (gst.is("cancel", "end")) {
      this.style.transition = ""
      this.style.boxShadow = ""
      this.style.transform = ""
    }
  }

  disconnectedCallback() {
    this.observer?.destroy()
    this.mediaDestroy()
    this.app.removeEventListener("click", this.bodyClick)
  }
}

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
