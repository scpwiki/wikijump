<script lang="ts">
  import { createEventDispatcher, onMount, SvelteComponent } from "svelte"
  import dialogPolyfill from "dialog-polyfill"

  // the additional methods are what it is in the spec and fulfilled by the polyfill
  let dialog: HTMLElement & { showModal: () => void; close: () => void; show: () => void }

  let dispatch = createEventDispatcher()

  /**
   * The Svelte component to slot. This is provided as an option in case
   * you are manually constructing this component, and can't use slots.
   */
  export let component: typeof SvelteComponent | null = null

  /** If true, the dialog is displayed. */
  export let open = false

  /**
   * If true, slotted content will only be inserted into the DOM when the
   * dialog is open. True by default.
   */
  export let lazy = true

  let state = open

  $: if (dialog && state !== open) {
    if (open && !state) dialog.showModal()
    else if (!open && state) dialog.close()
    state = open
    dispatch("change", state)
    if (state) dispatch("open")
  }

  onMount(() => {
    dialogPolyfill.registerDialog(dialog)
    if (open) dialog.showModal()
  })
</script>

<dialog bind:this={dialog} on:cancel on:close>
  {#if !lazy || open}
    {#if component}
      <svelte:component this={component} />
    {/if}
    <slot />
  {/if}
</dialog>
