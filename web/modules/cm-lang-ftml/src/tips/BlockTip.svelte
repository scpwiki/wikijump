<!--
  @component Autocomplete tooltip for FTML blocks.
-->
<script lang="ts">
  import type { Block } from "../data/types"
  import type { EditorSvelteComponentProps } from "@wikijump/codemirror"
  import * as Prism from "@wikijump/prism"
  import { t } from "@wikijump/api"
  import type { FTMLFragment } from "@wikijump/ftml-wasm-worker"
  import { Icon, TippySingleton } from "@wikijump/components"
  import { aliasesRaw } from "../util"

  interface Docs {
    title: string
    info: FTMLFragment
    example: string
  }

  export let name: string
  export let block: Block
  export let docs: Docs
  export let unmount: EditorSvelteComponentProps["unmount"]

  // TODO: deprecated styling (need a deprecated block first?)

  const aliases = aliasesRaw([name, block])
  const deprecated = block["deprecated"] ?? false
  const acceptsStar = block["accepts-star"] ?? false
  const acceptsScore = block["accepts-score"] ?? false
  const acceptsNewlines = block["accepts-newlines"] ?? false
  const usesHTMLAttributes = block["html-attributes"] ?? false
  const [outputType, outputTag, outputClass] = block["html-output"].split(",")

  let codeString =
    outputType === "html"
      ? outputClass
        ? `<${outputTag} class="${outputClass}">`
        : `<${outputTag}>`
      : `type: ${outputType}`

  codeString = Prism.highlight(codeString, outputType === "html" ? "html" : "log")

  // looks messy but this just assembles a reasonable looking block string
  let ftmlString = `[[${name}${
    block.head === "map"
      ? ' arg="value"'
      : block.head === "value"
      ? " value"
      : block.head === "value+map"
      ? ' value arg="value"'
      : ""
  }]]`

  ftmlString = Prism.highlight(ftmlString, "ftml")
</script>

<div class="cm-ftml-block-tip">
  <div class="cm-ftml-block-tip-header">
    <h5 class="cm-ftml-block-tip-title">
      {docs.title}
    </h5>
    <TippySingleton let:tip>
      <!-- In both types we won't display type "NONE" as that would just be confusing -->
      {#if block.head !== "none" || block.body !== "none"}
        <div class="cm-ftml-block-tip-type">
          {#if block.head === "map"}
            <span use:tip={$t("cmftml.blocks.argument_types.map.INFO")}>
              {$t("cmftml.blocks.argument_types.map.TITLE")}
            </span>
          {:else if block.head === "value"}
            <span use:tip={$t("cmftml.blocks.argument_types.value.INFO")}>
              {$t("cmftml.blocks.argument_types.value.TITLE")}
            </span>
          {:else if block.head === "value+map"}
            <span use:tip={$t("cmftml.blocks.argument_types.value_map.INFO")}>
              {$t("cmftml.blocks.argument_types.value_map.TITLE")}
            </span>
          {/if}
          {#if block.body === "elements"}
            <span use:tip={$t("cmftml.blocks.body_types.elements.INFO")}>
              {$t("cmftml.blocks.body_types.elements.TITLE")}
            </span>
          {:else if block.body === "raw"}
            <span use:tip={$t("cmftml.blocks.body_types.raw.INFO")}>
              {$t("cmftml.blocks.body_types.raw.TITLE")}
            </span>
          {:else if block.body === "other"}
            <span use:tip={$t("cmftml.blocks.body_types.other.INFO")}>
              {$t("cmftml.blocks.body_types.other.TITLE")}
            </span>
          {/if}
        </div>
      {/if}

      <!-- Don't display if there wouldn't be any icons -->
      {#if usesHTMLAttributes || acceptsStar || acceptsScore || acceptsNewlines}
        <div class="cm-ftml-block-tip-accepts">
          {#if usesHTMLAttributes}
            <span use:tip={$t("cmftml.blocks.HTML_ATTRIBUTES")}>
              <Icon i="whh:html" size="1rem" />
            </span>
          {/if}
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
      {/if}
    </TippySingleton>
  </div>

  {#if aliases.length > 1 || block["exclude-name"]}
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

  <div class="cm-ftml-block-tip-emit">
    <pre
      class="code cm-ftml-block-tip-emit-info">
    <code>{@html ftmlString}</code>
  </pre>

    <pre
      class="code cm-ftml-block-tip-emit-info">
    <code>{@html codeString}</code>
  </pre>
  </div>
</div>

<style lang="scss">
  .cm-ftml-block-tip {
    color: var(--col-text);
    background: none;
  }

  .cm-ftml-block-tip-header {
    display: flex;
    align-content: center;
  }

  .cm-ftml-block-tip-title {
    flex-grow: 1;
    color: var(--colcode-tag);
  }

  .cm-ftml-block-tip-type {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    font-size: 0.75rem;
    font-weight: bold;
    color: var(--col-hint);
    cursor: pointer;
  }

  .cm-ftml-block-tip-accepts {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding-right: 0.25rem;
    padding-left: 0.75rem;
    margin-left: 0.75rem;
    line-height: 0;
    color: var(--col-hint);
    cursor: pointer;
  }

  // must have both for the border to show
  .cm-ftml-block-tip-type + .cm-ftml-block-tip-accepts {
    border-left: solid 0.075rem var(--col-border);
  }

  .cm-ftml-block-tip-aliases {
    margin-top: 0.25rem;
    font-family: var(--font-mono);
    font-size: 0.75em;
    > span {
      margin-right: 0.5em;
      color: var(--colcode-string);
    }
  }

  .cm-ftml-block-tip-info {
    margin-top: 0.25rem;
    margin-bottom: 0.25rem;
  }

  .cm-ftml-block-tip-example {
    margin-top: 0.5rem;
    margin-bottom: 0.5rem;
    white-space: pre-wrap;
  }

  .cm-ftml-block-tip-emit {
    display: flex;
    margin-top: 0.5rem;

    > .cm-ftml-block-tip-emit-info:first-child {
      padding-right: 0.5rem;
      margin-right: 0.5rem;
      border-right: solid 0.075rem var(--col-border);
    }
  }

  .cm-ftml-block-tip-emit-info {
    display: block;
    font-size: 0.75em;
    color: var(--colcode-content);

    > code {
      overflow-x: visible;
    }
  }
</style>
