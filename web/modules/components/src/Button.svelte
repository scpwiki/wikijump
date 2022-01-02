<!--
  @component Generic button.
  Designed to be as versatile possible, so there shouldn't be much reason not to
  use this component for any sort of button (except links).
-->
<script lang="ts">
  import Icon from "./Icon.svelte"
  import { tip as tippy } from "./lib/tippy"
  import Sprite from "./Sprite.svelte"

  /**
   * If provided, the button will be displayed as an icon instead. This
   * will replace any slotted content - it does not preprend an icon.
   */
  export let i = ""

  /** Disables the button. */
  export let disabled = false

  /** Makes the button appear in an activated state. */
  export let active = false

  /** Text displayed for the tooltip. */
  export let tip = ""

  /** Sets the size of the button by scaling the font-size. */
  export let size = "1em"

  /** If given, the button will act as a link. */
  export let href = ""

  /** If true, the button will submit forms. */
  export let submit = false

  // -- STYLE

  /** Makes the button as wide as possible. */
  export let wide = false

  /** Denotes the button as being important, which changes how it appears. */
  export let primary = false

  /** Removes most of the styling. */
  export let baseline = false

  /** Removes most of the padding around the button's icon or text. */
  export let compact = false

  /**
   * Styles the button to be disconnected from the background. Good for
   * buttons that are placed outside layout flow.
   */
  export let floating = false

  /** Removes the round corners (`border-radius`) from the styling. */
  export let sharp = false
</script>

{#if href}
  <a
    class="button is-link"
    {href}
    on:click
    use:tippy={tip}
    style="font-size: {size};"
    class:is-icon={i}
    class:is-active={active}
    class:is-disabled={disabled}
    class:is-wide={wide}
    class:is-primary={primary}
    class:is-baseline={baseline}
    class:is-compact={compact}
    class:is-floating={floating}
    class:is-sharp={sharp}
    {...$$restProps}
  >
    {#if i?.startsWith("wj-")}
      <Sprite {i} />
    {:else if i}
      <Icon {i} size="1em" />
    {:else}
      <slot />
    {/if}
  </a>
{:else}
  <button
    type={submit ? "submit" : "button"}
    {disabled}
    on:click
    use:tippy={tip}
    class="button"
    style="font-size: {size};"
    class:is-icon={i}
    class:is-active={active}
    class:is-disabled={disabled}
    class:is-wide={wide}
    class:is-primary={primary}
    class:is-baseline={baseline}
    class:is-compact={compact}
    class:is-floating={floating}
    class:is-sharp={sharp}
    {...$$restProps}
  >
    {#if i?.startsWith("wj-")}
      <Sprite {i} />
    {:else if i}
      <Icon {i} size="1em" />
    {:else}
      <slot />
    {/if}
  </button>
{/if}

<style global lang="scss">
  .button {
    display: inline-flex;
    gap: 0.5ch;
    align-items: center;
    padding: 0.25rem 1rem;
    font-size: 1rem;
    color: var(--col-text-subtle);
    text-align: center;
    vertical-align: middle;
    cursor: pointer;
    user-select: none;
    background: var(--col-border);
    border-radius: 0.25rem;
    transition: background 0.125s, color 0.125s, filter 0.125s;
    @include shadow(2);

    &.is-disabled {
      color: var(--col-lightgray) !important;
      cursor: not-allowed;
      filter: grayscale(50%);
      @include shadow(0);
    }

    &.is-sharp {
      border-radius: 0;
    }

    &.is-wide {
      display: block;
      width: 100%;
      text-align: center;
    }

    &.is-primary {
      color: var(--col-white);
      background: var(--col-hint);

      &.is-baseline {
        color: var(--col-text-subtle);
        background: none;
      }
    }

    &.is-baseline {
      padding: 0.25rem;
      background: none;
      @include shadow(0);
    }

    &.is-compact {
      padding: 0.125rem 0.25rem;
    }

    &.is-floating {
      background: none;
      filter: drop-shadow(0 2px 2px rgba(0, 0, 0, 0.5));
      opacity: 0.5;
      transition: color 0.125s, opacity 0.125s;
      @include shadow(0);

      &.is-icon {
        transition: transform 0.125s, color 0.125s, filter 0.125s, opacity 0.125s;
      }
    }

    &.is-icon {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      padding: 0.25rem;

      &.is-compact {
        padding: 0.25rem;
      }

      &.is-baseline.is-compact {
        padding: 0;
      }
    }

    @include hover {
      color: var(--col-hint);
      background: var(--col-border);

      &.is-primary {
        color: var(--col-white);
        background: var(--col-hint);
        filter: brightness(110%);
      }

      &.is-floating {
        background: none;
        opacity: 1;

        &.is-icon {
          transform: scale(1.075);
        }
      }
    }

    &:focus-visible {
      color: var(--col-hint);
      background: var(--col-border);
      outline-color: var(--col-border);
    }

    &:active,
    &.is-active {
      color: var(--col-hint);
      background: var(--col-border);
      filter: brightness(90%);

      &.is-baseline,
      &.is-floating {
        background: none;
      }

      &.is-primary {
        color: var(--col-white);
        background: var(--col-hint);

        &.is-baseline {
          color: var(--col-white);
          background: var(--col-hint);
          filter: none;
        }
      }

      &.is-floating.is-icon {
        opacity: 1;
        transform: scale(1);
      }
    }

    // click only, so not using active class
    &:active {
      &.is-baseline {
        background: var(--col-border);
      }
    }
  }
</style>
