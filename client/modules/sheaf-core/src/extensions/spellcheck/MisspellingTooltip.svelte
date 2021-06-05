<!--
  @component Tooltip/menu for a misspelling, along with suggestions for said misspelling.
-->
<script lang="ts">
  import { focusGroup, TippySingleton } from "components"
  import { t } from "wj-state"
  import type { EditorSvelteComponentProps } from "../../adapters/svelte-dom"
  import { Spellchecker } from "./spellchecker"
  import type { Misspelling, Suggestion } from "./spellchecker/spellchecker"

  export let misspelling: Misspelling

  export let view: EditorSvelteComponentProps["view"]
  export let update: EditorSvelteComponentProps["update"]
  export let unmount: EditorSvelteComponentProps["unmount"]

  function applySuggestion(suggestion: Suggestion) {
    if (!view) return
    view.dispatch({
      changes: { from: misspelling.from, to: misspelling.to, insert: suggestion.term }
    })
  }

  function addToDictionary() {
    if (!view) return
    Spellchecker.saveToDictionary(misspelling.word)
    // replace range anyways so that the view gets updated
    view.dispatch({
      changes: { from: misspelling.from, to: misspelling.to, insert: misspelling.word }
    })
  }
</script>

<div class="cm-spellcheck-tip" use:focusGroup={"vertical"}>
  <h6 class="cm-spellcheck-tip-title">
    {$t("sheaf.spellcheck.MISSPELLED_WORD", { values: { slice: misspelling.word } })}
  </h6>

  <TippySingleton let:tip opts={{ placement: "right" }}>
    <ul class="cm-spellcheck-tip-list">
      {#each misspelling.suggestions as suggestion}
        <li>
          <button
            class="cm-spellcheck-tip-suggestion"
            type="button"
            on:click={() => applySuggestion(suggestion)}
            use:tip={$t("sheaf.spellcheck.tooltips.ACCEPT_SUGGESTION", {
              values: { slice: misspelling.word, suggestion: suggestion.term }
            })}
          >
            {suggestion.term}
          </button>
        </li>
      {/each}
      <li>
        <button
          class="cm-spellcheck-tip-suggestion cm-spellcheck-tip-suggestion-add"
          type="button"
          on:click={() => addToDictionary()}
          use:tip={$t("sheaf.spellcheck.tooltips.ADD_TO_DICTIONARY", {
            values: { slice: misspelling.word }
          })}
        >
          {$t("sheaf.spellcheck.ADD_TO_DICTIONARY", {
            values: { slice: misspelling.word }
          })}
        </button>
      </li>
    </ul>
  </TippySingleton>

  <hr />

  <pre
    class="code cm-spellcheck-tip-source">
      <code>{$t("sheaf.spellcheck.SOURCE", {
        values: { slice: misspelling.word, from: misspelling.from, to: misspelling.to }
      })}</code>
  </pre>
</div>

<style lang="scss">
  .cm-spellcheck-tip {
    padding: 0.25rem 0.5rem;
  }

  .cm-spellcheck-tip-list {
    padding-top: 0.25rem;
    padding-bottom: 0.25rem;
    list-style: none;

    > li:not(:last-child) {
      border-bottom: solid 0.05rem var(--colcode-border);
    }
  }

  .cm-spellcheck-tip-suggestion {
    display: block;
    width: 100%;
    padding: 0.125rem 0.25rem;
    padding-left: 0;
    font-size: 0.825rem;
    color: var(--col-text);
    text-align: left;
    background: none;
    border-radius: 0;
    transition: background-color 0.075s ease, color 0.075s ease;

    &:hover,
    &[aria-selected] {
      color: var(--colcode-accent);
      background-color: var(--colcode-hover);
    }
  }

  .cm-spellcheck-tip-suggestion-add {
    margin-top: 0.25rem;
    color: var(--colcode-accent);
  }

  .cm-spellcheck-tip-source {
    margin-top: 0.5rem;
    font-size: 0.675rem;
    opacity: 0.75;
  }
</style>
