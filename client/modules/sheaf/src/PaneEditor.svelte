<!--
  @component Sheaf Editor: Editor Pane.
-->
<script lang="ts">
  import { SheafCore, EditorSveltePanel } from "sheaf-core"
  import { FTMLLanguage } from "cm-lang-ftml"
  import type { SheafBindings } from "sheaf-core/src/bindings"
  import { onMount } from "svelte"

  import SheafPanel from "./SheafPanel.svelte"

  /** The value of the editor's contents. */
  export let doc = ""

  /** Callbacks to call depending on editor events. */
  export let bindings: SheafBindings = {}

  /** Reference to the editor-core that the editor-pane will render. */
  export let Editor: SheafCore

  let editorElement: HTMLElement

  const TestPanel = new EditorSveltePanel(SheafPanel, { top: true })

  onMount(async () => {
    await Editor.init(editorElement, doc, bindings, [FTMLLanguage.load(), TestPanel])
  })
</script>

<div class="sheaf-editor-view" bind:this={editorElement} />

<style lang="scss">
  .sheaf-editor-view {
    width: 100%;
    height: 100%;
  }
</style>
