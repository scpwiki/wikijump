<script lang="ts">
  export let exitPrompt: () => void
  import { onMount, onDestroy } from "svelte"
  import { page } from "$app/stores"
  import { useErrorPopup } from "$lib/stores"
  let showErrorPopup = useErrorPopup()
  function containerExitPrompt(e: Event) {
    if ((e.target as HTMLElement).classList.contains("modal-container")) exitPrompt()
  }
  const escKeydown = (e: KeyboardEvent) => {
    if (e.code.toLowerCase() === "escape") exitPrompt()
  }
  onMount(() => {
    window.addEventListener("keydown", escKeydown)
  })
  onDestroy(() => {
    window.removeEventListener("keydown", escKeydown)
  })
</script>

<div
  class="modal-container"
  aria-describedby="modal-message"
  aria-labelledby="modal-title"
  role="presentation"
  on:click={containerExitPrompt}
  on:keydown={escKeydown}
>
  <div class="modal error-modal">
    <h2 id="modal-title">
      {($page.data.internationalization ?? $page.error?.internationalization)?.error}
    </h2>
    <div id="modal-message" class="modal-message">
      {$showErrorPopup.message}
    </div>
    {#if $showErrorPopup.data}
      <div id="model-message-extra" class="modal-message-extra">
        {$showErrorPopup.data}
      </div>
    {/if}
  </div>
</div>

<style global lang="scss">
  .modal-container {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    width: 100%;
    height: 100%;
    text-align: center;
    background-color: #57575788;
  }
  .modal {
    display: inline-block;
    width: 30%;
    padding: 10px;
    margin: 40vh auto;
    color: var(--text);
    text-align: center;
    background-color: var(--background);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  .modal h2 {
    margin-top: 0;
  }
</style>
