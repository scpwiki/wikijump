<!--
  @component
  CodeMirror panel.
-->
<script lang="ts">
  import type { EditorSveltePanelProps } from "sheaf-core"

  export let view: EditorSveltePanelProps["view"]
  export let update: EditorSveltePanelProps["update"]
  export let unmount: EditorSveltePanelProps["unmount"]
</script>

<div class="codemirror-panel">
  <button class="close-panel" role="button" on:click={unmount}>X</button>
  This is a test panel.
</div>

<style>
  .codemirror-panel {
    min-height: 2rem;
  }
  .close-panel {
    position: absolute;
    right: 1rem;
    top: 0.25rem;
  }
</style>
