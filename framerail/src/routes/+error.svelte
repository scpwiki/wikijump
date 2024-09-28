<script lang="ts">
  import { page } from "$app/stores"
  import { goto, invalidateAll } from "$app/navigation"
  import { useErrorPopup } from "$lib/stores"
  import { Layout } from "$lib/types"
  import { parseDate } from "$lib/utils"
  let showErrorPopup = useErrorPopup()

  let showRestoreAction = false
  let deletedPages: Record<string, any>[] = []

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

  async function getDeleted() {
    let fdata = new FormData()
    fdata.set("site-id", $page.error.site.site_id)
    let res = await fetch(`/${$page.params.slug}/deleted-get`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message
      })
    } else {
      deletedPages = res
      showRestoreAction = true
    }
  }

  async function handleRestore() {
    let form = document.getElementById("page-restore")
    let fdata = new FormData(form)
    fdata.set("site-id", $page.error.site.site_id)
    let res = await fetch(`/${$page.params.slug}/restore`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message
      })
    } else {
      showRestoreAction = false
      invalidateAll()
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
      <input
        name="title"
        class="editor-title"
        placeholder={$page.error.internationalization?.title}
        type="text"
      />
      <input
        name="alt-title"
        class="editor-alt-title"
        placeholder={$page.error.internationalization?.["alt-title"]}
        type="text"
      />
      <textarea name="wikitext" class="editor-wikitext" />
      <input
        name="tags"
        class="editor-tags"
        placeholder={$page.error.internationalization?.tags}
        type="text"
      />
      <select name="layout" class="editor-layout">
        <option value={null}
          >{$page.error.internationalization?.["wiki-page-layout.default"]}</option
        >
        {#each Object.values(Layout) as layoutOption}
          <option value={layoutOption}
            >{$page.error.internationalization?.[
              `wiki-page-layout.${layoutOption}`
            ]}</option
          >
        {/each}
      </select>
      <textarea
        name="comments"
        class="editor-comments"
        placeholder={$page.error.internationalization?.["wiki-page-revision-comments"]}
      />
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

    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-restore clickable"
        type="button"
        on:click={getDeleted}
      >
        {$page.error.internationalization?.restore}
      </button>
    </div>

    {#if showRestoreAction}
      <form
        id="page-restore"
        class="page-restore"
        method="POST"
        on:submit|preventDefault={handleRestore}
      >
        <fieldset>
          <legend>{$page.error.internationalization?.["wiki-page-restore"]}</legend>
          {#each deletedPages as deletedPage}
            <input
              id={`restore-page-id-${deletedPage.page_id}`}
              name="page-id"
              class="page-restore-id"
              type="radio"
              value={deletedPage.page_id}
            />
            <label for={`restore-page-id-${deletedPage.page_id}`}>
              <span class="page-restore-title">{deletedPage.title}</span
              >{#if deletedPage.alt_title}&nbsp;-&nbsp;<span
                  class="page-restore-alt-title">{deletedPage.alt_title}</span
                >{/if} (<span class="page-restore-rating"
                >{(deletedPage.rating > 0 ? "+" : "") + deletedPage.rating}</span
              >) - {$page.error.internationalization?.["wiki-page-deleted"].replace(
                "{$datetime}",
                parseDate(deletedPage.page_deleted_at).toLocaleString()
              )}
            </label>
            <br />
          {/each}
        </fieldset>

        <textarea
          name="comments"
          class="page-restore-comments"
          placeholder={$page.error.internationalization?.["wiki-page-revision-comments"]}
        />

        <div class="action-row page-restore-actions">
          <button
            class="action-button page-restore-button button-cancel clickable"
            type="button"
            on:click|stopPropagation={() => {
              showRestoreAction = false
            }}
          >
            {$page.error.internationalization?.cancel}
          </button>
          <button
            class="action-button page-restore-button button-restore clickable"
            type="submit"
            on:click|stopPropagation
          >
            {$page.error.internationalization?.restore}
          </button>
        </div>
      </form>
    {/if}
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

  .editor,
  .page-restore {
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

  .editor-actions,
  .page-restore-actions {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: stretch;
    justify-content: flex-end;
    width: 100%;
  }
</style>
