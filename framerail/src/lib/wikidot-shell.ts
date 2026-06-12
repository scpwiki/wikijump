import { Layout } from "$lib/types"

interface ShellSiteData {
  from_wikidot?: boolean | null
  layout?: Layout | null
  top_bar_page?: string | null
  side_bar_page?: string | null
}

interface ShellPageData {
  from_wikidot?: boolean | null
  layout?: Layout | null
}

interface ShellRevisionData {
  from_wikidot?: boolean | null
}

interface ShellViewData {
  site?: ShellSiteData | null
  page?: ShellPageData | null
  page_revision?: ShellRevisionData | null
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

export function shouldUseWikidotShell(data: ShellViewData | null | undefined): boolean {
  const pageLayout = data?.page?.layout
  const siteLayout = data?.site?.layout

  if (pageLayout === Layout.WIKIDOT || siteLayout === Layout.WIKIDOT) {
    return true
  }

  if (pageLayout === Layout.WIKIJUMP) {
    return false
  }

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

export function resolveShellLayout(data: ShellViewData | null | undefined): Layout {
  return shouldUseWikidotShell(data)
    ? Layout.WIKIDOT
    : (data?.page?.layout ?? data?.site?.layout ?? Layout.WIKIJUMP)
}
