<!--
  @component Wikijump's primary page editor.
-->
<script lang="ts">
  import { SheafCore } from "sheaf-core"
  import PaneEditor from "./PaneEditor.svelte"
  import PanePreview from "./PanePreview.svelte"
  import type { SheafBindings } from "sheaf-core/src/bindings"

  /** Height of the editor's container. */
  export let height = "100%"

  /** Width of the editor's container. */
  export let width = "100%"

  /** The value of the editor's contents. */
  export let doc = ""

  /** Callbacks to call depending on editor events. */
  export let bindings: SheafBindings = {}

  const Editor = new SheafCore()
</script>

<div
  class="sheaf-container codetheme-dark dark"
  style=" width: {width};height: {height};"
>
  <div class="sheaf-panes">
    <div class="sheaf-pane sheaf-pane-editor">
      <PaneEditor {Editor} {doc} {bindings} />
    </div>

    <div class="sheaf-pane sheaf-pane-preview">
      <PanePreview {Editor} />
    </div>
  </div>
</div>

<style lang="scss">
  .sheaf-container {
    position: relative;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--colcode-background);
  }

  .sheaf-panes {
    display: flex;
    width: 100%;
    height: 100%;

    > *:not(:last-child) {
      border-right: solid 0.25rem var(--col-border);
    }
  }

  .sheaf-pane {
    height: 100%;
  }

  .sheaf-pane-editor {
    flex-grow: 1;
    min-width: 50%;
  }

  .sheaf-pane-preview {
    flex-shrink: 1;
    width: 100%;
    max-width: var(--layout-body-max-width);
  }
</style>
