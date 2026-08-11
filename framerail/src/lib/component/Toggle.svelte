<!--
  @component Generic "toggle" component.
-->
<script lang="ts">
  import { keyHandle } from "$lib/dom/svelte-adapters"

  let {
    name = "",
    type = "slider",
    toggled = $bindable(false),
    size = "1em",
    wide = false,
    flipped = false,
    children,
    ...others
  }: {
    /** Assigns a name to the input. */
    name?: string

    /** Type of "toggle" to use. Only has a visual effect. */
    type?: "slider" | "checkbox"

    /** State of the toggle. */
    toggled?: boolean

    /** Size of the toggle. */
    size?: string

    /** Makes the toggle fill its container. */
    wide?: boolean

    /** Flips the position of the label and the toggle. */
    flipped?: boolean

    children?: any
    [key: string]: any
  } = $props()
</script>

<label class="toggleinput" class:is-toggled={toggled} class:is-wide={wide}>
  <input
    {name}
    class="toggleinput-input"
    type="checkbox"
    bind:checked={toggled}
    use:keyHandle={[
      {
        key: "Enter",
        do: () => void (toggled = !toggled)
      }
    ]}
    {...others}
  />
  <span class="toggleinput-wrapper" role="presentation">
    {#if flipped}
      <span class="toggleinput-slot-before">{@render children?.()}</span>
    {/if}

    {#if type === "slider"}
      <svg class="toggleinput-sprite is-slider" height={size} viewBox="0 0 128 64">
        <rect class="toggleinput-track" height="60" rx="30" width="124" x="2" y="2" />
        <rect fill="#0002" height="32" rx="16" width="96" x="16" y="16" />
        <circle class="toggleinput-handle" cx="32" cy="32" r="26" />
      </svg>
    {:else if type === "checkbox"}
      <svg class="toggleinput-sprite is-checkbox" height={size} viewBox="0 0 64 64">
        <rect class="toggleinput-box" height="60" rx="8" width="60" x="2" y="2" />
        <path class="toggleinput-checkmark" d="m13.25 32 12.5 12.5 25 -25" />
      </svg>
    {/if}

    {#if !flipped}
      <span class="toggleinput-slot-after">{@render children?.()}</span>
    {/if}
  </span>
</label>

<style global lang="scss">
  @use "../css/abstracts/mixins" as *;

  .toggleinput {
    position: relative;
    cursor: pointer;

    &.is-wide {
      width: 100%;

      .toggleinput-wrapper {
        display: inline-flex;
        justify-content: space-between;
        width: 100%;
      }
    }

    @include hover {
      .toggleinput-wrapper > span {
        color: var(--col-hint);
      }

      &:not(.is-toggled) {
        .toggleinput-checkmark {
          opacity: 0.25;
          stroke: var(--col-text);
        }
      }
    }

    &.is-toggled {
      .toggleinput-track {
        fill: var(--col-hint);
      }

      .toggleinput-handle {
        cx: 96px;
      }

      .toggleinput-box {
        fill: var(--col-hint);
        stroke: #222;
      }

      .toggleinput-checkmark {
        opacity: 1;
      }
    }
  }

  .toggleinput-track {
    fill: #aaa;

    @include tolerates-motion {
      transition: fill 150ms;
    }
  }

  .toggleinput-handle {
    fill: #fff;
    cx: 32px;

    @include tolerates-motion {
      transition:
        cx 150ms,
        fill 150ms;
    }
  }

  .toggleinput-box {
    fill: none;
    stroke: var(--col-con-border);
    stroke-width: 4;

    @include tolerates-motion {
      transition:
        fill 75ms,
        stroke 75ms;
    }
  }

  .toggleinput-checkmark {
    opacity: 0;
    fill: none;
    stroke: #fff;
    stroke-width: 8;
    stroke-linecap: round;
    stroke-linejoin: round;

    @include tolerates-motion {
      transition: opacity 75ms;
    }
  }

  .toggleinput-wrapper {
    display: flex;
    align-items: center;

    > span {
      font-size: 0.875em;
      user-select: none;

      @include tolerates-motion {
        transition: color 150ms;
      }
    }

    .toggleinput-slot-after {
      margin-left: 0.5em;
    }
  }

  .toggleinput-input {
    position: absolute;
    top: 0;
    left: 0;
    width: 0;
    height: 0;
    opacity: 0;

    &:focus-visible ~ .toggleinput-wrapper {
      outline: 5px auto Highlight;
      outline: 5px auto -webkit-focus-ring-color;

      > span {
        color: var(--col-hint);
      }
    }
  }
</style>
