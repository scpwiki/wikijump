import type { SvelteComponent } from "svelte"
import type { EditorView, ViewUpdate } from "@codemirror/view"
import { DisconnectElement } from "./svelte-disconnect-detect"

export interface EditorSvelteComponentProps {
  /** The {@link EditorView} the component is mounted to. */
  view: EditorView
  /** The last {@link ViewUpdate} of the editor. */
  update: ViewUpdate | undefined
}

export interface EditorSvelteComponentInstance {
  /** DOM container that holds the Svelte component. */
  dom: DisconnectElement
  /** Function that needs to be called whenever the DOM container is mounted. */
  mount: () => void
  /** Function that needs to be called whenever the view updates. */
  update: (update: ViewUpdate) => void
}

export interface EditorSvelteComponentOpts<T extends SvelteComponent> {
  /** Props to pass to the component on mount. */
  pass?: Record<string, any>
  /**
   * Callback called immediately after the component is mounted.
   *
   * @param component The component that was just mounted.
   */
  mount?: (component: T) => void
  /** Callback called immediately after the component is unmounted. */
  unmount?: () => void
}

/**
 * Handler class for using Svelte components in the CodeMirror DOM.
 *
 * The component is provided with two props:
 * * `view`
 * * `update`
 *
 * You can see the types of these props in the {EditorSvelteComponentProps} interface.
 * @see {EditorSvelteComponentProps}
 */
export class EditorSvelteComponent<T extends typeof SvelteComponent> {
  /**
   * @param component The Svelte component to be mounted.
   */
  constructor(public component: T) {}

  /**
   * Creates the DOM container and lifecycle functions needed to mount a
   * Svelte component into CodeMirror structures, such as panels and tooltips.
   *
   * @param view The {@link EditorView} that the component will be attached to.
   * @param EditorSvelteComponentOptsntCreateOpts}
   */
  create(
    view: EditorView,
    opts: EditorSvelteComponentOpts<InstanceType<T>> = {}
  ): EditorSvelteComponentInstance {
    const dom = document.createElement(DisconnectElement.tag) as DisconnectElement
    let component: SvelteComponent

    const onDisconnect = () => {
      if (component) component.$destroy()
    }

    const mount = () => {
      dom.addEventListener("disconnected", onDisconnect)

      const unmount = () => {
        dom.removeEventListener("disconnected", onDisconnect)
        if (component) component.$destroy()
        if (opts.unmount) opts.unmount()
      }

      component = new this.component({
        target: dom,
        intro: true,
        props: { view, unmount, update: undefined, ...opts.pass }
      })

      if (opts.mount) opts.mount(component as InstanceType<T>)
    }

    const update = (update: ViewUpdate) => {
      if (!component) return
      const view = update.view
      component.$set({ view, update })
    }

    return { dom, mount, update }
  }
}
