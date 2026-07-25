<script lang="ts">
  import { page } from "$app/state"

  type AdminErrorData = NonNullable<typeof page.error> & { html?: string }

  let errorData: AdminErrorData = $derived(page.error as AdminErrorData)
</script>

<h1>UNTRANSLATED:Svelte Error</h1>

<p><textarea class="debug">{JSON.stringify(page, null, 2)}</textarea></p>

<!--
Use svelte-switch-case package with {#switch data.view}
as soon as we can figure out prettier support for it.
-->
{#if errorData.view === "admin_permissions"}
  UNTRANSLATED:Lacks permissions for page
  {@html errorData.html ?? ""}
{:else}
  UNTRANSLATED:Fatal error: Unable to display view
{/if}

<style global lang="scss">
  .debug {
    width: 100%;
    height: 60vh;
  }
</style>
