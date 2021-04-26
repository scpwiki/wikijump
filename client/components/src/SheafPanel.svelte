<!--
  @component
  CodeMirror panel.
-->
<script lang="ts">
  import type { EditorView, ViewUpdate } from "sheaf-core"

  export let view: EditorView
  export let update: ViewUpdate
  export let unmount: () => void
</script>

<div class="codemirror-panel">
  <button class="close-panel" role="button" on:click={unmount}>X</button>
</div>

<style>
  .codemirror-panel {
    min-height: 2rem;
  }
  .close-panel {
    position: absolute;
    right: 1rem;
    top: 1rem;
  }
</style>
