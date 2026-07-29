<!--
  @component Tab panel designed to be used with the `Tabview` component.

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
  import { createID } from "$lib/util"
  import { getContext } from "svelte"
  import type { Writable } from "svelte/store"
  import Button from "../Button.svelte"
  import { portal } from "../scripts/portal"

  let {
    button,
    children
  }: {
    button?: any
    children?: any
  } = $props()

  const id = createID()

  const buttonID = `tab-button${id}`
  const panelID = `tab-panel${id}`

  interface Tabs {
    buttons?: HTMLElement
    key: Writable<any>
    conditional: boolean
  }

  const { buttons, key, conditional } = getContext<Required<Tabs>>("tabs")

  let selected = $derived($key === id)

  // if the store has no tab selected, set the start tab to this tab
  if (!$key) selectThis()

  function selectThis() {
    $key = id
  }
</script>

<span
  class="tab-button"
  class:is-selected={selected}
  role="presentation"
  use:portal={buttons}
>
  <Button
    id={buttonID}
    active={selected}
    aria-controls={panelID}
    aria-selected={String(selected)}
    baseline
    sharp
    wide
    on:click={selectThis}
  >
    {@render button?.()}
  </Button>
</span>

<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
<div
  id={panelID}
  class="tab-panel"
  aria-labelledby={buttonID}
  hidden={!selected}
  tabindex="0"
>
  {#if selected || !conditional}{@render children?.()}{/if}
</div>

<style global lang="scss">
  @use "../../css/abstracts/mixins" as *;

  .tab-button {
    flex-grow: 1;
    font-family: var(--font-display);
    font-size: 1.125rem;
    font-weight: 500;
    text-align: center;
    border-bottom: solid 1px var(--col-border);
    border-left: solid 1px var(--col-border);

    &:first-child {
      border-left: none;
    }
  }

  .tab-panel {
    outline: none;
  }

  @include tolerates-motion {
    .tab-panel {
      animation: tab-panel-reveal 0.125s 0s 1 backwards ease-out;
    }
  }

  @keyframes tab-panel-reveal {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>
