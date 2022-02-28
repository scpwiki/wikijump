<!--
  @component CodeMirror test panel.
-->
<script lang="ts">
  import { FTMLLanguage } from "@wikijump/cm-lang-ftml"
  import type { EditorSveltePanelProps } from "@wikijump/codemirror"
  import { Button } from "@wikijump/components"
  import { unit } from "@wikijump/fluent"

  export let view: EditorSveltePanelProps["view"]
  export let update: EditorSveltePanelProps["update"]
  export let unmount: EditorSveltePanelProps["unmount"]

  let ftmlPerfs: number[] = []

  $: if (update) {
    const perf = FTMLLanguage.performance
    if (perf !== ftmlPerfs[0]) {
      ftmlPerfs.unshift(perf)
      if (ftmlPerfs.length > 5) ftmlPerfs.pop()
      // tells svelte to check the value
      ftmlPerfs = [...ftmlPerfs]
    }
  }
</script>

<div class="codemirror-panel">
  <div class="close-panel">
    <Button i="wj-close" size="1.5rem" tip="Close Panel" baseline on:click={unmount} />
  </div>
  <span>FTML Performance:</span>
  {#each ftmlPerfs as perf}
    <code>{unit(perf, "millisecond", { unitDisplay: "narrow" })}</code>
  {/each}
</div>

<style global lang="scss">
  .codemirror-panel {
    display: flex;
    align-items: center;
    min-height: 2rem;
    padding: 0 0.25rem;
    background: var(--col-background);

    > span {
      font-weight: bold;
    }

    > code {
      display: block;
      min-width: 2rem;
      padding: 0 0.5rem;
      text-align: center;

      &:not(:last-child) {
        border-right: solid 0.125rem var(--col-border);
      }
    }
  }
  .close-panel {
    position: absolute;
    top: 0.25rem;
    right: 0.5rem;
  }
</style>
