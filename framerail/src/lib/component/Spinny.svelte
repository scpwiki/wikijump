<!--
  @component Versatile inline or overlay based spinner.
-->
<script lang="ts">
  import { format as t } from "$lib/util/locale"
  import { createID, sleep } from "$lib/util"
  import Icon from "./Icon.svelte"
  import { anim } from "./scripts/animation"

  let {
    inline = false,
    top = inline ? "0" : "50%",
    left = inline ? "0" : "50%",
    size = inline ? "1em" : "120px",
    wait = 300,
    status = $bindable("active"),
    loud = false,
    description = "",
    ...others
  }: {
    /**
     * Sets the styling so that the spinner can be displayed inline along
     * with text.
     */
    inline?: boolean

    /** CSS `top` offset. */
    top?: string

    /** CSS `left` offset. */
    left?: string

    /** Sets the width of non-inline spinners, otherwise sets font-size. */
    size?: string

    /**
     * Sets a delay until the spinner appears. Only works for non-inline
     * spinners.
     */
    wait?: number

    /** Sets the icon for inline spinners. */
    status?: "active" | "success" | "warning" | "error"

    /**
     * Sets the component ARIA attributes so that screen readers will
     * announce the status of the spinner.
     */
    loud?: boolean

    /** Sets the text, if any, to be displayed alongside the spinner. */
    description?: string

    [key: string]: any
  } = $props()

  let label = $state("")

  let icon = $state("")

  const id = createID("spinny-label")

  const cssText = $derived(
    inline
      ? `top: ${top}; left: ${left}; height: ${size}; width: ${size};`
      : `top: ${top}; left: ${left}; width: ${size}`
  )

  $effect(() => {
    // prettier-ignore
    switch (status) {
      case "active":  label = t("spinny-label.active")  ?? t("message-loading") ?? "" ; break
      case "error":   label = t("spinny-label.error")   ?? t("message-loading") ?? "" ; break
      case "success": label = t("spinny-label.success") ?? t("message-loading") ?? "" ; break
      case "warning": label = t("spinny-label.warning") ?? t("message-loading") ?? "" ; break
    }
    // prettier-ignore
    switch (status) {
      case "active":  icon = ""                              ; break
      case "error":   icon = "fa-solid:exclamation-circle"   ; break
      case "success": icon = "fa-solid:check"                ; break
      case "warning": icon = "fa-solid:exclamation-triangle" ; break
    }
  })
</script>

<div
  style:top
  style:left
  style:font-size={inline ? size : "1rem"}
  class="spinny is-status-{status}"
  class:is-inline={inline}
  aria-atomic="true"
  aria-labelledby={id}
  aria-live={status === "active" && loud ? "assertive" : "off"}
  {...others}
>
  <!-- A11y label - is invisible to everything but screen readers -->
  <!-- svelte-ignore a11y_label_has_associated_control -->
  <label {id} class="spinny-label">{label}</label>

  <div class="spinny-symbol" aria-hidden="true">
    {#if status === "active"}
      {#if !inline}
        {#await sleep(wait) then _}
          <div
            transition:anim={{
              duration: 1000,
              easing: "circInOut",
              css: (t) => `transform: scale(${t}); opacity: ${t ** 2};`
            }}
          >
            <svg
              style={cssText}
              class="spinny-block"
              viewBox="-15 -15 30 30"
              xmlns="http://www.w3.org/2000/svg"
            >
              <circle class="spinny-block-circle" fill="none" r="13" stroke-width="2" />
              <circle class="spinny-block-arc" fill="none" r="13" stroke-width="2" />
            </svg>
          </div>
        {/await}
      {:else}
        <svg
          style={cssText}
          class="spinny-inline"
          viewBox="-3 -3 42 42"
          xmlns="http://www.w3.org/2000/svg"
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

<style global lang="scss">
  @use "../css/abstracts" as *;

  .spinny {
    position: absolute;
    z-index: $z-above;
    display: block;
    pointer-events: none;
    transform: translate(-50%, -50%);

    &.is-inline {
      position: relative;
      display: inline-flex;
      gap: 0.75ch;
      align-items: center;
      vertical-align: middle;
      text-align: center;
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

  .spinny-block-circle {
    opacity: 0.5;
    stroke: var(--col-border);
  }

  .spinny-block-arc {
    stroke: var(--col-hint);
    stroke-linecap: round;
    stroke-dasharray: 30, 90;
    stroke-dashoffset: 0;
  }

  @include tolerates-motion {
    .spinny-inline {
      animation: spinny-inline-rotate 1s linear infinite;
    }

    .spinny-inline-arc {
      transform-origin: top;
      transform-box: fill-box;
      animation: spinny-inline-spin 0.75s 0s infinite linear;
    }

    .spinny-block {
      animation: spinny-block-rotate 1s linear infinite;
    }

    .spinny-block-arc {
      animation: spinny-block-dash 3s ease-in-out alternate infinite;
    }
  }

  @include reduced-motion {
    // Don't show large spinners at all with reduced motion
    .spinny-block {
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

  // animations

  // prettier-ignore
  @keyframes skeleton-fade-in {
    0%   { opacity: 0; }
    100% { opacity: 1; }
  }

  // prettier-ignore
  @keyframes spinny-block-rotate {
    0%   { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }

  // prettier-ignore
  @keyframes spinny-block-dash {
    0%   { stroke-dasharray: 10, 90; }
    100% { stroke-dasharray: 60, 90; }
  }

  // prettier-ignore
  @keyframes spinny-inline-rotate {
    0%   { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
</style>
