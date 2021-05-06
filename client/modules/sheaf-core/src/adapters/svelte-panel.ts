import { Extension, StateEffect, StateEffectType, StateField } from "@codemirror/state"
import { Panel, showPanel } from "@codemirror/panel"
import type { SvelteComponent } from "svelte"
import type { EditorView } from "@codemirror/view"
import { EditorSvelteDOM, EditorSvelteDOMProps } from "./svelte-dom"

/**
 * The props provided to a {@link EditorSveltePanel} component.
 * @see {@link EditorSveltePanel}
 */
export interface EditorSveltePanelProps extends EditorSvelteDOMProps {
  /** Calls `$destroy()` on the component and then unmounts the panel. */
  unmount: () => void
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
export class EditorSveltePanel {
  /**
   * Extension that mounts the panel to the editor.
   * You don't really need to use this property - any object with the `extension`
   * property is a valid CodeMirror extension entrypoint.
   */
  declare extension: Extension

  private declare panelEffect: StateEffectType<boolean>
  private declare panelState: StateField<boolean>
  private declare handler: EditorSvelteDOM

  /**
   * @param component The Svelte component the panel will mount with.
   * @param top If true, the panel will be mounted on the top of the editor.
   */
  constructor(public component: typeof SvelteComponent, public top = false) {
    this.handler = new EditorSvelteDOM(component)
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
    const instance = this.handler.create(view, () => this.toggle(view, false))
    return { ...instance, top: this.top }
  }

  /**
   * Toggle, or directly set, the panel's state (whether or not it is mounted).
   *
   * @param view The {@link EditorView} that the panel is attached to.
   * @param state Forces the panel to either mount or unmount.
   */
  toggle(view: EditorView, state?: boolean) {
    if (state === undefined) state = !view.state.field(this.panelState)
    view.dispatch({ effects: this.panelEffect.of(state) })
  }
}
