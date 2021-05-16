<script lang="ts">
  import { EditorState, Compartment } from "@codemirror/state"
  import { EditorView } from "@codemirror/view"
  import { LanguageDescription } from "@codemirror/language"
  import { languages } from "@codemirror/language-data"
  import { onDestroy, onMount } from "svelte"
  import { createIdleQueued, createMutatingLock } from "wj-util"
  import { getCodeDisplayExtensions } from "sheaf-core"

  export let content: string | Promise<string>
  export let lang = ""

  let container: HTMLElement
  let view: EditorView

  const updateDocument = createIdleQueued((doc: string) => {
    if (view) {
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: doc } })
    }
  })

  const compartment = new Compartment()

  $: if (content && view) {
    getDoc(content).then(doc => {
      if (doc !== null) updateDocument(doc)
    })
  }

  $: if (lang && view) {
    getLang().then(lang => view.dispatch({ effects: compartment.reconfigure(lang!) }))
  }

  async function getLang() {
    let desc: LanguageDescription | null = null
    if (lang) desc = LanguageDescription.matchLanguageName(languages, lang, true)
    if (desc) return desc.support ?? (await desc.load())
  }

  const getDoc = createMutatingLock(async (input: typeof content) => {
    const result = await input
    return result
  })

  onMount(async () => {
    view = new EditorView({
      parent: container,
      state: EditorState.create({
        doc: await content,
        extensions: [
          ...getCodeDisplayExtensions(),
          compartment.of((await getLang()) ?? [])
        ]
      })
    })
  })

  onDestroy(() => {
    if (view) view.destroy()
  })
</script>

<div class="code-display" class:hidden={!view} bind:this={container} />

<style lang="scss">
  .code-display {
    height: 100%;
    opacity: 1;
    transition: opacity 0.1s ease-out;

    &.hidden {
      opacity: 0;
    }
  }
</style>
