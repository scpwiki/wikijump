<!--
  @component Sheaf Editor: Preivew Pane.
-->
<script lang="ts">
  import type { SheafCore } from "sheaf-core"
  import { createAnimQueued, perfy, toFragment } from "wj-util"
  import * as FTML from "ftml-wasm-worker"
  import morph from "morphdom"

  import { Card } from "components"

  export let theme: "light" | "dark" = "light"
  export let Editor: SheafCore

  let previewElement: HTMLElement

  let perfRender = 0
  let perfMorph = 0

  const update = createAnimQueued((html: string | Node) => {
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

  $: if ($Editor.value) {
    const measureRender = perfy()
    FTML.render($Editor.value).then(async ({ html, style }) => {
      perfRender = measureRender()
      const fragment = toFragment(html)
      update(fragment)
    })
  }
</script>

<div class="sheaf-preview-container">
  <div class="sheaf-preview-perf-panel">
    <Card title="Performance" theme="dark" width="10rem">
      <div><strong>RENDER:</strong> <code>{perfRender}ms</code></div>
      <div><strong>MORPH:</strong> <code>{perfMorph}ms</code></div>
    </Card>
  </div>
  <div
    class="sheaf-preview wikitext {theme} codetheme-{theme}"
    bind:this={previewElement}
  />
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
