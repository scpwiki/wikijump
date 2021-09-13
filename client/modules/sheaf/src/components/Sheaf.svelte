<!--
  @component Wikijump's primary page editor.
-->
<script lang="ts">
  import { EditorSveltePanel } from "@wikijump/codemirror"
  import { SheafCore } from "../core"
  import type { SheafBindings } from "../extensions/bindings"
  import { setContext } from "svelte"
  import type { Readable } from "svelte/store"
  import { matchBreakpoint, PreferenceHandler } from "@wikijump/state"
  import type { SheafContext } from "../context"
  import { getDefaultSheafSettings } from "../context"
  import PaneEditor from "./PaneEditor.svelte"
  import PaneEditorTopbar from "./PaneEditorTopbar.svelte"
  import PanePreview from "./PanePreview.svelte"
  import SheafPanel from "./SheafPanel.svelte"
  import { FTMLLanguage } from "@wikijump/cm-lang-ftml"

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

  const TestPanel = new EditorSveltePanel(SheafPanel, { top: true })
  const editor = new SheafCore(doc, bindings, [FTMLLanguage.load(), TestPanel])
  TestPanel.toggle(editor.state.view, true)

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
      <PaneEditor />
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
    display: flex;
    flex-direction: column;
    background: var(--colcode-background);
    contain: strict;
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
    height: 100%;
  }

  .sheaf-pane-editor {
    display: grid;
    flex-grow: 1;
    grid-template-areas:
      "topbar"
      "editor";
    grid-template-rows: 2.25rem calc(100% - 2.25rem);
    grid-template-columns: 1fr;
    min-width: 50%;
  }

  .sheaf-pane-preview {
    flex-shrink: 1;
    width: 100%;
    max-width: var(--layout-body-max-width);
  }
</style>
