<!--
  @component Sheaf Editor: Preview Pane.
-->
<script lang="ts">
  import { Tab, Tabview, Wikitext } from "components"
  import { getContext } from "svelte"
  import CodeDisplay from "./CodeDisplay.svelte"
  import type { SheafContext } from "./context"
  import { RenderHandler } from "./render-handler"
  import { t } from "wj-state"
  import { printTree } from "sheaf-core"

  const { editor, bindings, settings, small } = getContext<SheafContext>("sheaf")

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
      <span slot="button">{$t("sheaf.preview_tabs.RESULT")}</span>
      <div class="sheaf-preview">
        <Wikitext morph {debug} offline wikitext={() => render.result()} />
      </div>
    </Tab>

    <Tab>
      <span slot="button">{$t("sheaf.preview_tabs.HTML")}</span>
      <CodeDisplay content={render.html(true)} lang="html" />
    </Tab>

    <Tab>
      <span slot="button">{$t("sheaf.preview_tabs.CSS")}</span>
      <CodeDisplay content={render.style()} lang="css" />
    </Tab>

    {#if debug}
      <Tab>
        <span slot="button">{$t("sheaf.preview_tabs.AST")}</span>
        <CodeDisplay content={render.stringifiedAST()} lang="json" />
      </Tab>

      <Tab>
        <span slot="button">{$t("sheaf.preview_tabs.TOKENS")}</span>
        <CodeDisplay content={render.inspectTokens()} lang="FTMLTokens" />
      </Tab>

      <Tab>
        <span slot="button">{$t("sheaf.preview_tabs.EDITOR_AST")}</span>
        <CodeDisplay
          content={$editor.value().then(value => printTree($editor.tree, value))}
          lang="LezerTree"
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
