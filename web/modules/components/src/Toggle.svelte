<!--
  @component Generic "toggle" component.
-->
<script lang="ts">
  import { keyHandle } from "./lib/controls"
  import Sprite from "./Sprite.svelte"

  /** State of the toggle. */
  export let toggled = false

  /** If true, the toggle will be rendered smaller. */
  export let small = false

  $: size = small ? "1.5em" : "2em"
</script>

<label class="toggleinput" class:is-toggled={toggled}>
  <input
    class="toggleinput-input"
    type="checkbox"
    bind:checked={toggled}
    use:keyHandle={{
      key: "Enter",
      do: () => void (toggled = !toggled)
    }}
    {...$$restProps}
  />
  <span class="toggleinput-wrapper" role="presentation">
    {#if $$slots.before}
      <span class="toggleinput-slot-before">
        <slot name="before" />
      </span>
    {/if}
    <Sprite i="wj-toggle" width={size} height={size} />
    <span class="toggleinput-slot-after"><slot /></span>
  </span>
</label>

<style lang="scss">
  @import "../../../resources/css/abstracts";

  .toggleinput {
    position: relative;

    @include hover {
      .toggleinput-wrapper > span {
        color: var(--col-hint);
      }
    }

    &.is-toggled {
      --wj-toggle-track-fill: var(--col-hint);
      --wj-toggle-handle-cx: 96px;
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
