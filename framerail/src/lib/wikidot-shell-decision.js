export const WIKIDOT_LAYOUT = "wikidot"
export const WIKIJUMP_LAYOUT = "wikijump"

const hasText = (value) => Boolean(value?.trim())

const hasStandardWikidotShellPages = (data) => {
  return (
    data?.site?.top_bar_page === "nav:top" && data?.site?.side_bar_page === "nav:side"
  )
}

export const shouldUseWikidotShellValue = (data) => {
  const pageLayout = data?.page?.layout
  const siteLayout = data?.site?.layout

  if (pageLayout === WIKIJUMP_LAYOUT) return false
  if (pageLayout === WIKIDOT_LAYOUT) return true
  if (siteLayout === WIKIJUMP_LAYOUT) return false
  if (siteLayout === WIKIDOT_LAYOUT) return true

  const hasCompiledShellHtml =
    hasText(data?.compiled_top_bar_html) || hasText(data?.compiled_side_bar_html)
  const hasConfiguredShellPages =
    hasText(data?.site?.top_bar_page) || hasText(data?.site?.side_bar_page)
  const isWikidotImported =
    data?.site?.from_wikidot ||
    data?.page?.from_wikidot ||
    data?.page_revision?.from_wikidot

  return (
    hasCompiledShellHtml ||
    hasStandardWikidotShellPages(data) ||
    (Boolean(isWikidotImported) && hasConfiguredShellPages)
  )
}

export const resolveShellLayoutValue = (data) => {
  return shouldUseWikidotShellValue(data)
    ? WIKIDOT_LAYOUT
    : (data?.page?.layout ?? data?.site?.layout ?? WIKIJUMP_LAYOUT)
}
