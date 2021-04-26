import { Extension, StateEffect, StateEffectType, StateField } from "@codemirror/state"
import { Panel, showPanel } from "@codemirror/panel"
import type { SvelteComponent } from "svelte"
import type { EditorView, ViewUpdate } from "@codemirror/view"

export type { EditorView, ViewUpdate }

export class EditorSveltePanel {
  declare name: string
  declare extension: Extension

  private declare panelEffect: StateEffectType<boolean>
  private declare panelState: StateField<boolean>

  constructor(
    name: string,
    public component: typeof SvelteComponent,
    public top = false
  ) {
    // create a custom element that can be used to detect when the panel is destroyed
    this.name = `svelte-cm-${name}`
    if (!customElements.get(name)) {
      customElements.define(
        name,
        class extends HTMLElement {
          disconnectedCallback() {
            this.dispatchEvent(new CustomEvent("disconnected"))
          }
        }
      )
    }

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

  private create(view: EditorView): Panel {
    const dom = document.createElement(this.name)
    let component!: SvelteComponent

    const mount = () => {
      const unmount = () => {
        component.$destroy()
        this.toggle(view, false)
      }
      dom.addEventListener("disconnected", () => component.$destroy())
      component = new this.component({
        target: dom,
        intro: true,
        props: { view, unmount, update: undefined }
      })
    }

    const update = (update: ViewUpdate) => {
      if (!component) return
      const view = update.view
      component.$set({ view, update })
    }

    return { dom, mount, update, top: this.top }
  }

  toggle(view: EditorView, state?: boolean) {
    if (state === undefined) state = !view.state.field(this.panelState)
    const effect = this.panelEffect
    view.dispatch({
      effects: effect.of(state)
    })
  }
}
