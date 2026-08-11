<!--
  @component Tab handler component intended to be used with the `Tab` component.

  Usage:
  ```svelte
    <Tabview>
      <Tab>
        <span slot="button">Tab selector button text.</span>
        Tab panel contents.
      </Tab>
    </Tabview>
  ```
-->
<script lang="ts">
  import { focusGroup } from "$lib/dom"
  import { onMount, setContext } from "svelte"
  import { writable, type Writable } from "svelte/store"

  let {
    conditional = false,
    noborder = false,
    contained = false,
    compact = false,
    children
  }: {
    conditional?: boolean
    noborder?: boolean
    contained?: boolean
    compact?: boolean
    children?: any
  } = $props()

  let ready = $state(false)

  let buttons: HTMLElement | undefined

  let key = writable<any>(null)

  interface Tabs {
    buttons?: HTMLElement
    key: Writable<any>
    conditional: boolean
  }

  // svelte-ignore state_referenced_locally
  const tabs: Tabs = { key, conditional }
  setContext("tabs", tabs)

  onMount(() => {
    tabs.buttons = buttons
    ready = true
  })
</script>

<div
  class="tabs"
  class:is-compact={compact}
  class:is-contained={contained}
  class:is-noborder={noborder}
  role="presentation"
>
  <div
    bind:this={buttons}
    class="tab-buttons"
    role="tablist"
    use:focusGroup={"horizontal"}
  ></div>
  <div class="tab-panels" role="presentation">
    {#if ready}{@render children?.()}{/if}
  </div>
</div>

<style global lang="scss">
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

        > .tab-panel {
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

      .tab-buttons > .tab-button {
        flex-grow: 0.05;
      }
    }
  }
</style>
