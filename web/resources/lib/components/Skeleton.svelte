<script lang="ts">
  export let type: "block" | "inline" | "spinner" = "block"

  export let height = type === "inline" ? "1em" : "2rem"

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
  {:else if type === "spinner"}
    <div class="skeleton is-spinner" style="width: {width}; height: {height};">
      <svg
        class="skeleton-spinner"
        viewBox="-15 -15 30 30"
        height="75%"
        width="75%"
        xmlns="http://www.w3.org/2000/svg"
      >
        <circle r="13" class="skeleton-spinner-circle" fill="none" stroke-width="2" />
        <circle r="13" class="skeleton-spinner-arc" fill="none" stroke-width="2" />
      </svg>
    </div>
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

    &.is-spinner {
      display: block;
      border-radius: 1em;
    }

    &:hover,
    &:focus,
    &:active {
      border: none;
      outline: none;
    }

    &.is-block::before,
    &.is-line::before,
    &.is-spinner::before {
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

  .skeleton-spinner {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
  }

  .skeleton-spinner-circle {
    opacity: 0.25;
    stroke: var(--col-border);
  }

  .skeleton-spinner-arc {
    stroke: var(--col-hint);
    stroke-dasharray: 30, 90;
    stroke-dashoffset: 0;
    stroke-linecap: round;
  }

  @include tolerates-motion {
    .skeleton-spinner {
      animation: skeleton-spinner-rotate 1s linear infinite;
      will-change: transform;
    }

    .skeleton-spinner-arc {
      animation: skeleton-spinner-dash 3s ease-in-out alternate infinite;
      will-change: stroke-dasharray;
    }
  }

  @keyframes skeleton-spinner-rotate {
    0% {
      transform: translate(-50%, -50%) rotate(0deg);
    }
    100% {
      transform: translate(-50%, -50%) rotate(360deg);
    }
  }

  @keyframes skeleton-spinner-dash {
    0% {
      stroke-dasharray: 10, 90;
    }
    100% {
      stroke-dasharray: 50, 90;
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
