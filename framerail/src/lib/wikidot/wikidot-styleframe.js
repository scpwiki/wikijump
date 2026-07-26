import {
  escapeStyleFrameHtml,
  isUsableStyleFrameCss,
  localizeWikidotThemeUrl,
  safeInlineStyleFrameCss
} from "./wikidot-styleframe-contract.js"
import { buildWikidotStyleFrameRuntime } from "./wikidot-styleframe-runtime.js"
import {
  buildWikidotInlineStyleFrameHead,
  extractWikidotStyleFrameDeclarations,
  extractWikidotStyleFrameStylesheets
} from "./wikidot-styleframe-stylesheets.js"

export {
  isUsableStyleFrameCss,
  localizeWikidotThemeUrl,
  buildWikidotInlineStyleFrameHead,
  extractWikidotStyleFrameDeclarations,
  extractWikidotStyleFrameStylesheets
}

/**
 * @typedef {{
 *   priority?: string | null
 *   themes?: string[]
 *   css?: string | null
 *   origin?: string | null
 * }} WikidotStyleFrameInput
 */

/**
 * @param {WikidotStyleFrameInput} input
 * @returns {string}
 */
export const buildWikidotStyleFrameHtml = ({
  priority = "",
  themes = [],
  css = "",
  origin = null
}) => {
  const localizedThemes = themes
    .filter((theme) => theme.trim().length > 0)
    .map((theme) => localizeWikidotThemeUrl(theme, origin))
  const inlineCss = isUsableStyleFrameCss(css) ? safeInlineStyleFrameCss(css ?? "") : ""
  const themeList = localizedThemes
    .map((theme) => `<li>${escapeStyleFrameHtml(theme)}</li>`)
    .join("")
  const styleBlock = inlineCss ? `<style>${inlineCss}</style>` : ""
  const script = buildWikidotStyleFrameRuntime({
    priority: priority ?? "",
    themes: localizedThemes,
    css: inlineCss
  })

  return `<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Local Wikidot style frame</title>
    <meta name="wikidot-style-priority" content="${escapeStyleFrameHtml(priority)}">
    <meta name="wikidot-style-theme-count" content="${localizedThemes.length}">
    <meta name="wikidot-style-inline-css" content="${inlineCss ? "true" : "false"}">
    ${styleBlock}
  </head>
  <body>
    <ul hidden>${themeList}</ul>
    <script>${script}</script>
  </body>
</html>
`
}
