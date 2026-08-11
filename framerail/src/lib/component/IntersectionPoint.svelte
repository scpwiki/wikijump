<!--
  @component Component that fires callbacks when it comes in and out of view.
-->
<script lang="ts">
  import { onMount } from "svelte"

  /* global AnyFunction */
  let {
    onEnter = () => undefined,
    onExit = () => undefined,
    opts = {}
  }: {
    /** Function to call when the observer enters the viewport. */
    onEnter?: AnyFunction

    /** Function to call when the observer leaves the viewport. */
    onExit?: AnyFunction

    /** Options for the {@link IntersectionObserver}. */
    opts?: IntersectionObserverInit
  } = $props()

  let intersectionElement: HTMLElement

  function handler(entry: IntersectionObserverEntry) {
    const isVisible = Math.round(entry.intersectionRatio)
    if (isVisible) onEnter()
    else onExit()
  }

  // svelte-ignore state_referenced_locally
  const observer = new IntersectionObserver(([entry]) => {
    handler(entry)
  }, opts)

  onMount(() => {
    observer.observe(intersectionElement)
  })
</script>

<div bind:this={intersectionElement} class="intersection-point" role="presentation"></div>

<style global lang="scss">
  .intersection-point {
    width: 100%;
    height: 0;
  }
</style>
