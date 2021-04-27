<script lang="ts">
  import { onMount } from "svelte"

  export let onEnter: AnyFunction = () => undefined
  export let onExit: AnyFunction = () => undefined
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

<style>
  .intersection-point {
    width: 100%;
    height: 0;
  }
</style>
