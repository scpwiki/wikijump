/**
 * Standard OZONE shell for trusted, renderer-owned, read-only module
 * bodies.
 */
const dialogs = new WeakMap()

/** @param {Document} document @param {string} html @param {boolean} error */
export const showWikidotReadonlyDialog = (document, html, error = false) => {
  if (!document?.body) return
  dialogs.get(document)?.replace()
  const view = document.defaultView
  const iframe = document.createElement("iframe")
  iframe.id = "odialog-shader-iframe"
  iframe.src = "/common--misc/blank.html"
  iframe.setAttribute("frameborder", "0")
  iframe.className = "odialog-shader-iframe"
  const shader = document.createElement("div")
  shader.id = "odialog-shader"
  shader.className = "odialog-shader"
  let container = document.getElementById("odialog-container")
  if (!container) {
    container = document.createElement("div")
    container.id = "odialog-container"
    document.body.append(iframe, shader, container)
  } else document.body.append(iframe, shader)
  container.style.display = "block"
  const window = document.createElement("div")
  window.className = error ? "owindow error" : "owindow "
  const template = document.createElement("template")
  template.innerHTML = html
  window.appendChild(template.content)
  container.replaceChildren(window)
  // YUI setXY uses integer page coordinates, integer computed offsets, and
  // one retry. Theme !important rules remain authoritative over inline offsets.
  const left = Math.max(
    (document.documentElement.clientWidth - window.offsetWidth) / 2,
    0
  )
  const top = Math.max(
    (view?.scrollY ?? 0) +
      (document.documentElement.clientHeight - window.offsetHeight) / 2,
    0
  )
  const setXY = (x, y) => {
    for (let attempt = 0; attempt < 2; attempt += 1) {
      const rect = window.getBoundingClientRect()
      const style = view?.getComputedStyle(window)
      if (style?.position === "static") window.style.position = "relative"
      const cssLeft = Number.parseInt(style?.left ?? "", 10) || 0
      const cssTop = Number.parseInt(style?.top ?? "", 10) || 0
      window.style.left = `${x - Math.floor(rect.x + (view?.scrollX ?? 0)) + cssLeft}px`
      window.style.top = `${y - Math.floor(rect.y + (view?.scrollY ?? 0)) + cssTop}px`
    }
  }
  setXY(left, top)
  window.id = "owindow-1"
  const title = window.querySelector('[class="title"]')
  let dragging = null
  const move = (event) => {
    if (!dragging) return
    setXY(
      dragging.x + event.pageX - dragging.pointerX,
      dragging.y + event.pageY - dragging.pointerY
    )
  }
  const stopDrag = () => {
    dragging = null
  }
  if (title) {
    title.id = "ohandle-1"
    title.addEventListener("mousedown", (event) => {
      if (event.button !== 0) return
      const rect = window.getBoundingClientRect()
      dragging = {
        x: rect.x + (view?.scrollX ?? 0),
        y: rect.y + (view?.scrollY ?? 0),
        pointerX: event.pageX,
        pointerY: event.pageY
      }
      event.preventDefault()
    })
    view?.addEventListener("mousemove", move)
    view?.addEventListener("mouseup", stopDrag)
  }
  let timer
  let fadeTimer
  const hide = () => {
    view?.removeEventListener("mousemove", move)
    view?.removeEventListener("mouseup", stopDrag)
    if (timer) view?.clearTimeout(timer)
    if (fadeTimer) view?.clearInterval(fadeTimer)
    container.replaceChildren()
    container.style.display = "none"
    shader.remove()
    iframe.remove()
    dialogs.delete(document)
  }
  const close = () => {
    // ErrorDialog.close runs jQuery 1.11 fadeOut(100): one synchronous tick,
    // then 13 ms interval ticks with swing easing, hiding at completion.
    // cleanAll hides the container at 200 ms.
    if (error && view) {
      const start = Date.now()
      const tick = () => {
        const progress = 1 - Math.max(0, start + 100 - Date.now()) / 100
        if (progress < 1) {
          window.style.opacity = String(0.5 + Math.cos(progress * Math.PI) / 2)
          return true
        }
        window.style.display = "none"
        window.style.opacity = ""
        return false
      }
      if (tick()) {
        fadeTimer = view.setInterval(() => {
          if (!tick()) view.clearInterval(fadeTimer)
        }, 13)
      }
    }
    timer = view?.setTimeout(hide, 200)
  }
  // Capture before the source inline handler. No attribute or event-property
  // read compiles authored JavaScript, including in Firefox under CSP.
  for (const link of window.querySelectorAll(".button-bar a")) {
    link.addEventListener(
      "click",
      (event) => {
        event.preventDefault()
        event.stopImmediatePropagation()
        close()
      },
      true
    )
  }
  // OZONE does not close these windows on Escape or an outside click.
  if (error) window.querySelector(".button-bar a")?.focus()
  dialogs.set(document, { replace: hide })
}
