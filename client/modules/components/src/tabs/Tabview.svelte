<script lang="ts">
  import { focusGroup } from "../lib/focus"
  import { onMount, setContext } from "svelte"
  import { writable } from "svelte/store"
  import type { Writable } from "svelte/store"

  export let conditional = false
  export let noborder = false
  export let contained = false
  export let compact = false

  let ready = false

  let buttons: HTMLElement | undefined

  let key = writable<any>(null)

  interface Tabs {
    buttons?: HTMLElement
    key: Writable<any>
    conditional: boolean
  }

  const tabs: Tabs = { key, conditional }
  setContext("tabs", tabs)

  onMount(() => {
    tabs.buttons = buttons
    ready = true
  })
</script>

<div
  class="tabs"
  role="presentation"
  class:is-noborder={noborder}
  class:is-contained={contained}
  class:is-compact={compact}
>
  <div
    bind:this={buttons}
    use:focusGroup={"horizontal"}
    class="tab-buttons"
    role="tablist"
  />
  <div class="tab-panels" role="presentation">
    {#if ready}<slot />{/if}
  </div>
</div>

<style lang="scss">
  .tabs {
    width: 100%;

    .tab-buttons {
      display: flex;
      flex-wrap: wrap;
    }

    .tab-panels {
      padding: 0.5rem;
      border: 0.075rem solid var(--col-border);
      border-radius: 0 0 0.25rem 0.25rem;
      transition: border-color 0.125s;
    }

    &.is-contained {
      height: 100%;

      .tab-panels {
        position: relative;
        height: 100%;

        > :global(.tab-panel) {
          height: 100%;
        }
      }
    }

    &.is-noborder .tab-panels {
      padding: 0.5rem 0;
      border: none;
      border-radius: 0;
    }

    &.is-compact {
      .tab-panels {
        padding-top: 0;
      }

      .tab-buttons > :global(.tab-button) {
        flex-grow: 0.05;
      }
    }
  }
</style>
