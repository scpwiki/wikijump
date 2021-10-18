import { Gesture, Media, onSwipe } from "@wikijump/components"
import { addElement } from "@wikijump/util"

export class SidebarElement extends HTMLElement {
  static tag = "wj-sidebar"

  private open = false

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
      callback: (_node, gesture) => this.swipe(gesture),
      eventCallback: (_node, gesture) => this.move(gesture)
    })
  }

  private swipe({ direction }: Gesture) {
    if (direction === "right") {
      this.open = true
      this.classList.add("is-open")
    } else {
      this.open = false
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

declare global {
  interface HTMLElementTagNameMap {
    "wj-sidebar": SidebarElement
  }

  interface Window {
    SidebarElement: typeof SidebarElement
  }
}

addElement(SidebarElement, "SidebarElement")
