import { localizeWikidotThemeUrl } from "./wikidot-styleframe-contract.js"

const styleFrameSource =
  /<iframe\b[^>]*\bsrc=(['"])([^'"]*\/-\/wikidot-interwiki\/styleFrame\.html\?[^'"]*)\1[^>]*>/giu

/**
 * Extract the stylesheet contract already declared by rendered styleFrame
 * iframes. Rendering these links in the document head makes their CSS part
 * of the initial document load, as it is on Wikidot, instead of allowing
 * the local iframe to apply the theme only after DOMContentLoaded.
 *
 * @param {(string | null | undefined)[]} renderedHtml
 * @param {string | null | undefined} origin
 * @returns {{
 *   href: string
 *   priority: string
 *   priorityValue: number
 *   order: number
 * }[]}
 */
export const extractWikidotStyleFrameStylesheets = (renderedHtml, origin) => {
  const stylesheets = []
  let order = 0
  for (const html of renderedHtml) {
    if (!html) continue
    for (const match of html.matchAll(styleFrameSource)) {
      const source = match[2].replaceAll("&amp;", "&")
      let parsed
      try {
        parsed = new URL(source, origin || "https://wikijump.invalid")
      } catch {
        continue
      }
      if (parsed.pathname !== "/-/wikidot-interwiki/styleFrame.html") continue
      const priority = parsed.searchParams.get("priority") ?? ""
      const numericPriority = Number.parseFloat(priority)
      const priorityValue = Number.isFinite(numericPriority) ? numericPriority : 0
      for (const theme of parsed.searchParams.getAll("theme")) {
        if (!theme.trim()) continue
        stylesheets.push({
          href: localizeWikidotThemeUrl(theme, origin),
          priority,
          priorityValue,
          order: order++
        })
      }
    }
  }
  return stylesheets.sort(
    (left, right) => left.priorityValue - right.priorityValue || left.order - right.order
  )
}
