import { SvelteComponent } from "svelte"
import Dialog from "../Dialog.svelte"

/** Wraps around the {@link Dialog} component to provide a simple interface for */
export class Modal<T extends typeof SvelteComponent> {
  /** The internal {@link Dialog} component. */
  private dialog: Dialog

  /** Keeps track of the open state. */
  private _open: boolean

  /** @see {@link detail} */
  private _detail: Record<string, any> = {}

  /** The component being slotted into the dialog. */
  readonly component: T

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

  /** The open state of the modal. */
  get open() {
    return this._open
  }

  /** The open state of the modal. */
  set open(state: boolean) {
    if (this._open === state) return
    this.dialog.$set({ open: state })
    this._open = state
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
    this.dialog.$set({ detail })
    this._detail = detail
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
  addEventListener(
    type: "change" | "open" | "close" | "cancel",
    callback: (evt: Event & { detail?: boolean }) => void
  ) {
    // @ts-ignore
    this.dialog.$on(type, callback)
  }

  /** Destroys the modal. */
  destroy() {
    this.dialog.$destroy()
  }
}
