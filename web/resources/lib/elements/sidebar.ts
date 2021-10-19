import { Gesture, Media, onSwipe, tip } from "@wikijump/components"
import { addElement, BaseButton } from "@wikijump/util"

export class SidebarElement extends HTMLElement {
  static tag = "wj-sidebar"

  constructor() {
    super()

    const app = document.querySelector("#app") as HTMLElement
    if (!app) throw new Error("No app element found")

    onSwipe(app, {
      condition: () => Media.matchBreakpoint("<=small"),
      direction: ["left", "right"],
      threshold: 70,
      minThreshold: 25,
      immediate: false,
      timeout: false,
      callback: (_node, { direction }) => {
        if (direction === "right") this.show()
        else this.close()
      },
      eventCallback: (_node, gesture) => this.move(gesture)
    })

    app.addEventListener("click", evt => this.bodyClick(evt))
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

  private move({ type, diff }: Gesture) {
    if (type === "move") {
      const offset = Math.min(-diff[1], this.open ? 0 : this.offsetWidth)
      const ratio = this.open
        ? Math.max(1 - Math.abs(offset) / this.offsetWidth, 0)
        : Math.max(offset / this.offsetWidth, 0)

      const start = !this.open ? "-100% + " : ""

      this.style.transition = "none"
      this.style.boxShadow = `${-5 + ratio * 15}rem 0 10rem rgba(0, 0, 0, 0.25)`
      this.style.transform = `translateX(calc(${start}${offset}px))`
    }
    // reset style back to normal
    else if (type === "cancel" || type === "end") {
      this.style.transition = ""
      this.style.boxShadow = ""
      this.style.transform = ""
    }
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
