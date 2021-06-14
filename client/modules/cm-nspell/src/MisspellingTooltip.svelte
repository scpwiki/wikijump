<!--
  @component Tooltip/menu for a misspelling, along with suggestions for said misspelling.
-->
<script lang="ts">
  import { focusGroup, TippySingleton, Spinny, anim } from "wj-components"
  import { t } from "wj-state"
  import nspell from "./nspell"
  import type { EditorSvelteComponentProps } from "wj-codemirror"
  import type { FlaggedWord } from "./types"

  export let word: FlaggedWord

  export let view: EditorSvelteComponentProps["view"]
  export let update: EditorSvelteComponentProps["update"]
  export let unmount: EditorSvelteComponentProps["unmount"]

  let suggestions: string[] | null = null

  if (!word.info.forbidden) {
    nspell.suggest(word.word).then(result => (suggestions = result))
  }

  function applySuggestion(suggestion: string) {
    if (!view) return
    view.dispatch({
      changes: { from: word.from, to: word.to, insert: suggestion }
    })
  }

  function addToDictionary() {
    if (!view) return
    nspell.saveToDictionary(word.word)
    // replace range anyways so that the view gets updated
    view.dispatch({
      changes: { from: word.from, to: word.to, insert: word.word }
    })
  }
</script>

<div class="cm-spellcheck-tip" use:focusGroup={"vertical"}>
  {#if word.info.forbidden}
    <h6 class="cm-spellcheck-tip-title">
      {$t("sheaf.spellcheck.FORBIDDEN_WORD", { values: { slice: word.word } })}
    </h6>
    <!-- empty list just to preserve formatting -->
    <ul class="cm-spellcheck-tip-list" aria-hidden="true" />
  {:else if !word.info.correct || word.info.warn}
    {#if !word.info.correct}
      <h6 class="cm-spellcheck-tip-title">
        {$t("sheaf.spellcheck.MISSPELLED_WORD", { values: { slice: word.word } })}
      </h6>
    {:else}
      <h6 class="cm-spellcheck-tip-title">
        {$t("sheaf.spellcheck.WARNED_WORD", { values: { slice: word.word } })}
      </h6>
    {/if}

    {#if !suggestions}
      <div
        class="cm-spellcheck-tip-loading"
        transition:anim={{ duration: 250, css: t => `opacity: ${t}` }}
      >
        <Spinny inline />
      </div>
    {/if}

    <TippySingleton let:tip opts={{ placement: "right" }}>
      <ul class="cm-spellcheck-tip-list">
        {#if suggestions}
          {#each suggestions as suggestion}
            <li>
              <button
                class="cm-spellcheck-tip-suggestion"
                type="button"
                on:click={() => applySuggestion(suggestion)}
                use:tip={$t("sheaf.spellcheck.tooltips.ACCEPT_SUGGESTION", {
                  values: { slice: word.word, suggestion }
                })}
              >
                {suggestion}
              </button>
            </li>
          {/each}
        {/if}
        {#if !word.info.correct}
          <li>
            <button
              class="cm-spellcheck-tip-suggestion cm-spellcheck-tip-suggestion-add"
              type="button"
              on:click={() => addToDictionary()}
              use:tip={$t("sheaf.spellcheck.tooltips.ADD_TO_DICTIONARY", {
                values: { slice: word.word }
              })}
            >
              {$t("sheaf.spellcheck.ADD_TO_DICTIONARY", {
                values: { slice: word.word }
              })}
            </button>
          </li>
        {/if}
      </ul>
    </TippySingleton>
  {/if}

  <hr />

  <pre
    class="code cm-spellcheck-tip-source">
      <code>{$t("sheaf.spellcheck.SOURCE", {
        values: { slice: word.word, from: word.from, to: word.to }
      })}</code>
  </pre>
</div>

<style lang="scss">
  .cm-spellcheck-tip {
    padding: 0.25rem 0.5rem;
  }

  .cm-spellcheck-tip-loading {
    position: absolute;
    top: 0.5rem;
    right: 1rem;
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
