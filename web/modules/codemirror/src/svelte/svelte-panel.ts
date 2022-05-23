import type { Extension } from "@codemirror/state"
import { EditorView, showPanel, type Panel } from "@codemirror/view"
import type { SvelteComponent } from "svelte"
import { EditorField } from "../editor-field"
import {
  EditorSvelteComponent,
  type EditorSvelteComponentOpts,
  type EditorSvelteComponentProps
} from "./svelte-dom"

/**
 * The props provided to a {@link EditorSveltePanel} component.
 *
 * @see {@link EditorSveltePanel}
 */
export interface EditorSveltePanelProps extends EditorSvelteComponentProps {
  /** Calls `$destroy()` on the component and then unmounts the panel. */
  unmount: () => void
}

export interface EditorSveltePanelOpts<T extends SvelteComponent>
  extends EditorSvelteComponentOpts<T> {
  /** If true, the panel will be mounted on the top of the editor. */
  top?: boolean
}

/**
 * A panel that uses a Svelte component to render its contents.
 *
 * The component is provided with three props:
 *
 * - `view`
 * - `update`
 * - `unmount`
 *
 * You can see the types of these props in the
 * {@link EditorSveltePanelProps} interface.
 *
 * @see {@link EditorSveltePanelProps}
 */
export class EditorSveltePanel<T extends typeof SvelteComponent> {
  /**
   * Extension that mounts the panel to the editor. You don't really need
   * to use this property - any object with the `extension` property is a
   * valid CodeMirror extension entrypoint.
   */
  declare extension: Extension

  private declare field: EditorField<boolean>
  private declare handler: EditorSvelteComponent<T>

  /**
   * @param component - The Svelte component the panel will mount with.
   * @param opts - {@link EditorSveltePanelOpts}
   */
  constructor(
    public component: T,
    private opts: EditorSveltePanelOpts<InstanceType<T>> = {}
  ) {
    this.handler = new EditorSvelteComponent(component)

    const create = this.create.bind(this)
    this.field = new EditorField<boolean>({
      default: false,
      provide: field => showPanel.from(field, show => (show ? create : null))
    })

    this.extension = this.field
  }

  /**
   * Creates the Svelte component and DOM container element and returns the
   * CodeMirror panel instance.
   */
  private create(view: EditorView): Panel {
    const instance = this.handler.create(view, {
      unmount: () => this.toggle(view, false)
    })
    return { ...instance, top: this.opts.top }
  }

  /**
   * Toggle, or directly set, the panel's state (whether or not it is mounted).
   *
   * @param view - The {@link EditorView} that the panel is attached to.
   * @param state - Forces the panel to either mount or unmount.
   */
  toggle(view: EditorView, state?: boolean) {
    if (state === undefined) state = !this.field.get(view)
    this.field.set(view, state)
  }
}
