<script lang="ts">
  import { format as t } from "$lib/util/locale"
  import Icon from "./Icon.svelte"
  import { tip } from "./scripts/tippy"

  let {
    name = "",
    label = "",
    value = "",
    required = false,
    max = 0,
    wide = false,
    ...others
  }: {
    /** Assigns a name to the input. */
    name?: string

    /** The label describing the input. */
    label?: string

    /** The current value of the input. */
    value?: string

    /**
     * If true, the input must be filled or else it will display as
     * invalid.
     */
    required?: boolean

    /** Maximum characters for the input. */
    max?: number

    /** If true, the input will be as wide as possible. */
    wide?: boolean

    [key: string]: any
  } = $props()
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
      class="textbox-textarea"
      maxLength={max || null}
      {required}
      bind:value
      {...others}></textarea>
  </label>

  <!-- {#if max}
    <div class="textbox-count">
      {t("characters-left", { count: Math.max(0, max - value.length) })}
    </div>
  {/if} -->
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
      outline: none;
      border-color: var(--col-hint);
    }
  }
</style>
