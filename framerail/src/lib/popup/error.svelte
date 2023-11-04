<script lang="ts">
  export let exitPrompt: () => void
  import { onMount, onDestroy } from "svelte"
  import { useErrorPopup } from "$lib/stores"
  let showErrorPopup = useErrorPopup()
  function containerExitPrompt(e: Event) {
    if ((e.target as HTMLElement).classList.contains("modal-container")) exitPrompt()
  }
  const escKeydown = (e) => {
    if (e.code.toLowerCase() === "escape") exitPrompt()
  }
  onMount(() => {
    window.addEventListener("keydown", escKeydown)
  })
  onDestroy(() => {
    window.removeEventListener("keydown", escKeydown)
  })
</script>

<div class="modal-container" on:click={containerExitPrompt} on:keydown={escKeydown}>
  <div class="modal error-modal">
    <h2>UT: Error</h2>
    <div class="modal-message">
      {$showErrorPopup.message}
    </div>
  </div>
</div>

<style global lang="scss">
  .modal-container {
    background-color: #57575788;
    position: fixed;
    top: 0;
    bottom: 0;
    left: 0;
    right: 0;
    width: 100%;
    height: 100%;
    text-align: center;
  }
  .modal {
    display: inline-block;
    color: var(--text);
    background-color: var(--background);
    border: 1px solid var(--border);
    border-radius: 10px;
    width: 30%;
    padding: 10px;
    margin: 40vh auto;
    text-align: center;
  }
  .modal h2 {
    margin-top: 0;
  }
</style>
