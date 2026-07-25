import { Layout } from "$lib/types"

class ErrorPopupState {
  current = $state<{ state: boolean; message: string | null; data: unknown | null }>({
    state: false,
    message: null,
    data: null
  })
}

class PageLayoutState {
  current = $state<Layout>(Layout.WIKIJUMP)
}

export const errorPopupState = new ErrorPopupState()
export const pageLayoutState = new PageLayoutState()
