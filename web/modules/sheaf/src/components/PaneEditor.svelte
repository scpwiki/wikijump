<!--
  @component Sheaf Editor: Editor Pane.
-->
<script lang="ts">
  import { getContext, onMount } from "svelte"
  import type { SheafContext } from "../context"

  const { editor, bindings, settings, small } = getContext<SheafContext>("sheaf")

  let editorElement: HTMLElement

  $: $editor.gutters = !$small

  onMount(() => editor.mount(editorElement))
</script>

<div class="sheaf-editor-container">
  <div class="sheaf-editor-view" bind:this={editorElement} />
</div>

<style global lang="scss">
  .sheaf-editor-container {
    grid-area: "editor";
  }
  .sheaf-editor-view {
    width: 100%;
    height: 100%;
    contain: strict;
  }
</style>
