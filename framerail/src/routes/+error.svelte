<script lang="ts">
  import { page } from "$app/stores"
  import { goto } from "$app/navigation"
  import { useErrorPopup } from "$lib/stores"
  let showErrorPopup = useErrorPopup()

  function cancelCreate() {
    goto(`/${$page.params.slug}`, {
      noScroll: true
    })
  }

  async function saveCreate() {
    let form = document.getElementById("editor")
    let fdata = new FormData(form)
    fdata.set("site-id", $page.error.site.site_id)
    fdata.set("slug", $page.params.slug)
    let res = await fetch(`/${$page.params.slug}/edit`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message
      })
    } else {
      goto(`/${$page.params.slug}`, {
        noScroll: true
      })
    }
  }
</script>

<h1>UNTRANSLATED:Svelte Error</h1>

<p><textarea class="debug">{JSON.stringify($page, null, 2)}</textarea></p>

<!--
Use svelte-switch-case package with {#switch data.view}
as soon as we can figure out prettier support for it.
-->
{#if $page.error.view === "page_missing"}
  UNTRANSLATED:Page not found

  {#if $page.error.options?.edit}
    <form id="editor" class="editor" method="POST" on:submit|preventDefault={saveCreate}>
      <input name="title" class="editor-title" placeholder="title" type="text" />
      <input
        name="alt-title"
        class="editor-alt-title"
        placeholder="alternative title"
        type="text"
      />
      <textarea name="wikitext" class="editor-wikitext" />
      <input name="tags" class="editor-tags" placeholder="tags" type="text" />
      <textarea name="comments" class="editor-comments" placeholder="comments" />
      <div class="action-row editor-actions">
        <button
          class="action-button editor-button button-cancel clickable"
          type="button"
          on:click|stopPropagation={cancelCreate}
        >
          {$page.error.internationalization?.cancel}
        </button>
        <button
          class="action-button editor-button button-save clickable"
          type="submit"
          on:click|stopPropagation
        >
          {$page.error.internationalization?.save}
        </button>
      </div>
    </form>
  {:else}
    {@html $page.error.compiled_html}
  {/if}
{:else if $page.error.view === "page_permissions"}
  UNTRANSLATED:Lacks permissions for page
  {@html $page.error.compiled_html}
{:else if $page.error.view === "site_missing"}
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
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 80vw;
  }

  .editor-wikitext {
    height: 60vh;
  }

  .editor-actions {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: stretch;
    justify-content: flex-end;
    width: 100%;
  }
</style>
