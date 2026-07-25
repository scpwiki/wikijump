import { getContext } from "svelte"

import { Layout } from "$lib/types"

export const PAGE_LAYOUT_CONTEXT_KEY = Symbol("page-layout-context")

export interface PageLayoutContext {
  current: Layout
}

const DEFAULT_PAGE_LAYOUT_CONTEXT: PageLayoutContext = { current: Layout.WIKIJUMP }

export function getPageLayoutContext(): PageLayoutContext {
  return (
    getContext<PageLayoutContext | undefined>(PAGE_LAYOUT_CONTEXT_KEY) ??
    DEFAULT_PAGE_LAYOUT_CONTEXT
  )
}
