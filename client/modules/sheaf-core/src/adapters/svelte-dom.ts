import type { EditorView, ViewUpdate } from "@codemirror/view"
import type { SvelteComponent } from "svelte"
import { LifecycleElement } from "./svelte-lifecycle-element"

export interface EditorSvelteComponentProps {
  /**
   * The {@link EditorView} the component is mounted to. This can be
   * undefined if the component was created with no view. In that instance,
   * the component will still track lifecycle but won't be able to interact
   * with the editor.
   */
  view: EditorView | undefined
  /** The last {@link ViewUpdate} of the editor. */
  update: ViewUpdate | undefined
  /** Calls `$destroy()` on the component. */
  unmount: () => void
}

export interface EditorSvelteComponentInstance {
  /** DOM container that holds the Svelte component. */
  dom: LifecycleElement
  /** Function that needs to be called whenever the view updates. */
  update: (update: ViewUpdate) => void
  /**
   * Clones this instance so that it may be used in contexts where reusing
   * the node isn't safe.
   */
  clone: () => EditorSvelteComponentInstance
}

export interface EditorSvelteComponentOpts<T extends SvelteComponent> {
  /** Props to pass to the component on mount. */
  pass?: Record<string, any>
  /**
   * Callback called immediately after the component is mounted.
   *
   * @param component - The component that was just mounted.
   */
  mount?: (component: T) => void
  /**
   * Callback called immediately before the component is unmounted.
   *
   * @param component - The component that is about to be unmounted.
   */
  unmount?: (component: T) => void
}

/**
 * Handler class for using Svelte components in the CodeMirror DOM.
 *
 * The component is provided with two props:
 *
 * - `view`
 * - `update`
 * - `unmount`
 *
 * You can see the types of these props in the
 * {@link EditorSvelteComponentProps} interface.
 *
 * @see {@link EditorSvelteComponentProps}
 */
export class EditorSvelteComponent<T extends typeof SvelteComponent> {
  /** @param component - The Svelte component to be mounted. */
  constructor(public component: T) {}

  /**
   * Creates the DOM container and lifecycle functions needed to mount a
   * Svelte component into CodeMirror structures, such as panels and tooltips.
   *
   * @param view - The {@link EditorView} that the component will be attached to.
   * @param opts - {@link EditorSvelteComponentOpts}
   */
  create(
    view?: EditorView,
    opts: EditorSvelteComponentOpts<InstanceType<T>> = {}
  ): EditorSvelteComponentInstance {
    let component: SvelteComponent | null = null

    const unmount = (dom: LifecycleElement) => {
      // prevent unmount from being called twice, if something else called this function
      dom._unmount = undefined
      if (opts.unmount) opts.unmount(component as InstanceType<T>)
      if (component) component.$destroy()
      component = null
    }

    const mount = (dom: LifecycleElement) => {
      const svelteUnmount = () => unmount(dom)

      component = new this.component({
        target: dom,
        intro: true,
        props: view
          ? { view, unmount: svelteUnmount, update: undefined, ...opts.pass }
          : { unmount: svelteUnmount, ...opts.pass }
      })

      if (opts.mount) opts.mount(component as InstanceType<T>)

      // start listening to unmounting
      dom._unmount = unmount
    }

    const update = (update: ViewUpdate) => {
      if (!component) return
      const view = update.view
      component.$set({ view, update })
    }

    const dom = new LifecycleElement(mount)

    const clone = () => {
      return this.create(view, opts)
    }

    return { dom, update, clone }
  }
}
