<!--
  @component Generic wikitext container.
-->
<script lang="ts">
  import { Card, Spinny } from "@wikijump/components"
  import { anim } from "@wikijump/components/lib"
  import Locale, { unit } from "@wikijump/fluent"
  import FTML from "@wikijump/ftml-wasm-worker"
  import {
    animationFrame,
    createMutatingLock,
    idleCallback,
    perfy,
    toFragment
  } from "@wikijump/util"
  import micromorph from "micromorph"

  const t = Locale.makeComponentFormatter("wikitext")

  type Rendered = { html: string; styles: string[] }
  type WikitextInput =
    | Promisable<string>
    | Promisable<Rendered>
    | (() => Promisable<string>)
    | (() => Promisable<Rendered>)

  /**
   * Wikitext input. May be given in quite a few ways:
   *
   * 1. As a string.
   * 2. As an object, `{ html: string, style: string }`.
   * 3. As a function returning either a string or the aformentioned object.
   * 4. As a `Promise` resolving to either a string or the aforementioned object.
   * 5. As a function returning a `Promise` like the aformentioned one.
   *
   * Providing the input as a function has performance benefits. If the
   * wikitext takes a long time to render, and the source wikitext is
   * frequently updating, providing a function will make it so that the
   * wikitext is only evalulated when it needs to be. Otherwise, the
   * wikitext would be evalulated constantly, and that has obvious
   * performance implications.
   */
  export let wikitext: WikitextInput = ""

  /**
   * Flags the component to morph the output container rather than entirely
   * replacing the contents.
   *
   * This is for performance reasons. If you are frequently updating the
   * wikitext, it's probably best if the DOM is morphed rather than replaced.
   */
  export let morph = false

  /** Shows render performance information if true. */
  export let debug = false

  /**
   * If true, the wikitext container will fill its parent and have the
   * rendered wikitext scroll inside of it.
   */
  export let contain = false

  let element: HTMLElement
  let stylesheets: string[] = []
  let rendering = false
  let perfRender = 0

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

  const render = createMutatingLock(async (wikitext: WikitextInput) => {
    const displayIndicatorTimeout = setTimeout(() => (rendering = true), 500)
    const measure = perfy()

    if (typeof wikitext === "function") wikitext = wikitext()
    wikitext = await wikitext

    return await idleCallback(async () => {
      const result: Rendered = wikitext
        ? typeof wikitext !== "string"
          ? wikitext
          : await FTML.renderHTML(wikitext)
        : { html: "", styles: [""] }
      perfRender = measure()
      clearTimeout(displayIndicatorTimeout)
      return result
    })
  })

  const update = createMutatingLock(async ({ html, styles }: Rendered) => {
    if (!element) return

    const fragment = await idleCallback(() => toFragment(html))

    await animationFrame(() => {
      if (morph) {
        const oldBody = element.querySelector("wj-body")
        const newBody = fragment.querySelector("wj-body")

        if (!newBody || !oldBody) {
          element.innerText = ""
          element.appendChild(fragment)
          return
        }

        micromorph(oldBody, newBody)
      } else {
        element.innerText = ""
        element.appendChild(fragment)
      }
    })

    await animationFrame(() => {
      // prepend style with a index comment so that each style string is unique
      stylesheets = styles.map(
        (style, idx) => `\n/* stylesheet ${idx + 1} */\n\n${style}\n`
      )
    })

    rendering = false
  })

  $: if (wikitext) {
    render(wikitext).then(result => {
      if (result) update(result)
    })
  }
</script>

<svelte:head>
  {#each stylesheets as style (style)}
    <style use:stylesheet={style}></style>
  {/each}
</svelte:head>

<div class="wikitext-container" class:is-contained={contain}>
  {#if rendering}
    <div
      class="wikitext-loading-panel"
      transition:anim={{ duration: 250, css: t => `opacity: ${t}` }}
    >
      <Spinny inline size="1.25rem" description={$t("#-rendering")} />
    </div>
  {/if}
  {#if debug}
    <div class="wikitext-perf-panel">
      <Card title={$t("#-perf.title")} theme="dark" width="12rem">
        <div>
          <strong>{$t("#-perf.render")}</strong>
          <code>{unit(perfRender, "millisecond", { unitDisplay: "narrow" })}</code>
        </div>
      </Card>
    </div>
  {/if}
  <div bind:this={element} class="wikitext-body wikitext" />
</div>

<style global lang="scss">
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
    z-index: $z-above;
  }

  .wikitext-container.is-contained {
    contain: strict;
    height: 100%;

    .wikitext-body {
      max-height: 100%;
      padding: 0 1rem;
      padding-bottom: 10rem;
      overflow-x: hidden;
      overflow-y: auto;
    }
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
