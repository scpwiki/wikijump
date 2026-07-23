<script lang="ts">
  import { page } from "$app/state"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { publicErrorExtraMessage } from "$lib/popup/public-error-data.js"
  import { Layout } from "$lib/types"

  interface Props {
    exitPrompt: () => void
  }
  let { exitPrompt }: Props = $props()

  const pageLayoutContext = getPageLayoutContext()

  function containerExitPrompt(event: MouseEvent) {
    event.preventDefault()
    const classList = (event.target as HTMLElement).classList
    if (
      classList.contains("modal-container") ||
      classList.contains("odialog-container") ||
      classList.contains("odialog-shader") ||
      classList.contains("button-close-message")
    ) {
      exitPrompt()
    }
  }
  const escKeydown = (event: KeyboardEvent) => {
    if (event.code.toLowerCase() === "escape") exitPrompt()
  }

  const errorExtraMessage = $derived(
    publicErrorExtraMessage(errorPopupState.current.data)
  )
  $effect(() => {
    window.addEventListener("keydown", escKeydown)
    return () => window.removeEventListener("keydown", escKeydown)
  })
</script>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  <div
    id="odialog-shader"
    class="odialog-shader"
    onclick={containerExitPrompt}
    onkeydown={escKeydown}
    role="presentation"
  ></div>
  <div
    id="odialog-container"
    style:display="block"
    class="odialog-container"
    onclick={containerExitPrompt}
    onkeydown={escKeydown}
    role="presentation"
  >
    <div id="owindow-1" class="owindow error">
      <div class="title modal-header">
        {page.data?.internationalization?.error ??
          page.error?.internationalization?.error}
      </div>
      <div class="content modal-body">
        <h1 id="modal-title">
          {errorPopupState.current.message}
        </h1>
        {#if errorExtraMessage}
          <p id="model-message-extra" class="modal-message-extra">
            {errorExtraMessage}
          </p>
        {/if}
      </div>
      <div class="button-bar modal-footer">
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          class="btn btn-default button button-close-message"
          href="javascript:;"
          onclick={containerExitPrompt}
          onkeydown={escKeydown}
          role="button"
          tabindex="0"
          >{page.data?.internationalization?.close ??
            page.error?.internationalization?.close}</a
        >
      </div>
    </div>
  </div>
{:else}
  <div
    class="modal-container"
    aria-describedby="modal-message"
    aria-labelledby="modal-title"
    onclick={containerExitPrompt}
    onkeydown={escKeydown}
    role="presentation"
  >
    <div class="modal error-modal">
      <h2 id="modal-title">
        {page.data?.internationalization?.error ??
          page.error?.internationalization?.error}
      </h2>
      <div id="modal-message" class="modal-message">
        {errorPopupState.current.message}
      </div>
      {#if errorExtraMessage}
        <div id="model-message-extra" class="modal-message-extra">
          {errorExtraMessage}
        </div>
      {/if}
    </div>
  </div>
{/if}

<style global lang="scss">
  #odialog-container.odialog-container {
    position: fixed;
    display: flex !important;
    align-items: center;
    justify-content: center;
  }
  .modal-container {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    z-index: 100;
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
  .modal-message-extra {
    white-space: pre-wrap;
  }
</style>
