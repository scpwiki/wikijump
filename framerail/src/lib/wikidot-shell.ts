import { Layout } from "$lib/types"
import {
  resolveShellLayoutValue,
  shouldUseWikidotShellValue
} from "$lib/wikidot-shell-decision.js"

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

export function shouldUseWikidotShell(data: ShellViewData | null | undefined): boolean {
  return shouldUseWikidotShellValue(data)
}

export function resolveShellLayout(data: ShellViewData | null | undefined): Layout {
  return resolveShellLayoutValue(data) as Layout
}
