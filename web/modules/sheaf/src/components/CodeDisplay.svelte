<!--
  @component CodeMirror instance that displays code, like a `<code>` block.
-->
<script lang="ts">
  import { ftmlLanguages } from "@wikijump/cm-lang-ftml"
  import { defaultLanguages, IndentHack, languageList } from "@wikijump/codemirror"
  import { Compartment, EditorState, type Extension } from "@codemirror/state"
  import { drawSelection, EditorView } from "@codemirror/view"
  import { LanguageDescription, syntaxHighlighting } from "@codemirror/language"
  import { Spinny } from "@wikijump/components"
  import { createIdleQueued, createMutatingLock } from "@wikijump/util"
  import { onDestroy, onMount } from "svelte"
  import { confinement } from "../extensions/theme"

  /** Contents of the code block. Can be a promise that resolves to a string. */
  export let content: Promisable<string>

  /** Name of the language to syntax highlight with. */
  export let lang = ""

  let element: HTMLElement
  let view: EditorView

  const langExtension = new Compartment()

  const updateDocument = createIdleQueued((doc: string) => {
    if (view) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: doc } })
    }
  })

  async function getLang(): Promise<Extension> {
    if (!view) return []
    const langs = view.state.facet(languageList)
    return (await LanguageDescription.matchLanguageName(langs, lang, true)?.load()) ?? []
  }

  const getDoc = createMutatingLock(async (input: typeof content) => {
    const result = await input
    return result
  })

  $: if (content && view) {
    getDoc(content).then(doc => {
      if (doc !== null) updateDocument(doc)
    })
  }

  $: if (lang && view) {
    getLang().then(lang => view.dispatch({ effects: langExtension.reconfigure(lang!) }))
  }

  onMount(async () => {
    view = new EditorView({
      parent: element,
      state: EditorState.create({
        doc: await content,
        extensions: [
          drawSelection(),
          EditorView.editable.of(false),
          EditorView.lineWrapping,
          IndentHack,
          syntaxHighlighting(confinement),
          defaultLanguages,
          ftmlLanguages,
          langExtension.of([])
        ]
      })
    })
  })

  onDestroy(() => {
    if (view) view.destroy()
  })
</script>

<div class="code-display-container">
  <div class="code-display" bind:this={element} class:hidden={!view} />
  {#if !view}<Spinny />{/if}
</div>

<style global lang="scss">
  .code-display-container {
    position: relative;
    height: 100%;
  }

  .code-display {
    height: 100%;
    opacity: 1;

    @include tolerates-motion {
      transition: opacity 0.125s ease-out;

      &.hidden {
        opacity: 0;
      }
    }
  }
</style>
