<!--
  @component Sheaf Editor: Editor Pane.
-->
<script lang="ts">
  import { getContext, onMount } from "svelte"
  import type { SheafContext } from "../context"

  const { editor, bindings, settings, small } = getContext<SheafContext>("sheaf")

  let editorElement: HTMLElement

  $: theme = $settings.editor.darkmode ? "dark" : "light"

  $: $editor.gutters = !$small

  onMount(() => editor.mount(editorElement))
</script>

<div class="sheaf-editor-container {theme} codetheme-{theme}">
  <div class="sheaf-editor-view" bind:this={editorElement} />
</div>

<style lang="scss">
  .sheaf-editor-container {
    grid-area: "editor";
  }
  .sheaf-editor-view {
    width: 100%;
    height: 100%;
  }
</style>
