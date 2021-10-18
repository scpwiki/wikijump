import { Gesture, Media, onSwipe } from "@wikijump/components"
import { addElement } from "@wikijump/util"

export class SidebarElement extends HTMLElement {
  static tag = "wj-sidebar"

  private open = false

  constructor() {
    super()

    onSwipe(document.documentElement, {
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
      const max = this.open ? 0 : this.offsetWidth
      const start = !this.open ? "-100% + " : ""
      const offset = Math.min(-diff[1], max)
      this.style.transition = "none"
      this.style.transform = `translateX(calc(${start}${offset}px))`
    }
    // reset style back to normal
    else if (type === "cancel" || type === "end") {
      this.style.transition = ""
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
