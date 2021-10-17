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

  /** The input element. */
  export let input: HTMLInputElement | null = null

  /** If true, borders will be removed. */
  export let noborder = false

  /** If true, the input will be rendered much thinner. */
  export let thin = false

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

<label class="textinput">
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
    class:is-thin={thin}
    class:is-noborder={noborder}
    {...$$restProps}
  /><!--
  --><span role="presentation" class="textinput-icon" class:is-thin={thin} />
</label>

{#if info}
  <div class="textinput-info">{info}</div>
{/if}

<style lang="scss">
  @import "../../../resources/css/abstracts";

  .textinput-label {
    padding-left: 0.25em;
    font-size: 0.875em;
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
    vertical-align: text-bottom;
  }

  .textinput-input {
    padding: 0.25em 0.5em;
    color: var(--col-text);
    background: var(--col-background-dim);
    border: solid 0.075rem var(--col-border);
    border-radius: 0.25em;
    box-shadow: inset 0.2em 0 0 -0.1em transparent;

    @include tolerates-motion {
      transition: border 100ms, box-shadow 100ms;
    }

    &.is-thin {
      height: 2em;
      padding-top: 0;
      padding-bottom: 0;
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
      box-shadow: inset 0.2em 0 0 -0.1em var(--col-success);
    }

    &:invalid:not(:placeholder-shown) {
      box-shadow: inset 0.2em 0 0 -0.1em var(--col-danger);
    }
  }

  .textinput-icon {
    position: absolute;
    display: inline-block;
    width: 2.125em;
    height: 2.125em;
    pointer-events: none;
    user-select: none;
    background-color: var(--col-text-dim);
    opacity: 0.5;
    transform: translateX(-2em);
    mask-image: var(--icon-text-input);
    mask-repeat: no-repeat;
    mask-size: 1.25em;
    mask-position: center;

    @include tolerates-motion {
      transition: opacity 100ms;
    }

    &.is-thin {
      width: 2em;
      height: 2em;
    }
  }

  .textinput-input:not(:placeholder-shown) + .textinput-icon,
  .textinput-input:disabled + .textinput-icon {
    opacity: 0;
  }
</style>
