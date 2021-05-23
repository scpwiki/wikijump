<!--
  @component CodeMirror instance that displays code, like a `<code>` block.
-->
<script lang="ts">
  import { EditorState, Compartment } from "@codemirror/state"
  import { EditorView } from "@codemirror/view"
  import { LanguageDescription } from "@codemirror/language"
  import { languages } from "cm-lang-ftml"
  import { onDestroy, onMount } from "svelte"
  import { createIdleQueued, createMutatingLock } from "wj-util"
  import { getCodeDisplayExtensions } from "sheaf-core"
  import { Spinny } from "components"

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

  async function getLang() {
    let desc: LanguageDescription | null = null
    if (lang) desc = LanguageDescription.matchLanguageName(languages, lang, true)
    if (desc) return desc.support ?? (await desc.load())
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
          ...getCodeDisplayExtensions(),
          langExtension.of((await getLang()) ?? [])
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

<style lang="scss">
  @import "../../wj-css/src/abstracts";

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
