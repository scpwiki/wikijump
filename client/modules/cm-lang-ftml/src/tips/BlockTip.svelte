<!--
  @component Autocomplete tooltip for FTML blocks.
-->
<script lang="ts">
  import type { Block } from "../data/types"
  import type { EditorSvelteComponentProps } from "sheaf-core"
  import * as Prism from "wj-prism"
  import { t } from "wj-state"
  import type { FTMLFragment } from "ftml-wasm-worker"
  import { Icon, tip } from "components"

  interface Docs {
    title: string
    info: FTMLFragment
    example: string
  }

  export let name: string
  export let block: Block
  export let docs: Docs
  export let unmount: EditorSvelteComponentProps["unmount"]

  const aliases = [name, ...(block.aliases ?? [])]
  const deprecated = block["deprecated"] ?? false
  const acceptsStar = block["accepts-star"] ?? false
  const acceptsScore = block["accepts-score"] ?? false
  const acceptsNewlines = block["accepts-newlines"] ?? false
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
  <div class="cm-ftml-block-tip-header">
    <h4 class="cm-ftml-block-tip-title">
      {docs.title}
    </h4>
    <div class="cm-ftml-block-tip-accepts">
      {#if acceptsStar}
        <span use:tip={$t("cmftml.blocks.ACCEPTS_STAR")}>
          <Icon i="fa-solid:asterisk" size="1rem" />
        </span>
      {/if}
      {#if acceptsScore}
        <span use:tip={$t("cmftml.blocks.ACCEPTS_SCORE")}>
          <Icon i="feather:underline" size="1rem" />
        </span>
      {/if}
      {#if acceptsNewlines}
        <span use:tip={$t("cmftml.blocks.ACCEPTS_NEWLINES")}>
          <Icon i="ic:round-keyboard-return" size="1rem" />
        </span>
      {/if}
    </div>
  </div>

  {#if aliases.length > 1}
    <div class="cm-ftml-block-tip-aliases">
      {#each aliases as alias}
        <span>'{alias}'</span>
      {/each}
    </div>
  {/if}

  <div class="wikitext cm-ftml-block-tip-info">
    {#await docs.info.render() then info}
      {@html info.html}
    {/await}
  </div>

  {#if docs.example !== "_no_example_"}
    <pre
      class="code cm-ftml-block-tip-example">
      <code>{@html Prism.highlight(docs.example, "ftml")}</code>
    </pre>
  {/if}

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

  .cm-ftml-block-tip-header {
    display: flex;
    align-content: center;
    justify-content: space-between;
  }

  .cm-ftml-block-tip-accepts {
    display: flex;
    gap: 0.25rem;
    align-items: center;
    color: var(--col-hint);
    cursor: pointer;
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

  .cm-ftml-block-tip-example {
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
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
