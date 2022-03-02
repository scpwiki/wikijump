<!--
  @component Generic wikitext container.
-->
<script lang="ts">
  import FTML, { type RenderedHTML } from "@wikijump/ftml-wasm-worker"
  import {
    animationFrame,
    createLock,
    createMutatingLock,
    idleCallback,
    perfy,
    toFragment
  } from "@wikijump/util"
  import * as micromorph from "micromorph"
  import type { SheafState } from "../state"

  type Wikitext = SheafState | string

  export let wikitext: Wikitext = ""

  export let rendering = false

  export let timeTotal = 0
  export let timeCompile = 0
  export let timePatch = 0

  let element: HTMLElement
  let stylesheets: string[] = []
  let measureTotal: (() => number) | null = null

  function stylesheet(node: HTMLStyleElement, style: string) {
    node.innerHTML = style
    return {
      update(style: string) {
        node.innerHTML = style
      },
      destroy() {
        // not sure why I have to destroy the node, but I do apparently
        node.parentElement?.removeChild(node)
      }
    }
  }

  const compile = createMutatingLock(async (wikitext: Wikitext) => {
    rendering = true
    const measureCompile = perfy()
    if (!measureTotal) measureTotal = perfy()

    const result =
      typeof wikitext === "string"
        ? await FTML.renderHTML(wikitext, undefined, "draft")
        : await wikitext.render()

    timeCompile = measureCompile()
    return result
  })

  const patch = createLock(async ({ html, styles }: RenderedHTML) => {
    rendering = true
    const measurePatch = perfy()
    if (!measureTotal) measureTotal = perfy()

    const fragment = await idleCallback(() => toFragment(html))

    await idleCallback(async () => {
      const oldBody = element.querySelector("wj-body")
      const newBody = fragment.querySelector("wj-body")

      if (!newBody || !oldBody) {
        element.innerText = ""
        element.appendChild(fragment)
        return
      }

      const diff = micromorph.diff(oldBody, newBody)
      if (diff) await animationFrame(() => micromorph.patch(oldBody, diff))
    })

    await animationFrame(() => {
      // prepend style with a index comment so that each style string is unique
      stylesheets = styles.map(
        (style, idx) => `\n/* stylesheet ${idx + 1} */\n\n${style}\n`
      )
    })

    timePatch = measurePatch()
    if (measureTotal) timeTotal = measureTotal()
    measureTotal = null
    rendering = false
  })

  $: if (element) {
    compile(wikitext).then(result => {
      if (result) patch(result)
    })
  }
</script>

<svelte:head>
  {#each stylesheets as style (style)}
    <style use:stylesheet={style}></style>
  {/each}
</svelte:head>

<div bind:this={element} class="wikitext-body wikitext" />
