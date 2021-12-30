<!--
  @component Tooltip/menu for a misspelling, along with suggestions for said misspelling.
-->
<script lang="ts">
  import type { EditorSvelteComponentProps } from "@wikijump/codemirror"
  import { Spinny, TippySingleton } from "@wikijump/components"
  import { anim } from "@wikijump/components/lib"
  import { focusGroup } from "@wikijump/dom"
  import Locale from "@wikijump/fluent"
  import espells from "./espells"
  import type { FlaggedWord } from "./types"

  const t = Locale.makeComponentFormatter("spellcheck")

  export let word: FlaggedWord

  export let view: EditorSvelteComponentProps["view"]
  export let update: EditorSvelteComponentProps["update"]
  export let unmount: EditorSvelteComponentProps["unmount"]

  let suggestions: string[] | null = null

  if (!word.info.forbidden) {
    espells.suggest(word.word).then(result => (suggestions = result))
  }

  function applySuggestion(suggestion: string) {
    if (!view) return
    view.dispatch({
      changes: { from: word.from, to: word.to, insert: suggestion }
    })
  }

  function addToDictionary() {
    if (!view) return
    espells.addToLocalDictionary(word.word)
    // replace range anyways so that the view gets updated
    view.dispatch({
      changes: { from: word.from, to: word.to, insert: word.word }
    })
  }
</script>

<div class="cm-spellcheck-tip" use:focusGroup={"vertical"}>
  {#if word.info.forbidden}
    <h6 class="cm-spellcheck-tip-title">
      {$t("#-word.forbidden", { slice: word.word })}
    </h6>
    <!-- empty list just to preserve formatting -->
    <ul class="cm-spellcheck-tip-list" aria-hidden="true" />
  {:else if !word.info.correct || word.info.warn}
    {#if !word.info.correct}
      <h6 class="cm-spellcheck-tip-title">
        {$t("#-word.misspelled", { slice: word.word })}
      </h6>
    {:else}
      <h6 class="cm-spellcheck-tip-title">
        {$t("#-word.warned", { slice: word.word })}
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
                use:tip={$t("#-accept", { slice: word.word, suggestion })}
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
              use:tip={$t("#-add-word.tooltip", { slice: word.word })}
            >
              {$t("#-add-word", { slice: word.word })}
            </button>
          </li>
        {/if}
      </ul>
    </TippySingleton>
  {/if}

  <hr />

  <pre class="code cm-spellcheck-tip-source">
      <code>{$t("#-source", { slice: word.word, from: word.from, to: word.to })}</code>
  </pre>
</div>

<style global lang="scss">
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
