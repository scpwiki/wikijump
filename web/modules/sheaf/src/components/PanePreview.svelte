<!--
  @component Sheaf Editor: Preview Pane.
-->
<script lang="ts">
  import { printTree } from "@wikijump/codemirror"
  import { Tab, Tabview } from "@wikijump/components"
  import Locale from "@wikijump/fluent"
  import { getContext } from "svelte"
  import type { SheafContext } from "../context"
  import { RenderHandler } from "../render-handler"
  import CodeDisplay from "./CodeDisplay.svelte"
  import Wikitext from "./Wikitext.svelte"

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
        <Wikitext morph {debug} wikitext={render.result.bind(render)} />
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

<style global lang="scss">
  .sheaf-preview-container {
    height: 100%;
  }

  .sheaf-preview {
    width: 100%;
    height: 100%;
    padding: 1rem;
    padding-bottom: 10rem;
    overflow-y: auto;
    contain: strict;
  }
</style>
