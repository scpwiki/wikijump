<!--
  @component Generic wikitext container.
-->
<script lang="ts">
  import * as FTML from "ftml-wasm-worker"
  import { createAnimQueued, createMutatingLock, perfy, toFragment } from "wj-util"
  import Card from "./Card.svelte"
  import morphdom from "morphdom"
  import Spinny from "./Spinny.svelte"
  import { anim } from "./lib/animation"

  type Rendered = { html: string; styles: string[] }
  type WikitextInput =
    | Promisable<string>
    | Promisable<Rendered>
    | (() => Promisable<string>)
    | (() => Promisable<Rendered>)

  /**
   * Wikitext input. May be given in quite a few ways:
   * 1. As a string.
   * 2. As an object, `{ html: string, style: string }`.
   * 3. As a function returning either a string or the aformentioned object.
   * 4. As a `Promise` resolving to either a string or the aforementioned object.
   * 5. As a function returning a `Promise` like the aformentioned one.
   *
   * Providing the input as a function has performance benefits. If the wikitext
   * takes a long time to render, and the source wikitext is frequently updating,
   * providing a function will make it so that the wikitext is only evalulated when
   * it needs to be. Otherwise, the wikitext would be evalulated constantly, and that
   * has obvious performance implications.
   */
  export let wikitext: WikitextInput = ""

  /**
   * Flags the component to morph the output container
   * rather than entirely replacing the contents.
   *
   * This is for performance reasons. If you are frequently updating the wikitext,
   * it's probably best if the DOM is morphed rather than replaced.
   */
  export let morph = false

  let element: HTMLElement
  let stylesheet: HTMLStyleElement

  let rendering = false

  let perfRender = 0

  const render = createMutatingLock(async (wikitext: WikitextInput) => {
    const displayIndicatorTimeout = setTimeout(() => (rendering = true), 100)
    const measure = perfy()
    if (typeof wikitext === "function") wikitext = wikitext()
    wikitext = await wikitext
    const result: Rendered = wikitext
      ? typeof wikitext !== "string"
        ? wikitext
        : await FTML.render(wikitext)
      : { html: "", styles: [""] }
    perfRender = measure()
    clearTimeout(displayIndicatorTimeout)
    return result
  })

  // TODO: Security audit of this - how much should we trust FTML output right now?

  const update = createAnimQueued(async ({ html, styles }: Rendered) => {
    if (!element || !stylesheet) return
    const fragment = toFragment(html)
    if (morph) {
      morphdom(element, fragment, {
        childrenOnly: true,
        onBeforeElUpdated: function (fromEl, toEl) {
          if (fromEl.isEqualNode(toEl)) return false
          return true
        }
      })
    } else {
      element.innerText = ""
      element.append(fragment)
    }

    stylesheet.innerHTML = styles.join("\n")
    rendering = false
  })

  $: if (wikitext) {
    render(wikitext).then(result => {
      if (result) update(result)
    })
  }
</script>

<svelte:head>
  <style bind:this={stylesheet}></style>
</svelte:head>

<div class="wikitext-container">
  {#if rendering}
    <div
      class="wikitext-loading-panel"
      transition:anim={{ duration: 250, css: t => `opacity: ${t}` }}
    >
      <Spinny inline size="1.25rem" description="Rendering..." />
    </div>
  {/if}
  <div class="wikitext-perf-panel">
    <Card title="Performance" theme="dark" width="12rem">
      <div><strong>RENDER:</strong> <code>{perfRender}ms</code></div>
    </Card>
  </div>
  <div bind:this={element} class="wikitext-body wikitext" />
</div>

<style lang="scss">
  @import "../../wj-css/src/abstracts";

  .wikitext-loading-panel {
    position: absolute;
    top: 1rem;
    left: 1rem;
    padding: 0.25rem 0.5rem;
    color: var(--col-con-text);
    background: var(--col-con-background);
    border: solid 0.125rem var(--col-con-border);
    border-radius: 0.5rem;
    @include shadow(6);
  }

  .wikitext-perf-panel {
    position: absolute;
    top: 1rem;
    right: 1rem;
  }

  @include tolerates-motion {
    // Makes an empty container fade-in when it finally renders content
    .wikitext-body {
      opacity: 1;
      transition: opacity 0.125s ease-out;
      &:empty {
        opacity: 0;
      }
    }
  }
</style>
