<script lang="ts">
  import { getContext } from "svelte"
  import { createID } from "wj-util"
  import { portal } from "../lib/portal"
  import Button from "../Button.svelte"
  import type { Writable } from "svelte/store"

  const id = createID()

  const buttonID = `tab-button${id}`
  const panelID = `tab-panel${id}`

  interface Tabs {
    buttons?: HTMLElement
    key: Writable<any>
    conditional: boolean
  }

  const { buttons, key, conditional } = getContext<Required<Tabs>>("tabs")

  let selected = false
  $: selected = $key === id

  // if the store has no tab selected, set the start tab to this tab
  if (!$key) selectThis()

  function selectThis() {
    $key = id
  }
</script>

<span
  class="tab-button"
  class:is-selected={selected}
  use:portal={buttons}
  role="presentation"
>
  <Button
    baseline
    sharp
    wide
    active={selected}
    on:click={selectThis}
    id={buttonID}
    aria-controls={panelID}
    aria-selected={String(selected)}
  >
    <slot name="button" />
  </Button>
</span>

<div
  class="tab-panel"
  hidden={!selected}
  id={panelID}
  aria-labelledby={buttonID}
  tabindex="0"
>
  {#if selected || !conditional}<slot />{/if}
</div>

<style lang="scss">
  @import "../../../wj-css/src/abstracts";

  .tab-button {
    flex-grow: 1;
    border-left: solid 0.075rem var(--col-border);

    @include overlay {
      transition: box-shadow 0.125s;
    }

    &.is-selected::after {
      box-shadow: inset 0 -0.125rem 0 0 colvar("hint");
    }

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
