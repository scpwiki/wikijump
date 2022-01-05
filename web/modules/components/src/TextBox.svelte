<script lang="ts">
  import { format as t } from "@wikijump/fluent"
  import Icon from "./Icon.svelte"
  import { tip } from "./lib/tippy"

  /** Assigns a name to the input. */
  export let name = ""

  /** The label describing the input. */
  export let label = ""

  /** The current value of the input. */
  export let value = ""

  /** If true, the input must be filled or else it will display as invalid. */
  export let required = false

  /** Maximum characters for the input. */
  export let max = 0

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
      class="textbox-textarea"
      {required}
      maxLength={max || null}
      {...$$restProps}
    />
  </label>

  {#if max}
    <div class="textbox-count">
      {t("characters-left", { count: Math.max(0, max - value.length) })}
    </div>
  {/if}
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

  .textbox-count {
    padding-left: 0.25em;
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
