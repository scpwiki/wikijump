<!--
  @component Sheaf Editor: Preivew Pane.
-->
<script lang="ts">
  import type { SheafCore } from "sheaf-core"
  import { Tabview, Tab, Wikitext } from "components"
  import CodeDisplay from "./CodeDisplay.svelte"
  import { RenderHandler } from "./render-handler"

  /** Theme of the preview. */
  export let theme: "light" | "dark" = "light"

  /** Reference to the editor-core that will be previewed. */
  export let Editor: SheafCore

  let render = new RenderHandler()

  $: if ($Editor.doc) {
    render = new RenderHandler($Editor.doc)
  }
</script>

<div class="sheaf-preview-container {theme} codetheme-{theme}">
  <Tabview noborder contained compact conditional>
    <Tab>
      <span slot="button">Result</span>
      <div class="sheaf-preview">
        <Wikitext morph wikitext={() => render.result()} />
      </div>
    </Tab>

    <Tab>
      <span slot="button">HTML Output</span>
      <CodeDisplay content={render.html(true)} lang="html" />
    </Tab>

    <Tab>
      <span slot="button">CSS Output</span>
      <CodeDisplay content={render.style()} lang="css" />
    </Tab>

    <Tab>
      <span slot="button">AST</span>
      <CodeDisplay content={render.stringifiedAST()} lang="json" />
    </Tab>
  </Tabview>
</div>

<style lang="scss">
  .sheaf-preview-container {
    height: 100%;
  }

  .sheaf-preview {
    width: 100%;
    height: 100%;
    padding: 1rem;
    overflow-y: auto;
  }
</style>
