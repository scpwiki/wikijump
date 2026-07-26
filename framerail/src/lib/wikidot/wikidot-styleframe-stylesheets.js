import {
  escapeStyleFrameHtml,
  isUsableStyleFrameCss,
  localizeWikidotThemeUrl,
  safeInlineStyleFrameCss
} from "./wikidot-styleframe-contract.js"

const styleFrameSource =
  /<iframe\b[^>]*\bsrc=(['"])([^'"]*\/-\/wikidot-interwiki\/styleFrame\.html\?[^'"]*)\1[^>]*>/giu

/**
 * Extract the complete CSS contract declared by rendered styleFrame
 * iframes. Rendering these declarations in the document head makes their
 * CSS part of the initial document instead of waiting for the local iframe
 * runtime.
 *
 * @param {(string | null | undefined)[]} renderedHtml
 * @param {string | null | undefined} origin
 * @returns {(
 *   | {
 *       kind: "theme"
 *       href: string
 *       priority: string
 *       priorityValue: number
 *       order: number
 *     }
 *   | {
 *       kind: "inline"
 *       css: string
 *       priority: string
 *       priorityValue: number
 *       order: number
 *     }
 * )[]}
 */
export const extractWikidotStyleFrameDeclarations = (renderedHtml, origin) => {
  const declarations = []
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
        declarations.push({
          kind: "theme",
          href: localizeWikidotThemeUrl(theme, origin),
          priority,
          priorityValue,
          order: order++
        })
      }
      const css = parsed.searchParams.get("css")
      if (isUsableStyleFrameCss(css)) {
        declarations.push({
          kind: "inline",
          css: safeInlineStyleFrameCss(css),
          priority,
          priorityValue,
          order: order++
        })
      }
    }
  }
  return declarations.sort(
    (left, right) => left.priorityValue - right.priorityValue || left.order - right.order
  )
}

export const extractWikidotStyleFrameStylesheets = (renderedHtml, origin) =>
  extractWikidotStyleFrameDeclarations(renderedHtml, origin)
    .filter((declaration) => declaration.kind === "theme")
    .map((declaration) => ({
      href: declaration.href,
      priority: declaration.priority,
      priorityValue: declaration.priorityValue,
      order: declaration.order
    }))

/** @param {{ css: string; priority: string }} declaration */
export const buildWikidotInlineStyleFrameHead = ({ css, priority }) =>
  `<style data-wikidot-style-preloaded data-wikidot-style-priority="${escapeStyleFrameHtml(priority)}" type="text/css">${css}</style>`
