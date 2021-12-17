<script lang="ts">
  import { keyHandle, getFoci, onHover } from "@wikijump/dom"
  import type { Placement } from "@popperjs/core"
  import { popover } from "./lib/popper"
  import { guard } from "./lib/use-guard"

  /** If true, the menu will open when the summary element is hovered over. */
  export let hoverable = false

  /** Sets whether the menu is open or not. */
  export let open = false

  /** Popover placement location for the menu. */
  export let placement: Placement = "bottom"

  let details: HTMLElement
  let summary: HTMLElement

  // misc. functions

  /**
   * Checks if a pointer event is outside of the details menu. If so, the
   * menu will be closed if it isn't already.
   */
  function checkClose(evt: PointerEvent) {
    if (!open) return
    if (!evt.target) return
    if (evt.target === details) return
    if (details.contains(evt.target as Node)) return
    closeMenu()
  }

  // helper functions

  function toggleMenu() {
    open = !open
  }

  function openMenu() {
    open = true
  }

  function closeMenu() {
    open = false
  }

  function selectFirstActive() {
    if (document.activeElement === summary) {
      getFoci(details)[0]?.focus()
    }
  }
</script>

<svelte:body on:pointerdown={checkClose} />

<details
  class="details-menu"
  bind:this={details}
  {open}
  {...$$restProps}
  use:keyHandle={[
    { key: "Escape", do: closeMenu },
    { key: "ArrowDown", do: selectFirstActive }
  ]}
>
  <summary
    class="details-menu-summary"
    bind:this={summary}
    use:keyHandle={[
      { key: "click", preventDefault: true, do: toggleMenu },
      { key: "Enter", preventDefault: true, do: toggleMenu }
    ]}
    use:guard={{
      when: hoverable,
      use: [onHover, { alsoOnFocus: true, on: openMenu, off: closeMenu }]
    }}
  >
    <slot name="button" />
  </summary>

  <div
    class="details-menu-popover"
    use:popover={{ when: open && !!summary, placement, target: summary }}
  >
    <slot {open} />
  </div>
</details>

<style lang="scss">
  @import "../../../resources/css/abstracts";

  @keyframes reveal {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .details-menu {
    position: relative;
    display: inline-block;
  }

  .details-menu-popover {
    position: absolute;
    z-index: $z-popover;
  }

  .details-menu[open] > .details-menu-popover {
    animation: reveal 0.125s 1 0s backwards ease-out;
  }
</style>
