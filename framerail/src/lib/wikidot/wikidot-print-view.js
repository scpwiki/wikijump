const escapeHtml = (value) =>
  value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;")

export const WIKIDOT_PRINTER_FRIENDLY_PREFIX = "/printer--friendly/"

/**
 * Build the live Wikidot printer-friendly child URL. The page path already
 * begins with `/`, which preserves Wikidot's exact doubled slash
 * (`/printer--friendly//<page>`).
 *
 * @param {string} pagePath
 * @returns {string}
 */
export const buildWikidotPrinterFriendlyUrl = (pagePath) =>
  `${WIKIDOT_PRINTER_FRIENDLY_PREFIX}${pagePath}`

/** @type {readonly string[]} */
export const WIKIDOT_PRINT_FONT_SIZES = Object.freeze([
  "6pt",
  "8pt",
  "10pt",
  "12pt",
  "14pt",
  "16pt"
])

const fontSizesHtml = WIKIDOT_PRINT_FONT_SIZES.map(
  (size) =>
    `<a href="javascript:;" onclick="WIKIDOT.printview.listeners.changeFontSize(event, '${size}')">${size}</a>`
).join(" | ")

const WIKIDOT_PRINT_FONT_FAMILIES = Object.freeze([
  { label: "original font", value: "original", css: null },
  { label: "Georgia", value: "georgia", css: "Georgia" },
  { label: "Times New Roman", value: "times-new-roman", css: '"Times New Roman"' },
  { label: "Serif (generic)", value: "serif", css: "serif" },
  {
    label: "Arial/Helvetica",
    value: "arial-helvetica",
    css: "Arial, Helvetica, sans-serif"
  }
])

const fontFamilyByValue = new Map(
  WIKIDOT_PRINT_FONT_FAMILIES.map((choice) => [choice.value, choice])
)

const fontFamiliesHtml = WIKIDOT_PRINT_FONT_FAMILIES.map(
  ({ label, value }, index) =>
    `<a href="javascript:;" role="button" aria-pressed="${index === 0}" onclick="WIKIDOT.printview.listeners.changeFontFamily(event, '${value}')">${label}</a>`
).join(" | ")

/** Wikidot's verified printer-friendly option rows. */
export const buildWikidotPrintOptionsHtml = () =>
  [
    '<div id="print-options">',
    "<table>",
    "<tbody>",
    "<tr>",
    "<td>",
    "Base font size:",
    "</td>",
    `<td>${fontSizesHtml}</td>`,
    "</tr>",
    "<tr>",
    "<td>",
    "Body font:",
    "</td>",
    `<td>${fontFamiliesHtml}</td>`,
    "</tr>",
    "<tr>",
    "<td>",
    "Source info:",
    "</td>",
    '<td><a href="javascript:;" role="button" aria-pressed="false" onclick="WIKIDOT.printview.listeners.toggleSourceInfo(event)">toggle visibility</a></td>',
    "</tr>",
    "<tr>",
    "<td>",
    "Options:",
    "</td>",
    '<td><b><a href="javascript:;" onclick="window.print()">PRINT THE PAGE</a></b> | <a href="javascript:;">Close this window</a></td>',
    "</tr>",
    "</tbody>",
    "</table>",
    "</div>"
  ].join("\n")

/**
 * @param {{
 *   siteName: string
 *   siteUrl: string
 *   pageTitle: string
 *   pageUrl: string
 * }} input
 * @returns {string}
 */
export const buildWikidotPrintSourceInfoHtml = ({
  siteName,
  siteUrl,
  pageTitle,
  pageUrl
}) =>
  [
    '<div id="print-source-info">',
    `Site: <b>${escapeHtml(siteName)}</b> at ${escapeHtml(siteUrl)}`,
    `Source page: <b>${escapeHtml(pageTitle)}</b> at ${escapeHtml(pageUrl)}`,
    "</div>"
  ].join("\n")

const CHANGE_FONT_SIZE_ONCLICK =
  /^WIKIDOT\.printview\.listeners\.changeFontSize\(event, '([0-9]+pt)'\)$/u
const CHANGE_FONT_FAMILY_ONCLICK =
  /^WIKIDOT\.printview\.listeners\.changeFontFamily\(event, '([^']+)'\)$/u
const TOGGLE_SOURCE_INFO_ONCLICK = "WIKIDOT.printview.listeners.toggleSourceInfo(event)"

const originalFontStyles = new WeakMap()

/**
 * Bind the printer-friendly child window's trusted behavior. Served markup
 * keeps Wikidot's exact inert `javascript:;` anchors and generated handler
 * attributes; this action supplies the executable property under CSP
 * without evaluating authored script. Handling is delegated on the
 * printer-friendly root so `{@html}` child insertion or hydration timing
 * cannot leave a generated control unbound; only the exact generated
 * handler shapes activate.
 *
 * @param {HTMLElement} root
 */
export const wikidotPrintView = (root) => {
  const content = () => root.querySelector("#print-content")
  const sourceInfo = () => root.querySelector("#print-source-info")
  if (!originalFontStyles.has(root)) {
    const targetContent = content()
    originalFontStyles.set(root, {
      value: targetContent?.style.getPropertyValue("font-family") ?? "",
      priority: targetContent?.style.getPropertyPriority("font-family") ?? ""
    })
  }
  const listener = (event) => {
    const target = typeof event.target?.closest === "function" ? event.target : null
    if (!target) return
    const print = target.closest('a[onclick="window.print()"]')
    if (print) {
      event.preventDefault()
      globalThis.print()
      return
    }
    const change = target.closest(
      'a[onclick^="WIKIDOT.printview.listeners.changeFontSize"]'
    )
    if (change) {
      const match = CHANGE_FONT_SIZE_ONCLICK.exec(change.getAttribute("onclick") ?? "")
      if (match) {
        event.preventDefault()
        const targetContent = content()
        if (targetContent) targetContent.style.fontSize = match[1]
      }
      return
    }
    const fontFamily = target.closest(
      'a[onclick^="WIKIDOT.printview.listeners.changeFontFamily"]'
    )
    if (fontFamily) {
      const match = CHANGE_FONT_FAMILY_ONCLICK.exec(
        fontFamily.getAttribute("onclick") ?? ""
      )
      const choice = match ? fontFamilyByValue.get(match[1]) : null
      if (choice) {
        event.preventDefault()
        const targetContent = content()
        if (!targetContent) return
        if (choice.css === null) {
          const original = originalFontStyles.get(root)
          if (original?.value) {
            targetContent.style.setProperty(
              "font-family",
              original.value,
              original.priority
            )
          } else {
            targetContent.style.removeProperty("font-family")
          }
        } else {
          targetContent.style.setProperty("font-family", choice.css)
        }
        for (const control of root.querySelectorAll(
          'a[onclick^="WIKIDOT.printview.listeners.changeFontFamily"]'
        )) {
          const controlMatch = CHANGE_FONT_FAMILY_ONCLICK.exec(
            control.getAttribute("onclick") ?? ""
          )
          control.setAttribute("aria-pressed", String(controlMatch?.[1] === choice.value))
        }
      }
      return
    }
    const toggleSourceInfo = target.closest(`a[onclick="${TOGGLE_SOURCE_INFO_ONCLICK}"]`)
    if (toggleSourceInfo) {
      event.preventDefault()
      const info = sourceInfo()
      if (info) {
        info.hidden = !info.hidden
        toggleSourceInfo.setAttribute("aria-pressed", String(info.hidden))
      }
      return
    }
    const close = target.closest('#print-options a[href="javascript:;"]:not([onclick])')
    if (close) {
      event.preventDefault()
      globalThis.close()
    }
  }
  root.addEventListener("click", listener)
  return {
    destroy() {
      root.removeEventListener("click", listener)
    }
  }
}
