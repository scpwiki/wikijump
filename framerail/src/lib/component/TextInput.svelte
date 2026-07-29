<!--
  @component A generic text field input.

  Automatically handles password revealing if the field's type is set to `password`.
-->
<script lang="ts">
  import { keyHandle, whileHeld } from "$lib/dom"
  import { format as t } from "$lib/util/locale"
  import { createEventDispatcher } from "svelte"
  import Button from "./Button.svelte"
  import Icon from "./Icon.svelte"
  import { tip } from "./scripts/tippy"

  const dispatch = createEventDispatcher()

  let {
    name = "",
    label = "",
    value = "",
    required = false,
    clearable = false,
    info = "",
    icon = "bi:slash-square",
    input = null,
    noborder = false,
    novalidate = false,
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

    /** If true, a button will be displayed to clear the input. */
    clearable?: boolean

    /** Extra info provided below the input. */
    info?: string

    /** Placeholder icon. */
    icon?: string

    /** The input element. */
    input?: HTMLInputElement | null

    /** If true, borders will be removed. */
    noborder?: boolean

    /** If true, validation indicators won't be shown. */
    novalidate?: boolean

    /** If true, the input will be as wide as possible. */
    wide?: boolean

    [key: string]: any
  } = $props()
</script>

<div class="textinput" class:is-wide={wide}>
  <label>
    {#if label}
      <div role="presentation">
        <span class="textinput-label">{label}</span>
        {#if required}
          <span class="textinput-required" use:tip={t("field-required")}>
            <Icon i="fa-solid:asterisk" size="0.5em" />
          </span>
        {/if}
      </div>
    {/if}

    <input
      bind:this={input}
      {name}
      class="textinput-input"
      class:is-noborder={noborder}
      class:is-novalidate={novalidate}
      {required}
      bind:value
      use:keyHandle={[
        {
          key: "Enter",
          preventDefault: true,
          do: () => dispatch("enter")
        }
      ]}
      {...others}
    />

    <!-- special case: input is a password type -->
    <!-- when this happens, we'll turn the icon into a "show password" button -->
    {#if input?.type === "password"}
      <!-- prettier-ignore -->
      <span
        class="textinput-icon is-password"
        aria-hidden="true"
        use:tip={t("hold-to-show-password")}
        use:whileHeld={{
          pressed: () => { if (input) input.type = "text" },
          released: () => { if (input) input.type = "password" }
        }}
      >
        <Icon i={icon} size="1.25em" />
      </span>
    {:else if clearable && value}
      <span class="textinput-icon is-clearable">
        <Button
          baseline
          compact
          i="wj-close"
          size="1.25em"
          on:click={() => (value = "")}
        />
      </span>
    {:else}
      <span class="textinput-icon" aria-hidden="true">
        <Icon i={icon} size="1.25em" />
      </span>
    {/if}
  </label>

  {#if info}
    <div class="textinput-info">{info}</div>
  {/if}
</div>

<style global lang="scss">
  @use "../css/abstracts/mixins" as *;

  .textinput {
    margin: 0.25em 0;

    &.is-wide {
      width: 100%;
    }

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
      transition:
        border 50ms,
        border-radius 50ms,
        box-shadow 50ms;
    }

    &.is-noborder {
      border: none;
    }

    &::placeholder {
      color: var(--col-text-dim);
      opacity: 0.5;
    }

    &:focus {
      outline: none;
      border-color: var(--col-hint);
    }

    &:not(.is-novalidate) {
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
    }

    &:disabled,
    &:not(:placeholder-shown) {
      + .textinput-icon:not(.is-password):not(.is-clearable) {
        opacity: 0;
      }

      + .textinput-icon.is-password {
        opacity: 1;
      }
    }

    &::-webkit-search-cancel-button {
      display: none;
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
      transition:
        color 100ms,
        opacity 100ms;
    }

    &.is-password,
    &.is-clearable {
      pointer-events: all;
      cursor: pointer;

      @include hover {
        color: var(--col-hint);
      }
    }
  }
</style>
