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

  /** A prop object passed to the {@link component}, if one has been provided. */
  export let detail: Record<string, any> = {}

  /** If true, the dialog is displayed. */
  export let open = false

  /**
   * If true, slotted content will only be inserted into the DOM when the
   * dialog is open. True by default.
   */
  export let lazy = true

  let state = open
  let previousFocus: HTMLElement | null = null

  /** Restore the previous focus. */
  function restoreFocus() {
    if (previousFocus) {
      previousFocus.focus()
      previousFocus = null
    }
  }

  /** Show the modal and save the previous focus. */
  function show() {
    previousFocus = document.activeElement as HTMLElement
    dialog.showModal()
  }

  /** Hide the modal and restore the previous focus. */
  function close() {
    dialog.close()
    restoreFocus()
  }

  $: if (dialog && state !== open) {
    if (open && !state) show()
    else if (!open && state) close()
    state = open
    dispatch("change", state)
    if (state) dispatch("open")
  }

  onMount(() => {
    dialogPolyfill.registerDialog(dialog)
    if (open) show()
    dialog.addEventListener("cancel", () => restoreFocus())
  })
</script>

<dialog bind:this={dialog} on:cancel on:close>
  {#if !lazy || open}
    {#if component}
      <svelte:component this={component} {detail} />
    {/if}
    <slot />
  {/if}
</dialog>
