<!--
  @component Autocomplete tooltip for FTML modules.
-->
<script lang="ts">
  import type { Module } from "../data/types"
  import type { EditorSvelteComponentProps } from "wj-codemirror"

  export let name: string
  export let module: Module
  export let info = ""
  export let unmount: EditorSvelteComponentProps["unmount"]

  const aliases = [name, ...(module.aliases ?? [])]
</script>

<div class="cm-ftml-module-tip">
  <h4 class="cm-ftml-module-tip-name">
    {name}
    <span>module</span>
  </h4>

  {#if aliases.length > 1}
    <div class="cm-ftml-module-tip-aliases">
      {#each aliases as alias}
        <span>'{alias}'</span>
      {/each}
    </div>
  {/if}

  <div class="wikitext cm-ftml-module-tip-info">
    {#if !info}
      <p><i>This module has no documentation.</i></p>
    {:else}
      {@html info}
    {/if}
  </div>
</div>

<style lang="scss">
  .cm-ftml-module-tip {
    color: var(--col-text);
    background: none;
  }

  .cm-ftml-module-tip-name > span {
    font-size: 0.75em;
    color: var(--col-text-dim);
  }

  .cm-ftml-module-tip-aliases {
    font-family: var(--font-mono);
    font-size: 0.75em;
    > span {
      margin-right: 0.5em;
      color: var(--colcode-string);
    }
  }

  .cm-ftml-module-tip-info {
    margin-top: 0.5rem;
    margin-bottom: 0.25rem;
  }
</style>
