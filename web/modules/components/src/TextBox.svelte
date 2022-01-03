<script lang="ts">
  import { keyHandle } from "@wikijump/dom"
  import { format as t } from "@wikijump/fluent"
  import { createEventDispatcher } from "svelte"
  import Icon from "./Icon.svelte"
  import { tip } from "./lib/tippy"

  const dispatch = createEventDispatcher()

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

<div class="textbox" class:is-wide={wide}>
  <label>
    {#if label}
      <div role="presentation">
        <span class="textbox-label">{label}</span>
        {#if required}
          <span class="textbox-required" use:tip={t("field-required")}>
            <Icon i="fa-solid:asterisk" size="0.5em" />
          </span>
        {/if}
      </div>
    {/if}

    <textarea
      {name}
      bind:value
      use:keyHandle={{
        key: "Enter",
        preventDefault: true,
        do: () => dispatch("enter")
      }}
      class="textbox-textarea"
      {required}
      {...$$restProps}
    />
  </label>
</div>

<style global lang="scss">
  .textbox {
    margin: 0.25rem 0;

    &.is-wide {
      width: 100%;
    }

    > label {
      position: relative;
      display: block;
    }
  }

  .textbox-label {
    padding-left: 0.25em;
    font-size: 0.825em;
    color: var(--col-text-subtle);
  }

  .textbox-info {
    padding-left: 0.25em;
    margin-top: 0.25em;
    font-size: 0.75em;
    line-height: 1.4;
    color: var(--col-text-subtle);
  }

  .textbox-required {
    margin-left: 0.25em;
    color: var(--col-danger);
  }

  .textbox-textarea {
    width: 100%;
    min-height: 5rem;
    padding: 0.5em;
    font-family: var(--font-mono);
    color: var(--col-text);
    background: var(--col-background-dim);
    border: solid 0.075rem var(--col-border);
    border-radius: 0.25em;
    box-shadow: inset 0.2em 0 0 -0.1em transparent;

    &:focus {
      border-color: var(--col-hint);
      outline: none;
    }
  }
</style>
