<!--
  @component Autocomplete tooltip for FTML blocks.
-->
<script lang="ts">
  import type { EditorSvelteComponentProps } from "@wikijump/codemirror"
  import { Icon, TippySingleton } from "@wikijump/components"
  import Locale from "@wikijump/fluent"
  import Prism from "@wikijump/prism"
  import type { BlockData } from "../data/block"

  export let block: BlockData
  export let unmount: EditorSvelteComponentProps["unmount"]

  const t = Locale.makeComponentFormatter("cmftml")

  // TODO: deprecated styling (need a deprecated block first?)

  let codeString =
    block.outputType === "html"
      ? block.outputClass
        ? `<${block.outputTag} class="${block.outputClass}">`
        : `<${block.outputTag}>`
      : `type: ${block.outputType}`

  // looks messy but this just assembles a reasonable looking block string
  let ftmlString = `[[${block.name}${
    block.head === "map"
      ? ' arg="value"'
      : block.head === "value"
      ? " value"
      : block.head === "value+map"
      ? ' value arg="value"'
      : ""
  }]]`

  Prism.highlight(codeString, block.outputType === "html" ? "html" : "log").then(
    code => (codeString = code)
  )

  Prism.highlight(ftmlString, "ftml").then(code => (ftmlString = code))
</script>

<div class="cm-ftml-block-tip">
  <div class="cm-ftml-block-tip-header">
    <h5 class="cm-ftml-block-tip-title">
      {block.docs?.title ?? block.name}
    </h5>
    <TippySingleton let:tip>
      <!-- In both types we won't display type "NONE" as that would just be confusing -->
      {#if block.head !== "none" || block.body !== "none"}
        <div class="cm-ftml-block-tip-type">
          {#if block.head === "map"}
            <span use:tip={$t("#-argument-map.info")}>
              {$t("#-argument-map")}
            </span>
          {:else if block.head === "value"}
            <span use:tip={$t("#-argument-value.info")}>
              {$t("#-argument-value")}
            </span>
          {:else if block.head === "value+map"}
            <span use:tip={$t("#-argument-value-map.info")}>
              {$t("#-argument-value-map")}
            </span>
          {/if}
          {#if block.body === "elements"}
            <span use:tip={$t("#-body-elements.info")}>
              {$t("#-body-elements")}
            </span>
          {:else if block.body === "raw"}
            <span use:tip={$t("#-body-raw.info")}>
              {$t("#-body-raw")}
            </span>
          {:else if block.body === "other"}
            <span use:tip={$t("#-body-other.info")}>
              {$t("#-body-other")}
            </span>
          {/if}
        </div>
      {/if}

      <!-- Don't display if there wouldn't be any icons -->
      {#if block.htmlAttributes || block.acceptsStar || block.acceptsScore || block.acceptsNewlines}
        <div class="cm-ftml-block-tip-accepts">
          {#if block.htmlAttributes}
            <span use:tip={$t("#-accepts.html-attributes")}>
              <Icon i="whh:html" size="1rem" />
            </span>
          {/if}
          {#if block.acceptsStar}
            <span use:tip={$t("#-accepts.star")}>
              <Icon i="fa-solid:asterisk" size="1rem" />
            </span>
          {/if}
          {#if block.acceptsScore}
            <span use:tip={$t("#-accepts.score")}>
              <Icon i="feather:underline" size="1rem" />
            </span>
          {/if}
          {#if block.acceptsNewlines}
            <span use:tip={$t("#-accepts.newlines")}>
              <Icon i="ic:round-keyboard-return" size="1rem" />
            </span>
          {/if}
        </div>
      {/if}
    </TippySingleton>
  </div>

  {#if block.aliasesRaw.length > 1 || block.excludeName}
    <div class="cm-ftml-block-tip-aliases">
      {#each block.aliasesRaw as alias}
        <span>'{alias}'</span>
      {/each}
    </div>
  {/if}

  <div class="wikitext cm-ftml-block-tip-info">
    {#if block.docs}
      {#await block.docs.info.render() then info}
        {@html info.html}
      {/await}
    {:else}
      <wj-body class="wj-body">
        <p>{$t("#-undocumented-block")}</p>
      </wj-body>
    {/if}
  </div>

  {#if block.docs}
    <pre class="code cm-ftml-block-tip-example"><code
        >{@html Prism.highlight(block.docs.example, "ftml")}</code
      ></pre>
  {/if}

  <hr />

  <div class="cm-ftml-block-tip-emit">
    <pre class="code cm-ftml-block-tip-emit-info"><code>{@html ftmlString}</code></pre>
    <pre class="code cm-ftml-block-tip-emit-info"><code>{@html codeString}</code></pre>
  </div>
</div>

<style global lang="scss">
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
