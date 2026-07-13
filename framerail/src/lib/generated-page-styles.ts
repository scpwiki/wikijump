const escapeStyleRawText = (css: string) => css.replaceAll("<", "\\3C ")

const cjkUnifiedIdeograph =
  /[\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff\u{20000}-\u{3134f}\u{2f800}-\u{2fa1f}]/u
const italicElement = /<(?:em|i)(?:\s|>)/iu
const monospaceElement = /<(?:code|kbd|pre|samp|tt|var)(?:\s|>)/iu

const publicSans = "/fonts/variable/PublicSans-VariableFont.woff2"
const publicSansItalic = "/fonts/variable/PublicSans-Italic-VariableFont.woff2"
const redHatDisplay = "/fonts/variable/RedHatDisplayVF.woff2"
const redHatDisplayItalic = "/fonts/variable/RedHatDisplayVF-Italic.woff2"
const cascadiaMono = "/fonts/variable/CascadiaMono.woff2"
const cascadiaMonoItalic = "/fonts/variable/CascadiaMonoItalic.woff2"

const cjkFontFileForLocale = (locale: string): string => {
  const normalizedLocale = locale.toLowerCase()

  if (normalizedLocale === "zh-hk" || normalizedLocale.startsWith("zh-hk-")) {
    return "NotoSansHK-VF.woff2"
  }

  if (
    normalizedLocale === "zh-tw" ||
    normalizedLocale.startsWith("zh-tw-") ||
    normalizedLocale === "zh-hant" ||
    normalizedLocale.startsWith("zh-hant-")
  ) {
    return "NotoSansTC-VF.woff2"
  }

  if (normalizedLocale === "ko" || normalizedLocale.startsWith("ko-")) {
    return "NotoSansKR-VF.woff2"
  }

  if (normalizedLocale === "zh" || normalizedLocale.startsWith("zh-")) {
    return "NotoSansSC-VF.woff2"
  }

  return "NotoSansJP-VF.woff2"
}

export function buildGeneratedPageStylesHead(styles: readonly string[]): string {
  return styles
    .map(
      (css, index) =>
        `<style type="text/css" data-wikijump-generated-css="${index}">${escapeStyleRawText(css)}</style>`
    )
    .join("")
}

export function getCjkFontPreloadHref(
  locale: string,
  renderedText: readonly (string | null | undefined)[]
): string | null {
  if (
    !renderedText.some(
      (text) => text !== null && text !== undefined && cjkUnifiedIdeograph.test(text)
    )
  ) {
    return null
  }

  return `/fonts/variable/${cjkFontFileForLocale(locale)}`
}

export function getPageFontPreloadHrefs(
  locale: string,
  renderedHtml: string | null | undefined,
  additionalRenderedText: readonly (string | null | undefined)[] = []
): string[] {
  const hrefs = [publicSans, redHatDisplay]
  const html = renderedHtml ?? ""
  const usesItalic = italicElement.test(html)
  const usesMonospace = monospaceElement.test(html)

  if (usesItalic) hrefs.push(publicSansItalic, redHatDisplayItalic)
  if (usesMonospace) hrefs.push(cascadiaMono)
  if (usesItalic && usesMonospace) hrefs.push(cascadiaMonoItalic)

  const cjkHref = getCjkFontPreloadHref(locale, [renderedHtml, ...additionalRenderedText])
  if (cjkHref !== null) hrefs.push(cjkHref)

  return hrefs
}
