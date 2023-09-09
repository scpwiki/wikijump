<script lang="ts">
  import { page } from "$app/stores"
</script>

<h1>UNTRANSLATED:Svelte Error</h1>

<p><textarea class="debug">{JSON.stringify($page)}</textarea></p>

<!--
Use svelte-switch-case package with {#switch data.view}
as soon as we can figure out prettier support for it.
-->
{#if $page.error.view === "pageMissing"}
  UNTRANSLATED:Page not found
  {@html $page.error.compiledHtml}
{:else if $page.error.view === "pagePermissions"}
  UNTRANSLATED:Lacks permissions for page
  {@html $page.error.compiledHtml}
{:else if $page.error.view === "siteMissing"}
  UNTRANSLATED:No such site
  {@html $page.error.html}
{:else}
  UNTRANSLATED:Fallback error, something really went wrong
{/if}
