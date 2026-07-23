<script lang="ts">
  import { deserialize } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"

  import type { PageProps } from "./$types"

  let { data }: PageProps = $props()

  let isLoggedIn = $derived<boolean>(data.isLoggedIn)

  async function tryLogout() {
    const res = await fetch(`?/logout`, {
      method: "POST",
      body: new FormData()
    }).then((res) => res.text())

    const result = deserialize<
      { success: boolean },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "success" && result.data?.success) {
      isLoggedIn = false
      await invalidateAll()
    } else if (result.type === "failure") {
      errorPopupState.current = {
        state: true,
        message: result.data?.message ?? null,
        data: result.data?.data
      }
    }
  }
</script>

{#if isLoggedIn}
  <div class="action-row auth-actions">
    <button
      class="action-button auth-button button-logout clickable"
      onclick={tryLogout}
      type="button"
    >
      {data.internationalization?.logout}
    </button>
  </div>
{:else}
  {data.internationalization?.["logout.toast"]}
{/if}
