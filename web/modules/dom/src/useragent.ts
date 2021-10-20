/**
 * Browser / User-Agent info. Contains contextual information like
 * normalized mouse position values.
 */
export namespace UserAgent {
  /** Horizontal mouse position, normalized between 0-1. */
  export let mouseX = 0
  /** Vertical mouse position, normalized between 0-1. */
  export let mouseY = 0
  /** Scroll position, normalized between 0-1. */
  export let scroll = 0
  /**
   * Flag that is true if the agent is using a mobile device. Probably,
   * anyways. It's difficult to rely on this and it should only be used
   * with caution.
   */
  export const isMobile = /Mobi|Android/i.test(navigator.userAgent)

  // set up our listeners

  window.addEventListener("mousemove", evt => {
    mouseX = evt.clientX / window.innerWidth
    mouseY = evt.clientY / window.innerHeight
  })

  window.addEventListener("scroll", () => {
    scroll =
      document.documentElement.scrollTop /
      (document.body.scrollHeight - window.innerHeight)
  })
}
