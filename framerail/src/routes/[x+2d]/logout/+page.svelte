<script lang="ts">
  import { page } from "$app/stores"
  import { invalidateAll } from "$app/navigation"

  let isLoggedIn = $page.data.isLoggedIn

  async function tryLogout() {
    let res = await fetch(`/-/logout`, {
      method: "DELETE"
    }).then((res) => res.ok)

    if (res) {
      isLoggedIn = false
      invalidateAll()
    }
  }
</script>

{#if isLoggedIn}
  <div class="action-row auth-actions">
    <button
      class="action-button auth-button button-logout clickable"
      type="button"
      on:click={tryLogout}
    >
      {$page.data.internationalization?.logout}
    </button>
  </div>
{:else}
  {$page.data.internationalization?.["logout.toast"]}
{/if}
