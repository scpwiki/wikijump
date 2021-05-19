<!--
  @component Sheaf Editor: Preview Pane.
-->
<script lang="ts">
  import { Tab, Tabview, Wikitext } from "components"
  import { getContext } from "svelte"
  import CodeDisplay from "./CodeDisplay.svelte"
  import type { SheafContext } from "./context"
  import { RenderHandler } from "./render-handler"

  const { editor, bindings, settings } = getContext<SheafContext>("sheaf")

  let render = new RenderHandler()

  $: debug = $settings.debug
  $: theme = $settings.preview.darkmode ? "dark" : "light"

  $: if ($editor.doc) {
    render = new RenderHandler($editor.doc)
  }
</script>

<div class="sheaf-preview-container {theme} codetheme-{theme}">
  <Tabview noborder contained compact conditional>
    <Tab>
      <span slot="button">Result</span>
      <div class="sheaf-preview">
        <Wikitext morph {debug} wikitext={() => render.result()} />
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

    {#if debug}
      <Tab>
        <span slot="button">AST</span>
        <CodeDisplay content={render.stringifiedAST()} lang="json" />
      </Tab>
    {/if}
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
