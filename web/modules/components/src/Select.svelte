<script lang="ts">
  import { format as t } from "@wikijump/fluent"
  import Icon from "./Icon.svelte"
  import Sprite from "./Sprite.svelte"
  import { tip } from "./lib/tippy"

  /** Assigns a name to the input. */
  export let name = ""

  /** The label describing the input. */
  export let label = ""

  /** The current value of the input. */
  export let value = ""

  /** If true, the input must be filled or else it will display as invalid. */
  export let required = false

  /** If true, the input will be as wide as possible. */
  export let wide = false
</script>

<div class="select" class:is-wide={wide}>
  <label>
    {#if label}
      <div role="presentation">
        <span class="select-label">{label}</span>
        {#if required}
          <span class="select-required" use:tip={t("field-required")}>
            <Icon i="fa-solid:asterisk" size="0.5em" />
          </span>
        {/if}
      </div>
    {/if}

    <select {name} bind:value class="select-element" {required} {...$$restProps}>
      <slot />
    </select>

    <span class="select-icon" aria-hidden="true">
      <Sprite i="wj-downarrow" size="1.25em" />
    </span>
  </label>
</div>

<style global lang="scss">
  .select {
    margin: 0.25rem 0;

    &.is-wide {
      width: 100%;
    }

    > label {
      position: relative;
      display: block;
    }
  }

  .select-label {
    padding-left: 0.25em;
    font-size: 0.825em;
    color: var(--col-text-subtle);
  }

  .select-required {
    margin-left: 0.25em;
    color: var(--col-danger);
  }

  .select-element {
    width: 100%;
    min-height: 2rem;
    padding: 0.25em 0.5em;
    color: var(--col-text);
    background: var(--col-background-dim);
    border: solid 0.075rem var(--col-border);
    border-radius: 0.25em;
    box-shadow: inset 0.2em 0 0 -0.1em transparent;
    appearance: none;

    &:focus {
      border-color: var(--col-hint);
      outline: none;

      &:active {
        border-bottom: solid 0.075rem var(--col-border);
        border-radius: 0.25em 0.25em 0 0;

        + .select-icon {
          opacity: 1;
        }
      }
    }

    option {
      font-family: var(--font-mono);
      font-size: 0.825em;
      white-space: pre;
    }
  }

  .select-icon {
    position: absolute;
    right: 0.25em;
    bottom: 0.375em;
    display: inline-block;
    width: 1.5em;
    height: 1.5em;
    color: var(--col-text-dim);
    pointer-events: none;
    user-select: none;
    opacity: 0.5;

    @include tolerates-motion {
      transition: 100ms opacity;
    }
  }
</style>
