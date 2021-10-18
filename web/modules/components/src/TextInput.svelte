<script lang="ts">
  import { createEventDispatcher } from "svelte"
  import { keyHandle } from "./lib/controls"
  import { tip } from "./lib/tippy"
  import Icon from "./Icon.svelte"
  import { t } from "@wikijump/api"

  /** The label describing the input. */
  export let label = ""

  /** The current value of the input. */
  export let value = ""

  /** If true, the input must be filled or else it will display as invalid. */
  export let required = false

  /** Extra info provided below the input. */
  export let info = ""

  /** Placeholder icon. */
  export let icon = "bi:slash-square"

  /** The input element. */
  export let input: HTMLInputElement | null = null

  /** If true, borders will be removed. */
  export let noborder = false

  const dispatch = createEventDispatcher()

  const keyHandler = [
    {
      key: "Enter",
      preventDefault: true,
      do() {
        dispatch("enter")
      }
    }
  ]
</script>

<div class="textinput">
  <label>
    {#if label}
      <div role="presentation">
        <span class="textinput-label">{label}</span>
        {#if required}
          <span class="textinput-required" use:tip={$t("components.textinput.REQUIRED")}>
            <Icon i="fa-solid:asterisk" size="0.5em" />
          </span>
        {/if}
      </div>
    {/if}

    <input
      bind:this={input}
      bind:value
      use:keyHandle={keyHandler}
      class="textinput-input"
      class:is-noborder={noborder}
      {...$$restProps}
    />

    <span class="textinput-icon" aria-hidden="true">
      <Icon i={icon} size="1.25em" />
    </span>
  </label>

  {#if info}
    <div class="textinput-info">{info}</div>
  {/if}
</div>

<style lang="scss">
  @import "../../../resources/css/abstracts";

  .textinput {
    margin: 0.25rem 0;

    > label {
      position: relative;
      display: block;
    }
  }

  .textinput-label {
    padding-left: 0.25em;
    font-size: 0.825em;
    color: var(--col-text-subtle);
  }

  .textinput-info {
    padding-left: 0.25em;
    margin-top: 0.25em;
    font-size: 0.75em;
    line-height: 1.4;
    color: var(--col-text-subtle);
  }

  .textinput-required {
    margin-left: 0.25em;
    color: var(--col-danger);
  }

  .textinput-input {
    width: 100%;
    padding: 0.25em 0.5em;
    color: var(--col-text);
    background: var(--col-background-dim);
    border: solid 0.075rem var(--col-border);
    border-radius: 0.25em;
    box-shadow: inset 0.2em 0 0 -0.1em transparent;

    @include tolerates-motion {
      transition: border 50ms, border-radius 50ms, box-shadow 50ms;
    }

    &.is-noborder {
      border: none;
    }

    &::placeholder {
      color: var(--col-text-dim);
      opacity: 0.5;
    }

    &:focus {
      border-color: var(--col-hint);
      outline: none;
    }

    &:valid:not(:placeholder-shown) {
      border-left-color: var(--col-success);
      border-radius: 0.125em 0.25em 0.25em 0.125em;
      box-shadow: inset 0.25em 0 0 -0.1em var(--col-success);
    }

    &:invalid:not(:placeholder-shown) {
      border-left-color: var(--col-danger);
      border-radius: 0.125em 0.25em 0.25em 0.125em;
      box-shadow: inset 0.25em 0 0 -0.1em var(--col-danger);
    }

    &:disabled,
    &:not(:placeholder-shown) {
      + .textinput-icon {
        opacity: 0;
      }
    }
  }

  .textinput-icon {
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
      transition: opacity 100ms;
    }
  }
</style>
