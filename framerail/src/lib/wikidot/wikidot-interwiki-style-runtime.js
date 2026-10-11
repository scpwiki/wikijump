/**
 * Style half of the Wikidot interwiki frame runtime.
 *
 * This is a source-equivalent port of the `requestStyleChange` contract in
 * https://interwiki.scpwiki.com/interwiki.js (retrieved 2026-10-11): each
 * `styleFrame.html` query declares a `priority`, an optional `theme`
 * stylesheet, and optional inline `css`. The interwiki frame merges every
 * declaration into its own document head in ascending priority order, a
 * theme before inline CSS of the same priority, ignoring exact duplicates.
 * Style frames push their query when the interwiki frame is already
 * initialized; the interwiki frame pulls from style frames that loaded
 * first. Nothing is ever applied to the parent frameDocument.
 *
 * The function is serialized into the frame document with
 * `Function.prototype.toString`, so it must stay self-contained: no
 * imports and no references to module scope. `localizeThemeUrl` is passed
 * in the same way.
 *
 * @param {Window & {
 *   requestStyleChange?: (search: string) => void
 *   isInterwikiFrame?: boolean
 * }} frameWindow
 * @param {{ siteUrl: string }} config
 * @param {(
 *   themeUrl: string,
 *   origin: string | null | undefined
 * ) => string} localizeThemeUrl
 */
export const installWikidotInterwikiStyleRuntime = (
  frameWindow,
  config,
  localizeThemeUrl
) => {
  const frameDocument = frameWindow.document

  // Wikidot reads the last occurrence of a parameter and decodes it with
  // decodeURIComponent only ("+" stays literal).
  const queryParameter = (search, name) => {
    const query = search.indexOf("?") === 0 ? search.substring(1) : search
    const parts = query.split("&").reverse()
    const part = parts.find((candidate) => candidate.indexOf(`${name}=`) === 0)
    return part === undefined ? "" : decodeURIComponent(part.substring(name.length + 1))
  }

  const customStyles = (selector) =>
    Array.prototype.slice.call(frameDocument.head.querySelectorAll(selector))

  const sameDeclaration = (priority, value) => (element) => {
    if (Number(element.getAttribute("data-priority")) !== priority) return false
    if (element.tagName === "LINK") return element.getAttribute("href") === value
    if (element.tagName === "STYLE") return element.textContent === value
    return false
  }

  const insertByPriority = (priority, element) => {
    element.classList.add("custom-style")
    element.dataset.priority = String(priority)
    const tagName = element.tagName
    const inserted = customStyles("link.custom-style, style.custom-style").some(
      (existing) => {
        const existingPriority = Number(existing.dataset.priority)
        if (priority > existingPriority) return false
        if (
          existingPriority === priority &&
          existing.tagName === "LINK" &&
          tagName === "STYLE"
        ) {
          return false
        }
        frameDocument.head.insertBefore(element, existing)
        return true
      }
    )
    if (!inserted) frameDocument.head.appendChild(element)
  }

  const addTheme = (priority, href) => {
    if (customStyles("link.custom-style").some(sameDeclaration(priority, href))) return
    const link = frameDocument.createElement("link")
    link.rel = "stylesheet"
    link.href = href
    insertByPriority(priority, link)
  }

  const addCss = (priority, css) => {
    if (customStyles("style.custom-style").some(sameDeclaration(priority, css))) return
    const style = frameDocument.createElement("style")
    style.textContent = css
    insertByPriority(priority, style)
  }

  // Wikidot resolves a bare code-page name to its first code block, and
  // `page/code/N` to block N of that page, on the community's own site.
  const resolveTheme = (theme) => {
    if (theme.indexOf("http") === 0 || theme.indexOf("//") === 0) return theme
    if (!config.siteUrl) return ""
    if (theme.indexOf("/") === -1) return `${config.siteUrl}/local--code/${theme}/1`
    const parts = theme.split("/")
    return parts.length >= 3 && parts[1] === "code"
      ? `${config.siteUrl}/local--code/${parts[0]}/${parts[2]}`
      : ""
  }

  frameWindow.requestStyleChange = (search) => {
    const rawPriority = queryParameter(search, "priority")
    const priority = Number(rawPriority)
    if (Number.isNaN(priority)) return
    const theme = queryParameter(search, "theme")
    if (theme) {
      const href = resolveTheme(theme)
      if (href) addTheme(priority, localizeThemeUrl(href, frameWindow.location.origin))
    }
    const css = queryParameter(search, "css")
    if (css) addCss(priority, css)
  }

  const pullStyleFrames = () => {
    Array.prototype.slice.call(frameWindow.parent).forEach((frame) => {
      try {
        if (frame.isStyleFrame) frameWindow.requestStyleChange(frame.location.search)
      } catch (error) {
        if (!(error instanceof frameWindow.DOMException)) throw error
      }
    })
  }

  const initialize = () => {
    pullStyleFrames()
    frameWindow.isInterwikiFrame = true
  }
  if (frameDocument.readyState === "loading") {
    frameWindow.addEventListener("DOMContentLoaded", initialize)
  } else {
    initialize()
  }
}
