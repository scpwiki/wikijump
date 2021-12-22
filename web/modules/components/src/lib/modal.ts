import type { SvelteComponent } from "svelte"
import type DialogType from "../Dialog.svelte"

/**
 * Wraps around the {@link Dialog} component to provide a simple interface
 * for adding a modal that can be open or closed programatically.
 */
export class Modal<
  C extends typeof SvelteComponent,
  T extends C | Promise<{ default: C }>
> {
  /** The internal {@link Dialog} component. */
  private dialog?: DialogType

  /** Promise that resolves when the Dialog component loads. */
  private loading: Promise<void>

  /** @see {@link open} */
  private _open: boolean

  /** @see {@link detail} */
  private _detail: Record<string, any> = {}

  /** The component being slotted into the dialog. */
  declare component?: C

  /** Callback fired when the open state of the modal changes. */
  declare onChange?: (open: boolean) => void

  /** Callback fired when the modal opens. */
  declare onOpen?: () => void

  /** Callback fired when the modal closes. */
  declare onClose?: () => void

  /**
   * Callback fired when the modal is "cancelled" by pressing escape. This
   * callback is provided with the actual `cancel` event, so the
   * cancellation itself can be cancelled by calling the `preventDefault()`
   * method on the event.
   *
   * @param evt - The cancel event.
   */
  declare onCancel?: (evt: Event) => void

  /**
   * @param component - The component to be slotted into the dialog.
   * @param open - Whether the dialog should be open initially.
   * @param lazy - If true, slotted content will only be inserted into the
   *   DOM when the dialog is open.
   */
  constructor(component: T, open = false, lazy = true) {
    this._open = open

    const modals = document.getElementById("modals")
    if (!modals) throw new Error("Modals container not found")

    this.loading = this.loadDialog(component, open, lazy)
  }

  /** Imports and constructs the dialog. */
  private async loadDialog(component: T, open: boolean, lazy: boolean) {
    component = component instanceof Promise ? (await component).default : component
    this.component = component as C

    const Dialog = (await import("../Dialog.svelte")).default

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

  /** Helper for firing a callback once the Dialog component has loaded. */
  private async whenLoaded(cb: (dialog: DialogType) => void) {
    await this.loading
    cb(this.dialog!)
  }

  /** The open state of the modal. */
  get open() {
    return this._open
  }

  /** The open state of the modal. */
  set open(state: boolean) {
    if (this._open === state) return
    this._open = state
    this.whenLoaded(dialog => dialog.$set({ open: state }))
  }

  /**
   * A prop (simply named `detail`) passed to the {@link component} that has
   * been slotted.
   */
  get detail() {
    return this._detail
  }

  /**
   * A prop (simply named `detail`) passed to the {@link component} that has
   * been slotted.
   */
  set detail(detail: Record<string, any>) {
    this._detail = detail
    this.whenLoaded(dialog => dialog.$set({ detail }))
  }

  /**
   * Sets the modal's state.
   *
   * @param state - The new state, if any. If not given, the modal will be toggled.
   */
  toggle(state?: boolean) {
    if (state === undefined) state = !this.open
    this.open = state
  }

  /**
   * Adds a listener for {@link Dialog} events. Be aware that you can't
   * remove a listener once it has been added. This function can be used
   * instead of the `onFoo` methods provided by this class, particularly if
   * you need to add more than one listener.
   *
   * @param event - The event to listen for.
   * @param callback - The callback to call when the event is fired.
   */
  async addEventListener(
    type: "change" | "open" | "close" | "cancel",
    callback: (evt: Event & { detail?: boolean }) => void
  ) {
    await this.loading
    this.dialog!.$on(type, callback)
  }

  /** Destroys the modal. */
  async destroy() {
    await this.loading
    this.dialog!.$destroy()
  }
}
