<!--
  @component Component that fires callbacks when it comes in and out of view.
-->
<script lang="ts">
  import { onMount } from "svelte"

  /** Function to call when the observer enters the viewport. */
  export let onEnter: AnyFunction = () => undefined

  /** Function to call when the observer leaves the viewport. */
  export let onExit: AnyFunction = () => undefined

  /** Options for the {@link IntersectionObserver}. */
  export let opts: IntersectionObserverInit = {}

  let intersectionElement: HTMLElement

  function handler(entry: IntersectionObserverEntry) {
    const isVisible = Math.round(entry.intersectionRatio)
    if (isVisible) onEnter()
    else onExit()
  }

  const observer = new IntersectionObserver(([entry]) => {
    handler(entry)
  }, opts)

  onMount(() => {
    observer.observe(intersectionElement)
  })
</script>

<div class="intersection-point" bind:this={intersectionElement} role="presentation" />

<style global lang="scss">
  .intersection-point {
    width: 100%;
    height: 0;
  }
</style>
