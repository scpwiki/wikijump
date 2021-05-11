<!--
  @component Autocomplete tooltip for FTML blocks.
-->
<script lang="ts">
  import type { Block } from "../data/types"
  import type { EditorSvelteComponentProps } from "sheaf-core"
  import * as Prism from "wj-prism"

  export let name: string
  export let block: Block
  export let info = ""
  export let unmount: EditorSvelteComponentProps["unmount"]

  const aliases = Array.from(new Set([name, ...(block.aliases ?? [])]))
  const [outputType, outputTag, outputClass] = block["html-output"].split(",")

  let codeString =
    outputType === "html"
      ? outputClass
        ? `<${outputTag} class="${outputClass}">`
        : `<${outputTag}>`
      : `type: ${outputType}`

  codeString = Prism.highlight(codeString, outputType === "html" ? "html" : "log")
</script>

<div class="cm-ftml-block-tip">
  <h4 class="cm-ftml-block-tip-name">
    {name}
  </h4>

  {#if aliases.length > 1}
    <div class="cm-ftml-block-tip-aliases">
      {#each aliases as alias}
        <span>'{alias}'</span>
      {/each}
    </div>
  {/if}

  <div class="wikitext cm-ftml-block-tip-info">
    {#if !info}
      <p><i>This block has no documentation.</i></p>
    {:else}
      {@html info}
    {/if}
  </div>

  <hr />

  <pre class="code cm-ftml-block-tip-html">
    <code>{@html codeString}</code>
  </pre>
</div>

<style lang="scss">
  .cm-ftml-block-tip {
    color: var(--col-text);
    background: none;
  }

  .cm-ftml-block-tip-aliases {
    font-family: var(--font-mono);
    font-size: 0.75em;
    > span {
      margin-right: 0.5em;
      color: var(--colcode-string);
    }
  }

  .cm-ftml-block-tip-info {
    margin-top: 0.5rem;
    margin-bottom: 0.25rem;
  }

  .cm-ftml-block-tip-html {
    display: block;
    margin-top: 0.25rem;
    font-size: 1em;
    color: var(--colcode-content);
    > code {
      overflow-x: visible;
    }
  }
</style>
