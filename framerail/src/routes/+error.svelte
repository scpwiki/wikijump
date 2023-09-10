<script lang="ts">
  import { page } from "$app/stores"
  import { goto } from "$app/navigation"

  function cancelEdit() {
    goto(`/${$page.params.slug}`, {
      noScroll: true
    })
  }

  async function saveEdit() {
    let form = document.getElementById("editor")
    let fdata = new FormData(form)
    fdata.set("site-id", $page.error.site.siteId)
    fdata.set("slug", $page.params.slug)
    await fetch(`/${$page.params.slug}`, {
      method: "POST",
      body: fdata
    })
    goto(`/${$page.params.slug}`, {
      noScroll: true
    })
  }
</script>

<h1>UNTRANSLATED:Svelte Error</h1>

<p><textarea class="debug">{JSON.stringify($page, null, 2)}</textarea></p>

<!--
Use svelte-switch-case package with {#switch data.view}
as soon as we can figure out prettier support for it.
-->
{#if $page.error.view === "pageMissing"}
  UNTRANSLATED:Page not found

  {#if $page.error.options?.edit}
    <form id="editor" class="editor" method="POST" on:submit|preventDefault={saveEdit}>
      <input name="title" class="editor-title" placeholder="title" type="text" />
      <input
        name="alt-title"
        class="editor-alt-title"
        placeholder="alternative title"
        type="text"
      />
      <textarea name="wikitext" class="editor-wikitext" />
      <input name="tags" class="editor-tags" placeholder="tags" type="text" />
      <div class="editor-actions">
        <button
          class="editor-button button-cancel clickable"
          type="button"
          on:click|stopPropagation={cancelEdit}
        >
          UT:Cancel
        </button>
        <button
          class="editor-button button-save clickable"
          type="submit"
          on:click|stopPropagation
        >
          UT:Save
        </button>
      </div>
    </form>
  {:else}
    {@html $page.error.compiledHtml}
  {/if}
{:else if $page.error.view === "pagePermissions"}
  UNTRANSLATED:Lacks permissions for page
  {@html $page.error.compiledHtml}
{:else if $page.error.view === "siteMissing"}
  UNTRANSLATED:No such site
  {@html $page.error.html}
{:else}
  UNTRANSLATED:Fallback error, something really went wrong
{/if}

<style global lang="scss">
  .debug {
    width: 80vw;
    height: 60vh;
  }

  .editor {
    width: 80vw;
    display: flex;
    flex-direction: column;
    justify-content: stretch;
    align-items: stretch;
    gap: 15px;
  }

  .editor-wikitext {
    height: 60vh;
  }

  .editor-actions {
    width: 100%;
    display: flex;
    flex-direction: row;
    justify-content: flex-end;
    align-items: stretch;
    gap: 10px;
  }
</style>
