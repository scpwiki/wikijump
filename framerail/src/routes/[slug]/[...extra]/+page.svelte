<script lang="ts">
  export let data
</script>

<!--
Use svelte-switch-case package with {#switch data.view}
as soon as we can figure out prettier support for it.
-->
{#if data.view == "pageFound"}
  UNTRANSLATED:Page view
  {@html data.compiledHtml}
{:else if data.view == "pageMissing"}
  UNTRANSLATED:Page not found
  {@html data.compiledHtml}
{:else if data.view == "pagePermissions"}
  UNTRANSLATED:Lacks permissions for page
  {@html data.compiledHtml}
{:else if data.view == "siteMissing"}
  UNTRANSLATED:No such site
  {@html data.html}
{/if}

<h1>UNTRANSLATED:Loaded slug page</h1>
<p class="spin-yay">UNTRANSLATED:This is some other page on the site</p>
<p>UNTRANSLATED:Response <textarea class="debug">{JSON.stringify(data)}</textarea></p>

<style global lang="scss">
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .spin-yay {
    display: inline-block;
    animation: spin 2s linear infinite;
  }

  .debug {
    width: 80vw;
    height: 60vh;
  }
</style>
