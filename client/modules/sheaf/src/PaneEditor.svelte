<!--
  @component Sheaf Editor: Editor Pane.
-->
<script lang="ts">
  import { FTMLLanguage } from "cm-lang-ftml"
  import { EditorSveltePanel } from "sheaf-core"
  import { getContext, onMount } from "svelte"
  import type { SheafContext } from "./context"
  import SheafPanel from "./SheafPanel.svelte"

  /** The value of the editor's contents. */
  export let doc = ""

  const { editor, bindings, settings } = getContext<SheafContext>("sheaf")

  let editorElement: HTMLElement

  $: theme = $settings.editor.darkmode ? "dark" : "light"

  const TestPanel = new EditorSveltePanel(SheafPanel, { top: true })

  onMount(async () => {
    await editor.init(editorElement, doc, bindings, [FTMLLanguage.load(), TestPanel])
  })
</script>

<div class="sheaf-editor-container {theme} codetheme-{theme}">
  <div class="sheaf-editor-view" bind:this={editorElement} />
</div>

<style lang="scss">
  .sheaf-editor-container,
  .sheaf-editor-view {
    width: 100%;
    height: 100%;
  }
</style>
