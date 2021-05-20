<!--
  @component Wikijump's primary page editor.
-->
<script lang="ts">
  import { SheafCore } from "sheaf-core"
  import type { SheafBindings } from "sheaf-core/src/bindings"
  import { setContext } from "svelte"
  import type { Readable } from "svelte/store"
  import { matchBreakpoint, PreferenceHandler } from "wj-state"
  import type { SheafContext } from "./context"
  import { getDefaultSheafSettings } from "./context"
  import PaneEditor from "./PaneEditor.svelte"
  import PaneEditorTopbar from "./PaneEditorTopbar.svelte"
  import PanePreview from "./PanePreview.svelte"

  /** Height of the editor's container. */
  export let height = "100%"
  /** Width of the editor's container. */
  export let width = "100%"
  /** The value of the editor's contents. */
  export let doc = ""
  /** Callbacks to call depending on editor events. */
  export let bindings: SheafBindings = {}

  // setup context, which is shared across all child components
  // this is so that we don't have to pass everything in as component attributes

  const editor = new SheafCore()

  const settings = new PreferenceHandler("_sheaf_").bind(
    "settings",
    getDefaultSheafSettings()
  )

  const small: Readable<boolean> = {
    subscribe: sub => matchBreakpoint.subscribe(fn => sub(fn("<normal")))
  }

  const ctx: SheafContext = {
    editor,
    bindings,
    settings,
    small
  }

  setContext("sheaf", ctx)
</script>

<div class="sheaf-container" style="width: {width}; height: {height};">
  <div class="sheaf-panes">
    <div class="sheaf-pane sheaf-pane-editor">
      <PaneEditorTopbar />
      <PaneEditor {doc} />
    </div>

    {#if $settings.preview.enabled && !$small}
      <div class="sheaf-pane sheaf-pane-preview">
        <PanePreview />
      </div>
    {/if}
  </div>
</div>

<style lang="scss">
  .sheaf-container {
    position: relative;
    display: flex;
    flex-direction: column;
    overflow: auto;
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
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100%;
    contain: strict;
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
