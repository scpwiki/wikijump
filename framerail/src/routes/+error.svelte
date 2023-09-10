<script lang="ts">
  import { page } from "$app/stores"
  import { goto } from "$app/navigation"

  function cancelEdit() {
    goto(`/${$page.params.slug}`, {
      noScroll: true,
    })
  }

  async function saveEdit() {
    let form = document.getElementById("editor")
    let fdata = new FormData(form)
    fdata.set("site-id", $page.error.site.siteId)
    fdata.set("slug", $page.params.slug)
    await fetch(`/${$page.params.slug}`, {
      method: "POST",
      body: fdata,
    })
    goto(`/${$page.params.slug}`, {
      noScroll: true,
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
    <form
      id="editor"
      class="editor"
      method="POST"
      on:submit|preventDefault={saveEdit} >
      <input
        type="text"
        class="editor-title"
        name="title"
        placeholder="title"
        />
      <input
        type="text"
        class="editor-alt-title"
        name="alt-title"
        placeholder="alternative title"
        />
      <textarea class="editor-wikitext" name="wikitext"></textarea>
      <input
        type="text"
        class="editor-tags"
        name="tags"
        placeholder="tags"
        />
      <div class="editor-actions">
        <button
          type="button"
          class="editor-button button-cancel clickable"
          on:click|stopPropagation={cancelEdit} >
          UT:Cancel
        </button>
        <button
          type="submit"
          class="editor-button button-save clickable"
          on:click|stopPropagation >
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

.page-content,
.page-tags-container {
  padding: 0 0 2em;
}

.page-tags {
  display: flex;
  flex-direction: row;
  flex-wrap: wrap;
  justify-content: flex-start;
  align-items: center;
  gap: 10px;
  padding: 0;
  margin: 0;
  list-style: none;
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
