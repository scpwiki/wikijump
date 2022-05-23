import {
  Compartment,
  EditorState,
  StateEffect,
  StateField,
  Transaction,
  type Extension
} from "@codemirror/state"
import { EditorView } from "@codemirror/view"
import { writable, type Writable } from "svelte/store"

export interface EditorFieldOpts<T> {
  /** The default value for the field when it is created. */
  default: T

  /**
   * Function that runs when the view is updated. This does not replace the
   * `update` method the `StateField` object is created with, instead this
   * is ran after that method determines the field's value.
   *
   * If a value that isn't undefined is returned, that'll replace the field's value.
   */
  update?: (value: T, transaction: Transaction, changed: boolean) => T | undefined | void

  /** Allows for providing values to facets, or even just purely adding extensions. */
  provide?: (field: StateField<T>) => Extension

  /**
   * Function that, if given, will reconfigure a `Compartment` with the
   * returned `Extension` when this field updates. Return null to indicate
   * no extensions, return false to indicate that the extensions should not
   * actually be reconfigured.
   */
  reconfigure?: (value: T, last: T | null) => Extension | null | false
}

/**
 * Smart handler for adding fields to CodeMirror editor instances.
 *
 * @typeParam T - The value that the field contains.
 */
export class EditorField<T> {
  /**
   * The `StateEffect` for the field. This is a unique object that is
   * solely capable of modifying the field's value.
   */
  private declare effect

  /**
   * The `StateField` for the field. This is a object that describes the
   * behavior of the field, such as how it is created or updated.
   */
  private declare field

  /** A compartment for extension reconfiguration. */
  private declare compartment

  /** Function that determines what extensions should be given to the compartment. */
  private declare reconfigure?: (value: T, last: T | null) => Extension | null | false

  /**
   * The extension that mounts this field to an editor. Additionally,
   * providing this field allows for simply treating any instance of an
   * `EditorField` as an extension.
   */
  declare extension: Extension

  /**
   * A mapping of `EditorView` objects to observables, for the purpose of
   * tracking their existence and for updating them.
   */
  private observableMap = new Map<EditorView, Writable<T>>()

  /** @param opts - Configuration for this field. A default state is required. */
  constructor(opts: EditorFieldOpts<T>) {
    this.effect = StateEffect.define<T>()

    this.field = StateField.define<T>({
      create: () => opts.default,
      provide: opts.provide,
      update: (value, tr) => {
        let out = value
        let changed = false

        // check if this transaction has our effect(s)
        for (const effect of tr.effects) {
          if (effect.is(this.effect)) {
            out = effect.value
            changed = true
          }
        }

        // run the optional update function, mutate output if needed
        if (opts.update) {
          const result = opts.update(value, tr, changed)
          if (result !== undefined) out = result
        }

        return out
      }
    })

    if (opts.reconfigure) {
      this.compartment = new Compartment()
      this.reconfigure = opts.reconfigure
      const defaultExtensions = this.reconfigure(opts.default, null)
      this.extension = [this.field, this.compartment.of(defaultExtensions || [])]
    } else {
      this.extension = this.field
    }
  }

  /**
   * Gets the current value for this field.
   *
   * @param state - The `EditorState` to source the value from.
   */
  get(state: EditorState): T
  /**
   * Gets the current value for this field.
   *
   * @param view - The `EditorView` to source the value from.
   */
  get(view: EditorView): T
  get(source: EditorView | EditorState): T {
    if (source instanceof EditorView) {
      return source.state.field(this.field)
    } else {
      return source.field(this.field)
    }
  }

  /**
   * Sets the value for this field.
   *
   * @param view - The `EditorView` to dispatch the change to.
   * @param state - The value to set the field to.
   */
  set(view: EditorView, state: T) {
    const from = this.get(view)
    if (from === state) return

    view.dispatch({ effects: this.effect.of(state) })

    const to = this.get(view)

    if (from !== to) {
      // reconfigure compartment
      if (this.reconfigure && this.compartment) {
        const extensions = this.reconfigure(to, from)
        if (extensions !== false) {
          view.dispatch({ effects: this.compartment.reconfigure(extensions ?? []) })
        }
      }

      // inform observers
      if (this.observableMap.size) {
        for (const [obView, observable] of this.observableMap) {
          if (obView === view) observable.set(to)
        }
      }
    }
  }

  /**
   * Returns an extension that mounts this field, but using a different
   * creation value.
   *
   * @param value - The value to set the field to on creation.
   */
  of(value: T) {
    return this.field.init(() => value)
  }

  /**
   * Returns a Svelte-compatible observable for reactively reading and
   * updating this field. If a observable already exists for the view
   * given, it'll simply be reused. This means it is safe to call this
   * method repeatedly for a view.
   *
   * @param view - The `EditorView` to attach the observable to.
   */
  bind(view: EditorView): Writable<T> {
    if (this.observableMap.has(view)) return this.observableMap.get(view)!

    // create an observer that automatically adds and
    // deletes itself from the observableMap
    const observable = writable(this.get(view), () => {
      this.observableMap.set(view, observable)
      return () => void this.observableMap.delete(view)
    })

    // create a handler around that observable so that we update the editor state
    return {
      subscribe: observable.subscribe,
      // subscribers get informed when the state updates,
      // so we don't want to do a double update by informing them again here
      set: value => this.set(view, value),
      update: updater => {
        const value = updater(this.get(view))
        this.set(view, value)
      }
    }
  }
}
