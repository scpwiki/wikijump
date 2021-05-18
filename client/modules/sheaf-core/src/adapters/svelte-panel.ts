import { Panel, showPanel } from "@codemirror/panel"
import { Extension, StateEffect, StateEffectType, StateField } from "@codemirror/state"
import type { EditorView } from "@codemirror/view"
import type { SvelteComponent } from "svelte"
import {
  EditorSvelteComponent,
  EditorSvelteComponentOpts,
  EditorSvelteComponentProps
} from "./svelte-dom"

/**
 * The props provided to a {@link EditorSveltePanel} component.
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
 * * `view`
 * * `update`
 * * `unmount`
 *
 * You can see the types of these props in the {@link EditorSveltePanelProps} interface.
 * @see {@link EditorSveltePanelProps}
 */
export class EditorSveltePanel<T extends typeof SvelteComponent> {
  /**
   * Extension that mounts the panel to the editor.
   * You don't really need to use this property - any object with the `extension`
   * property is a valid CodeMirror extension entrypoint.
   */
  declare extension: Extension

  private declare panelEffect: StateEffectType<boolean>
  private declare panelState: StateField<boolean>
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
    const panelEffect = (this.panelEffect = StateEffect.define<boolean>())
    this.panelState = StateField.define<boolean>({
      create: () => true,
      update(value, tr) {
        for (const effect of tr.effects) {
          if (effect.is(panelEffect)) value = effect.value
        }
        return value
      },
      provide: facet => showPanel.from(facet, show => (show ? create : null))
    })

    this.extension = [this.panelState]
  }

  /**
   * Creates the Svelte component and DOM container element
   * and returns the CodeMirror panel instance.
   */
  private create(view: EditorView): Panel {
    const toggle = this.toggle.bind(this)
    const instance = this.handler.create(view, {
      unmount() {
        toggle(view, false)
      }
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
    if (state === undefined) state = !view.state.field(this.panelState)
    view.dispatch({ effects: this.panelEffect.of(state) })
  }
}
