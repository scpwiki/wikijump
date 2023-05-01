<script lang="ts">
  import { page } from "$app/stores"
</script>

<h1>UNTRANSLATED:Svelte Error</h1>

<!-- What's going on here? Every guide says the error() call
data appears in "page", but it's empty. I can't render an error
page without contextual information. -->
<textarea class="debug">{JSON.stringify(page)}</textarea>

<!--
Use svelte-switch-case package with {#switch data.view}
as soon as we can figure out prettier support for it.
-->
{#if page.error.view == "pageMissing"}
  UNTRANSLATED:Page not found
  {@html page.error.compiledHtml}
{:else if page.error.view == "pagePermissions"}
  UNTRANSLATED:Lacks permissions for page
  {@html page.error.compiledHtml}
{:else if page.error.view == "siteMissing"}
  UNTRANSLATED:No such site
  {@html page.error.html}
{:else}
  UNTRANSLATED:Fallback error, something really went wrong
{/if}
