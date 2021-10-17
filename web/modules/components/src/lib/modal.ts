import { SvelteComponent } from "svelte"
import Dialog from "../Dialog.svelte"

export class Modal<T extends typeof SvelteComponent> {
  private dialog: Dialog
  private _open: boolean

  readonly component: T

  constructor(component: T, open = false, lazy = true) {
    this.component = component
    this._open = open

    const modals = document.getElementById("modals")
    if (!modals) throw new Error("Modals container not found")

    this.dialog = new Dialog({ target: modals, props: { component, open, lazy } })

    this.dialog.$on("change", (evt: Event & { detail: boolean }) => {
      const state = evt.detail
      this._open = state
      if (this.onChange) this.onChange(state)
      if (state && this.onOpen) this.onOpen()
      if (!state && this.onClose) this.onClose()
    })

    this.dialog.$on("cancel", (evt: Event) => {
      if (this.onCancel) this.onCancel(evt)
    })
  }

  get open() {
    return this._open
  }

  set open(state: boolean) {
    if (this._open === state) return
    this.dialog.$set({ open: state })
    this._open = state
  }

  declare onChange?: (open: boolean) => void

  declare onOpen?: () => void

  declare onClose?: () => void

  declare onCancel?: (evt: Event) => void

  destroy() {
    this.dialog.$destroy()
  }
}
