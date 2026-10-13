<script lang="ts">
  import { page } from "$app/state"

  import type { TranslatedKeys } from "$lib/types"

  type AdminErrorData = NonNullable<typeof page.error> & {
    html?: string
    internationalization?: TranslatedKeys
  }

  let errorData: AdminErrorData = $derived(page.error as AdminErrorData)
</script>

<h1>{errorData?.internationalization?.["error"]}</h1>

<p><textarea class="debug">{JSON.stringify(page, null, 2)}</textarea></p>

<!--
Use svelte-switch-case package with {#switch data.view}
as soon as we can figure out prettier support for it.
-->
{#if errorData.view === "admin_permissions"}
  {errorData.internationalization?.["admin-unauthorized"]}
  {@html errorData.html ?? ""}
{:else}
  {errorData.internationalization?.["error-api.GENERIC"]}
{/if}

<style global lang="scss">
  .debug {
    width: 100%;
    height: 60vh;
  }
</style>
