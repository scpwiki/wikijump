<!--
  @component Generic dialog component.
  Doesn't handle inserting itself into a "correct" place in the DOM.
-->
<script lang="ts">
  import { scrollElement } from "$lib/dom/scrolling"
  import { createEventDispatcher, onMount, type Component } from "svelte"

  // the additional methods are what it is in the spec and fulfilled by the polyfill
  let dialog:
    | HTMLDialogElement
    | (HTMLElement & { showModal: () => void; close: () => void; show: () => void })

  let dispatch = createEventDispatcher()

  let {
    component = null,
    detail = {},
    open = $bindable(false),
    lazy = true,
    children,
    ...rest
  }: {
    /**
     * The Svelte component to slot. This is provided as an option in case
     * you are manually constructing this component, and can't use slots.
     */
    component?: Component | null

    /**
     * A prop object passed to the {@link component}, if one has been
     * provided.
     */
    detail?: Record<string, any>

    /** If true, the dialog is displayed. */
    open?: boolean

    /**
     * If true, slotted content will only be inserted into the DOM when the
     * dialog is open. True by default.
     */
    lazy?: boolean

    children?: any
    [key: string]: any
  } = $props()

  /**
   * Function passed to a provided {@link component} that closes the dialog
   * when called.
   */
  const closeDialog = () => void (open = false)

  // svelte-ignore state_referenced_locally
  let state = $state(open)
  let previousFocus: HTMLElement | null = null

  /**
   * Event handler for preventing the main page from scrolling if the user
   * tries to scroll inside of the dialog.
   */
  function preventPageScrolling(evt: WheelEvent) {
    if (!dialog || !evt.target) evt.preventDefault()
    const target = evt.target as HTMLElement
    // check if target is inside of the dialog
    if (target === dialog || dialog.contains(target)) {
      // figure out if our scrolling element is inside of the dialog
      // if it isn't, prevent the scroll
      const scroll = scrollElement(evt.target as HTMLElement)
      if (target !== scroll && !dialog.contains(scroll)) evt.preventDefault()
    }
    // target outside of dialog, we can prevent scrolling for sure
    else {
      evt.preventDefault()
    }
  }

  /**
   * Cleans up after the dialog has been closed, e.g. by restoring the
   * previous focus.
   */
  function cleanup() {
    document.removeEventListener("wheel", preventPageScrolling)
    if (previousFocus) {
      previousFocus.focus()
      previousFocus = null
    }
  }

  /** Show the modal and save the previous focus. */
  function show() {
    previousFocus = document.activeElement as HTMLElement
    dialog.showModal()
    document.addEventListener("wheel", preventPageScrolling, { passive: false })
    dispatch("open")
  }

  /** Hide the modal and restore the previous focus. */
  function close() {
    dialog.close()
    cleanup()
  }

  $effect(() => {
    if (dialog && state !== open) {
      if (open && !state) show()
      else if (!open && state) close()
      state = open
      dispatch("change", state)
    }
  })

  onMount(() => {
    // dialogPolyfill.registerDialog(dialog)
    if (open) show()
    dialog.addEventListener("cancel", () => cleanup())
    return cleanup
  })
</script>

<dialog bind:this={dialog} oncancel={() => {}} onclose={() => {}} {...rest}>
  {#if !lazy || open}
    {#if component}
      <component {closeDialog} {detail}></component>
    {/if}
    {@render children?.()}
  {/if}
</dialog>
