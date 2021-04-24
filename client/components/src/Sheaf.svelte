<!--
  @component
  Wikijump's primary page editor.
-->
<script lang="ts">
  import { SheafCore } from "sheaf-core"
  import { FTMLLanguage } from "cm-lang-ftml"
  import { perfy } from "wj-util"
  import * as FTML from "ftml-wasm-worker"
  import { onMount } from "svelte"

  /** Height of the editor's container. */
  export let height = "100%"

  /** Width of the editor's container.*/
  export let width = "100%"

  /** The value of the editor's contents. */
  export let doc = ""

  let editorElement: HTMLElement

  const Editor = new SheafCore()

  $: if ($Editor.value) {
    const report = perfy("ftml-perf", 5)
    FTML.render($Editor.value).then(({ html, style }) => {
      // console.log(html)
      // console.log(style)
      report()
    })
  }

  onMount(async () => {
    await Editor.init(editorElement, doc, [FTMLLanguage.load()])
  })
</script>

<div class="overflow-container" style="height: {height}; width: {width};">
  <div bind:this={editorElement} class="editor-container codetheme-dark" />
</div>

<style>
  .overflow-container {
    position: relative;
    overflow: hidden;
    background: var(--colcode-background);
  }

  .editor-container {
    height: 100%;
    width: 100%;
  }
</style>
