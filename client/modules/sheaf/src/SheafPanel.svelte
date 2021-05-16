<!--
  @component CodeMirror test panel.
-->
<script lang="ts">
  import { Button } from "components"
  import type { EditorSveltePanelProps } from "sheaf-core"

  export let view: EditorSveltePanelProps["view"]
  export let update: EditorSveltePanelProps["update"]
  export let unmount: EditorSveltePanelProps["unmount"]
</script>

<div class="codemirror-panel">
  <div class="close-panel">
    <Button i="ion:close" size="1.5rem" tip="Close Panel" baseline on:click={unmount} />
  </div>
  This is a test panel.
</div>

<style>
  .codemirror-panel {
    min-height: 2rem;
  }
  .close-panel {
    position: absolute;
    top: 0.25rem;
    right: 0.5rem;
  }
</style>
