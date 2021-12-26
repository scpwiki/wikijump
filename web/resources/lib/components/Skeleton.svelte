<script lang="ts">
  export let type: "block" | "inline" = "block"

  export let height = type === "block" ? "auto" : "1em"

  export let width = "100%"

  export let lines = 1

  export let theme: "auto" | "dark" | "light" = "auto"
</script>

<div
  class="skeleton-container {theme !== 'auto' ? theme : ''}"
  class:is-block={type === "block"}
  class:is-inline={type === "inline"}
>
  {#if type === "block"}
    <div class="skeleton is-block" style="width: {width}; height: {height};" />
  {:else if type === "inline"}
    {#each Array(lines).fill(0) as _}
      <div class="skeleton is-line" style="width: {width}; height: {height};" />
    {/each}
  {/if}
</div>

<style lang="scss">
  @import "../../../resources/css/abstracts";

  .skeleton-container {
    position: relative;
    width: 100%;
    cursor: progress;

    &.is-block {
      display: block;
    }

    &.is-inline {
      display: inline-block;
    }
  }

  .skeleton {
    position: relative;
    overflow: hidden;
    background: var(--col-background-dim);

    &.is-block {
      display: block;
      border-radius: 0.5em;
    }

    &.is-line {
      display: inline-block;
      width: 100%;
      border-radius: 0.25em;
    }

    &:hover,
    &:focus,
    &:active {
      border: none;
      outline: none;
    }

    &.is-block::before,
    &.is-line::before {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      content: "";
      background: var(--col-border);

      @include tolerates-motion {
        will-change: transform;
        animation: 2000ms cubic-bezier(0.645, 0.045, 0.355, 1) 0s infinite skeleton-wave;
      }
    }
  }

  @keyframes skeleton-wave {
    0% {
      transform: translateX(-100%);
    }
    100% {
      transform: translateX(calc(100%));
    }
  }
</style>
