<script lang="ts">
  import { tip as tippy } from "./lib/tippy"
  import Icon from "./Icon.svelte"

  export let i = ""
  export let disabled = false
  export let active = false
  export let tip = ""

  export let wide = false
  export let primary = false
  export let baseline = false
  export let compact = false
  export let floating = false
  export let sharp = false
  export let size = "1em"
</script>

<button
  type="button"
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
  {#if i}<Icon {i} size="1em" />{:else}<slot />{/if}
</button>

<style lang="scss">
  @import "../../wj-css/src/abstracts";

  .button {
    display: inline-flex;
    gap: 0.5ch;
    align-items: center;
    padding: 0.25rem 1rem;
    font-size: 1rem;
    color: var(--col-subtle);
    text-align: center;
    vertical-align: middle;
    background: var(--col-border);
    border-radius: 0.25rem;
    transition: background 0.125s, color 0.125s, filter 0.125s;
    @include shadow(4);

    &.is-disabled {
      color: var(--col-lightgray) !important;
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

      &.is-baseline {
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

    &:active,
    &.is-active {
      color: var(--col-hint);
      background: var(--col-border);
      filter: brightness(90%);

      &.is-primary {
        color: var(--col-white);
        background: var(--col-hint);
      }

      &.is-baseline,
      &.is-floating {
        background: none;
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
