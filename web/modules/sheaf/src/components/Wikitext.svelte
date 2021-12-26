<!--
  @component Generic wikitext container.
-->
<script lang="ts">
  import FTML from "@wikijump/ftml-wasm-worker"
  import morphdom from "morphdom"
  import { createAnimQueued, createMutatingLock, perfy, toFragment } from "@wikijump/util"
  import Card from "./Card.svelte"
  import { anim } from "@wikijump/components/lib"
  import Spinny from "./Spinny.svelte"
  import Locale, { unit } from "@wikijump/fluent"

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

  /** Prevents the rendering of elements which may cause a network request. */
  export let offline = false

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
    const displayIndicatorTimeout = setTimeout(() => (rendering = true), 100)
    const measure = perfy()
    if (typeof wikitext === "function") wikitext = wikitext()
    wikitext = await wikitext
    const result: Rendered = wikitext
      ? typeof wikitext !== "string"
        ? wikitext
        : await FTML.renderHTML(wikitext)
      : { html: "", styles: [""] }
    perfRender = measure()
    clearTimeout(displayIndicatorTimeout)
    return result
  })

  // TODO: Security audit of this - how much should we trust FTML output right now?

  const update = createAnimQueued(async ({ html, styles }: Rendered) => {
    if (!element) return

    // there are better ways to do this, but this is mostly just a development tool
    // this prevents the console from getting spammed with
    // crossorigin or missing link errors
    if (offline) {
      html = html.replaceAll(
        /<(img|iframe)[^]+?>/g,
        "<div>Offline Replacement Element</div>"
      )
    }

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

    // prepend style with a index comment so that each style string is unique
    stylesheets = styles.map(
      (style, idx) => `\n/* stylesheet ${idx + 1} */\n\n${style}\n`
    )
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

<div class="wikitext-container">
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

<style lang="scss">
  @import "../../../resources/css/abstracts";

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
