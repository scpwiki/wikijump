export const WIKIDOT_LAYOUT = "wikidot"
export const WIKIJUMP_LAYOUT = "wikijump"

export type ShellLayout = typeof WIKIDOT_LAYOUT | typeof WIKIJUMP_LAYOUT

export interface ShellViewData {
  site?: {
    from_wikidot?: boolean | null
    layout?: ShellLayout | null
    top_bar_page?: string | null
    side_bar_page?: string | null
  } | null
  page?: {
    from_wikidot?: boolean | null
    layout?: ShellLayout | null
  } | null
  page_revision?: { from_wikidot?: boolean | null } | null
  compiled_top_bar_html?: string | null
  compiled_side_bar_html?: string | null
}

function hasText(value: string | null | undefined): boolean {
  return !!value?.trim()
}

function hasStandardWikidotShellPages(data: ShellViewData | null | undefined): boolean {
  return (
    data?.site?.top_bar_page === "nav:top" && data?.site?.side_bar_page === "nav:side"
  )
}

export function shouldUseWikidotShellValue(
  data: ShellViewData | null | undefined
): boolean {
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
    (!!isWikidotImported && hasConfiguredShellPages)
  )
}

export function resolveShellLayoutValue(
  data: ShellViewData | null | undefined
): ShellLayout {
  return shouldUseWikidotShellValue(data)
    ? WIKIDOT_LAYOUT
    : (data?.page?.layout ?? data?.site?.layout ?? WIKIJUMP_LAYOUT)
}
