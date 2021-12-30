<!--
  @component Autocomplete tooltip for FTML modules.
-->
<script lang="ts">
  import type { EditorSvelteComponentProps } from "@wikijump/codemirror"
  import type { ModuleData } from "../data/block"

  export let module: ModuleData
  export let unmount: EditorSvelteComponentProps["unmount"]
</script>

<div class="cm-ftml-module-tip">
  <h4 class="cm-ftml-module-tip-name">
    {module.name}
    <span>module</span>
  </h4>

  {#if module.aliasesRaw.length > 1}
    <div class="cm-ftml-module-tip-aliases">
      {#each module.aliasesRaw as alias}
        <span>'{alias}'</span>
      {/each}
    </div>
  {/if}

  <div class="wikitext cm-ftml-module-tip-info" />
</div>

<style global lang="scss">
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
