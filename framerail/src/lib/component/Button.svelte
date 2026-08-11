<!--
  @component Generic button.
  Designed to be as versatile possible, so there shouldn't be much reason not to
  use this component for any sort of button (except links).
-->
<script lang="ts">
  import Icon from "./Icon.svelte"
  import { tip as tippy } from "./scripts/tippy"
  import Sprite from "./Sprite.svelte"
  import { resolve } from "$app/paths"

  let {
    i = "",
    disabled = false,
    active = false,
    tip = "",
    size = "1em",
    href = "",
    submit = false,

    // -- STYLE
    wide = false,
    primary = false,
    baseline = false,
    compact = false,
    floating = false,
    sharp = false,

    children,
    ...others
  }: {
    /**
     * If provided, the button will be displayed as an icon instead. This
     * will replace any slotted content - it does not preprend an icon.
     */
    i?: string
    /** Disables the button. */
    disabled?: boolean

    /** Makes the button appear in an activated state. */
    active?: boolean

    /** Text displayed for the tooltip. */
    tip?: string

    /** Sets the size of the button by scaling the font-size. */
    size?: string

    /** If given, the button will act as a link. */
    href?: string

    /** If true, the button will submit forms. */
    submit?: boolean

    // -- STYLE

    /** Makes the button as wide as possible. */
    wide?: boolean

    /** Denotes the button as being important, which changes how it appears. */
    primary?: boolean

    /** Removes most of the styling. */
    baseline?: boolean

    /** Removes most of the padding around the button's icon or text. */
    compact?: boolean

    /**
     * Styles the button to be disconnected from the background. Good for
     * buttons that are placed outside layout flow.
     */
    floating?: boolean

    /** Removes the round corners (`border-radius`) from the styling. */
    sharp?: boolean

    children?: any
    [key: string]: any
  } = $props()
</script>

{#if href}
  <a
    style:font-size={size}
    class="wj-button is-link"
    class:is-active={active}
    class:is-baseline={baseline}
    class:is-compact={compact}
    class:is-disabled={disabled}
    class:is-floating={floating}
    class:is-icon={i}
    class:is-primary={primary}
    class:is-sharp={sharp}
    class:is-wide={wide}
    href={resolve(href, {})}
    onclick={() => {}}
    use:tippy={tip}
    {...others}
  >
    {#if i?.startsWith("wj-")}
      <Sprite {i} />
    {:else if i}
      <Icon {i} size="1em" />
    {:else}
      {@render children?.()}
    {/if}
  </a>
{:else}
  <button
    style:font-size={size}
    class="wj-button"
    class:is-active={active}
    class:is-baseline={baseline}
    class:is-compact={compact}
    class:is-disabled={disabled}
    class:is-floating={floating}
    class:is-icon={i}
    class:is-primary={primary}
    class:is-sharp={sharp}
    class:is-wide={wide}
    {disabled}
    onclick={() => {}}
    type={submit ? "submit" : "button"}
    use:tippy={tip}
    {...others}
  >
    {#if i?.startsWith("wj-")}
      <Sprite {i} />
    {:else if i}
      <Icon {i} size="1em" />
    {:else}
      {@render children?.()}
    {/if}
  </button>
{/if}

<style global lang="scss">
  @use "../css/abstracts/mixins" as *;

  .wj-button {
    display: inline-flex;
    gap: 0.5ch;
    align-items: center;
    padding: 0.25rem 1rem;
    font-size: 1rem;
    vertical-align: middle;
    color: var(--col-text-subtle);
    text-align: center;
    cursor: pointer;
    user-select: none;
    background: var(--col-border);
    border: 0;
    border-radius: 0.25rem;
    transition:
      background 0.125s,
      color 0.125s,
      filter 0.125s;
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
      opacity: 0.5;
      filter: drop-shadow(0 2px 2px rgba(0, 0, 0, 0.5));
      transition:
        color 0.125s,
        opacity 0.125s;
      @include shadow(0);

      &.is-icon {
        transition:
          transform 0.125s,
          color 0.125s,
          filter 0.125s,
          opacity 0.125s;
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
      outline-color: var(--col-border);
      background: var(--col-border);
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
