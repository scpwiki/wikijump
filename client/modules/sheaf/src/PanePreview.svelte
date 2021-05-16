<!--
  @component Sheaf Editor: Preivew Pane.
-->
<script lang="ts">
  import type { SheafCore } from "sheaf-core"
  import { createAnimQueued, createMutatingLock, perfy } from "wj-util"
  import { Card, Tabview, Tab } from "components"
  import CodeDisplay from "./CodeDisplay.svelte"
  import { RenderHandler } from "./render-handler"
  import morph from "morphdom"

  export let theme: "light" | "dark" = "light"
  export let Editor: SheafCore

  let previewElement: HTMLElement

  let render = new RenderHandler()

  let perfRender = 0
  let perfMorph = 0

  $: if ($Editor.doc) {
    render = new RenderHandler($Editor.doc)
  }

  const morphPreview = createAnimQueued((html: string | Node) => {
    const measureMorph = perfy()
    morph(previewElement, html, {
      childrenOnly: true,
      onBeforeElUpdated: function (fromEl, toEl) {
        if (fromEl.isEqualNode(toEl)) return false
        return true
      }
    })
    perfMorph = measureMorph()
  })

  const getFragment = createMutatingLock(async (handler: RenderHandler) => {
    const measure = perfy()
    const fragment = await handler.fragment()
    perfRender = measure()
    return fragment
  })

  $: if (render && previewElement) {
    getFragment(render).then(fragment => {
      if (fragment) morphPreview(fragment)
    })
  }
</script>

<div class="sheaf-preview-container {theme} codetheme-{theme}">
  <Tabview noborder contained compact conditional>
    <Tab>
      <span slot="button">Result</span>
      <div class="sheaf-preview-perf-panel">
        <Card title="Performance" theme="dark" width="10rem">
          <div><strong>RENDER:</strong> <code>{perfRender}ms</code></div>
          <div><strong>MORPH:</strong> <code>{perfMorph}ms</code></div>
        </Card>
      </div>
      <div class="sheaf-preview wikitext" bind:this={previewElement} />
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

  .sheaf-preview-perf-panel {
    position: absolute;
    top: 1rem;
    right: 1rem;
  }
</style>
