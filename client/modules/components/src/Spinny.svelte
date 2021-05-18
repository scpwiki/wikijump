<!--
  @component Versatile inline or overlay based spinner.
-->
<script lang="ts">
  import { createID, sleep } from "wj-util"
  import Icon from "./Icon.svelte"
  import { anim } from "./lib/animation"

  type Status = "active" | "success" | "warning" | "error"

  /** Sets the styling so that the spinner can be displayed inline along with text. */
  export let inline = false
  /** CSS `top` offset. */
  export let top = inline ? "0" : "50%"
  /** CSS `left` offset. */
  export let left = inline ? "0" : "50%"
  /** Sets the width of non-inline spinners, otherwise sets font-size. */
  export let size = inline ? "1em" : "120px"
  /** Sets a delay until the spinner appears. Only works for non-inline spinners. */
  export let wait = 300
  /** Sets the icon for inline spinners. */
  export let status: Status = "active"
  /**
   * Sets the component ARIA attributes so that screen readers will
   * announce the status of the spinner.
   */
  export let loud = false
  /** Sets the text, if any, to be displayed alongside the spinner. */
  export let description = ""

  // TODO: replace hardcoded-english when translation system is available
  let label = ""
  // prettier-ignore
  $: switch (status) {
    case "active":  label = "Active loading indicator" ; break
    case "error":   label = "Error indicator"          ; break
    case "success": label = "Success indicator"        ; break
    case "warning": label = "Warning indicator"        ; break
  }

  let icon = ""
  // prettier-ignore
  $: switch (status) {
    case "active":  icon = ""                              ; break
    case "error":   icon = "fa-solid:exclamation-circle"   ; break
    case "success": icon = "fa-solid:check"                ; break
    case "warning": icon = "fa-solid:exclamation-triangle" ; break
  }

  const id = createID("spinny-label")

  const cssText = inline
    ? `top: ${top}; left: ${left}; height: ${size}; width: ${size};`
    : `top: ${top}; left: ${left}; width: ${size}`
</script>

<div
  class="spinny is-status-{status}"
  class:is-inline={inline}
  aria-atomic="true"
  aria-labelledby={id}
  aria-live={status === "active" && loud ? "assertive" : "off"}
  style="top: {top}; left: {left}; font-size: {inline ? size : '1rem'};"
  {...$$restProps}
>
  <!-- A11y label - is invisible to everything but screen readers -->
  <!-- svelte-ignore a11y-label-has-associated-control -->
  <label class="spinny-label" {id}>{label}</label>

  <div class="spinny-symbol" aria-hidden="true">
    {#if status === "active"}
      {#if !inline}
        {#await sleep(wait) then _}
          <svg
            class="spinny-spinner"
            transition:anim={{
              duration: 500,
              css: t => `transform: scale(${t}); opacity: ${t ** 2};`
            }}
            viewBox="0 -50 120 70"
            xmlns="http://www.w3.org/2000/svg"
            style={cssText}
          >
            <g>
              <circle cx="15" cy="-15" r="15" class="spinny-circle1" />
              <circle cx="60" cy="-15" r="15" class="spinny-circle2" />
              <circle cx="105" cy="-15" r="15" class="spinny-circle3" />
            </g>
          </svg>
        {/await}
      {:else}
        <svg
          class="spinny-spinner"
          viewBox="-3 -3 42 42"
          xmlns="http://www.w3.org/2000/svg"
          style={cssText}
        >
          <g fill="none" fill-rule="evenodd" stroke-width="6">
            <circle class="spinny-inline-background" cx="18" cy="18" r="18" />
            <path class="spinny-inline-arc" d="M36,18 A 18 18 0 0 1 0,18" />
          </g>
        </svg>
      {/if}
    {:else}
      <Icon i={icon} {size} />
    {/if}
  </div>

  {#if description}
    <span class="spinny-description">{description}</span>
  {/if}
</div>

<style lang="scss">
  @import "../../wj-css/src/abstracts";

  .spinny {
    position: absolute;
    z-index: 99;
    display: block;
    pointer-events: none;
    transform: translate(-50%, -50%);

    &.is-inline {
      position: relative;
      display: inline-flex;
      gap: 0.75ch;
      align-items: center;
      text-align: center;
      vertical-align: middle;
      transform: none;
    }

    .spinny-symbol {
      display: inline-flex;
      fill: var(--col-text-subtle);
    }

    &.is-status-error .spinny-symbol {
      fill: var(--col-danger);
    }

    &.is-status-success .spinny-symbol {
      fill: var(--col-success);
    }

    &.is-status-warning .spinny-symbol {
      fill: var(--col-warning);
    }
  }

  .spinny-description {
    font-size: 0.75em;
  }

  .spinny-label {
    @include hide-visually;
  }

  .spinny-inline-background {
    opacity: 0.5;
    stroke: var(--col-hint);
  }

  .spinny-inline-arc {
    stroke: var(--col-hint);
  }

  @include reduced-motion {
    // Don't show large spinners at all with reduced motion
    .spinny:not(.is-inline) .spinny-spinner {
      display: none;
    }
    // Solid, fixed circle with reduced motion
    .spinny-inline-background {
      opacity: 1;
    }
    .spinny-inline-arc {
      display: none;
    }
  }

  @include tolerates-motion {
    .spinny-circle1 {
      animation: spinny-wave 0.3s -0.9s infinite alternate ease-in-out;
    }
    .spinny-circle2 {
      animation: spinny-wave 0.3s -0.8s infinite alternate ease-in-out;
    }
    .spinny-circle3 {
      animation: spinny-wave 0.3s -0.7s infinite alternate ease-in-out;
    }
    .spinny-inline-arc {
      transform-origin: top;
      transform-box: fill-box;
      animation: spinny-inline-spin 0.75s 0s infinite linear;
    }
  }

  @keyframes spinny-inline-spin {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(360deg);
    }
  }

  @keyframes spinny-wave {
    0% {
      transform: translateY(15px);
    }
    100% {
      transform: translateY(-15px);
    }
  }
</style>
