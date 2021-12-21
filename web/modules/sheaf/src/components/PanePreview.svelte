<!--
  @component Sheaf Editor: Preview Pane.
-->
<script lang="ts">
  import { Tab, Tabview, Wikitext } from "@wikijump/components"
  import { getContext } from "svelte"
  import CodeDisplay from "./CodeDisplay.svelte"
  import type { SheafContext } from "../context"
  import { RenderHandler } from "../render-handler"
  import Locale from "@wikijump/fluent"
  import { printTree } from "@wikijump/codemirror"

  const t = Locale.makeComponentFormatter("sheaf")

  const { editor, bindings, settings, small } = getContext<SheafContext>("sheaf")

  let render = new RenderHandler()

  $: debug = $settings.debug

  $: if ($editor.doc) {
    render = new RenderHandler($editor.doc)
  }
</script>

<div class="sheaf-preview-container">
  <Tabview noborder contained conditional>
    <Tab>
      <span slot="button">{$t("#-preview.result")}</span>
      <div class="sheaf-preview">
        <Wikitext morph {debug} wikitext={() => render.result()} />
      </div>
    </Tab>

    <Tab>
      <span slot="button">{$t("#-preview.html")}</span>
      <CodeDisplay content={render.html(true)} lang="html" />
    </Tab>

    <Tab>
      <span slot="button">{$t("#-preview.css")}</span>
      <CodeDisplay content={render.style()} lang="css" />
    </Tab>

    {#if debug}
      <Tab>
        <span slot="button">{$t("#-preview.ast")}</span>
        <CodeDisplay content={render.stringifiedAST()} lang="json" />
      </Tab>

      <Tab>
        <span slot="button">{$t("#-preview.tokens")}</span>
        <CodeDisplay content={render.inspectTokens()} lang="FTMLTokens" />
      </Tab>

      <Tab>
        <span slot="button">{$t("#-preview.editor-ast")}</span>
        <CodeDisplay
          content={$editor.value().then(value => printTree($editor.tree, value))}
          lang="PrintTree"
        />
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
    padding-bottom: 10rem;
    overflow-y: auto;
  }
</style>
