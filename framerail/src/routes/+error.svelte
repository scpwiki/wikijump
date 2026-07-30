<script lang="ts">
  import { page } from "$app/state"
  import { deserialize } from "$app/forms"
  import { resolve } from "$app/paths"
  import { goto, invalidateAll } from "$app/navigation"
  import { Layout } from "$lib/types"
  import { pageLayoutState, errorPopupState } from "$lib/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageData } from "./$types"
  import type { PageDeletedGet } from "$lib/server/deepwell/page"

  let errorData: PageData | null = $derived(page.error as unknown as PageData)

  let showRestoreAction = $state<boolean>(false)
  let deletedPages = $state<PageDeletedGet[]>([])

  function cancelCreate() {
    goto(resolve(`/${page.params.slug}`, {}), {
      noScroll: true
    })
  }

  const { form: editForm, enhance: editEnhance } = superForm(
    untrack(() => errorData.forms.pageEditForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$editForm,
          siteId: page.data.site.site_id,
          slug: page.params.slug ?? page.error?.site.default_page
        }
        jsonData(submitForm)
      },
      onResult: async ({ result, cancel }) => {
        if (result.type === "success" && result.data) {
          cancel()
          goto(resolve(`/${page.params.slug ?? page.data.site.default_page}`, {}), {
            noScroll: true
          })
        }
        if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data.message,
            data: result.data.data
          }
        }
      }
    }
  )

  async function getDeleted() {
    const res = await fetch(`?/deletedGet`, {
      method: "POST",
      body: JSON.stringify({
        siteId: page.data.site.site_id,
        slug: page.params.slug ?? page.data.site.default_page
      })
    }).then((res) => res.text())
    const result = deserialize<
      { res: PageDeletedGet[] },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)
    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data?.message,
        data: result.data?.data
      }
    } else if (result.type === "success" && result.data?.res) {
      deletedPages = result.data.res
      showRestoreAction = true
    }
  }

  const { form: restoreForm, enhance: restoreEnhance } = superForm(
    untrack(() => errorData.forms.pageRestoreForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$restoreForm,
          siteId: page.data.site.site_id
        }
        jsonData(submitForm)
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          showRestoreAction = false
          invalidateAll()
        }
        if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data.message,
            data: result.data.data
          }
        }
      }
    }
  )
</script>

<h1>UNTRANSLATED:Svelte Error</h1>

<p><textarea class="debug">{JSON.stringify(page, null, 2)}</textarea></p>

{#if errorData.view === "missing"}
  UNTRANSLATED:Page not found

  {#if errorData.options?.edit}
    {#if pageLayoutState.current === Layout.WIKIDOT}
      <h1 class="page-create-header">
        {errorData.internationalization?.["wiki-page-create"]}
      </h1>
    {:else}
      <h2 class="page-create-header">
        {errorData.internationalization?.["wiki-page-create"]}
      </h2>
    {/if}

    <form id="editor" class="editor" action="?/edit" method="POST" use:editEnhance>
      <input
        name="title"
        class="editor-title"
        placeholder={errorData.internationalization?.title}
        type="text"
        bind:value={$editForm.title}
      />
      <input
        name="altTitle"
        class="editor-alt-title"
        placeholder={errorData.internationalization?.["alt-title"]}
        type="text"
        bind:value={$editForm.altTitle}
      />
      <textarea name="wikitext" class="editor-wikitext" bind:value={$editForm.wikitext}
      ></textarea>
      <input
        name="tags"
        class="editor-tags"
        placeholder={errorData.internationalization?.tags}
        type="text"
        bind:value={$editForm.tags}
      />
      <select name="layout" class="editor-layout" bind:value={$editForm.layout}>
        <option value={null}>
          {errorData.internationalization?.["wiki-page-layout.default"]}
        </option>
        {#each Object.values(Layout) as layoutOption (layoutOption.toString())}
          <option value={layoutOption}>
            {errorData.internationalization?.[`wiki-page-layout.${layoutOption}`]}
          </option>
        {/each}
      </select>
      <textarea
        name="comments"
        class="editor-comments"
        placeholder={errorData.internationalization?.["wiki-page-revision-comments"]}
        bind:value={$editForm.comments}></textarea>
      {#if pageLayoutState.current === Layout.WIKIDOT}
        <div class="buttons">
          <input
            class="btn btn-danger"
            onclick={cancelCreate}
            type="button"
            value={errorData.internationalization?.cancel}
          />
          <input
            class="btn btn-primary"
            type="submit"
            value={errorData.internationalization?.save}
          />
        </div>
      {:else}
        <div class="action-row editor-actions">
          <button
            class="action-button editor-button button-cancel clickable"
            onclick={cancelCreate}
            type="button"
          >
            {errorData.internationalization?.cancel}
          </button>
          <button class="action-button editor-button button-save clickable" type="submit">
            {errorData.internationalization?.save}
          </button>
        </div>
      {/if}
    </form>
  {:else}
    <div id="page-content">
      {@html errorData.compiled_body_html}
    </div>

    {#if pageLayoutState.current === Layout.WIKIDOT}
      <div id="page-options-container">
        <div id="page-options-bottom" class="page-options-bottom">
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a
            id="restore-button"
            class="btn btn-default"
            href="javascript:;"
            onclick={getDeleted}
            type="button"
          >
            {errorData.internationalization?.restore}
          </a>
        </div>
      </div>
    {:else}
      <div class="action-row editor-actions">
        <button
          class="action-button editor-button button-restore clickable"
          onclick={getDeleted}
          type="button"
        >
          {errorData.internationalization?.restore}
        </button>
      </div>
    {/if}

    {#if showRestoreAction}
      {#if pageLayoutState.current === Layout.WIKIDOT}
        <div id="action-area">
          <h1 class="page-restore-header">
            {errorData.internationalization?.["wiki-page-restore"]}
          </h1>

          <form
            id="page-restore"
            class="page-restore"
            action="?/restore"
            method="POST"
            use:restoreEnhance
          >
            <fieldset>
              <legend>
                {errorData.internationalization?.["wiki-page-restore.select"]}
              </legend>
              {#each deletedPages as deletedPage (deletedPage.page_id)}
                <input
                  id={`restore-page-id-${deletedPage.page_id}`}
                  name="pageId"
                  class="page-restore-id"
                  checked={$restoreForm.pageId === deletedPage.page_id}
                  type="radio"
                  value={deletedPage.page_id}
                />
                <label for={`restore-page-id-${deletedPage.page_id}`}>
                  <span class="page-restore-title">{deletedPage.title}</span>
                  {#if deletedPage.alt_title}
                    &nbsp;-&nbsp;
                    <span class="page-restore-alt-title">
                      {deletedPage.alt_title}
                    </span>
                  {/if} (
                  <span class="page-restore-rating">
                    {(deletedPage.rating > 0 ? "+" : "") + deletedPage.rating}
                  </span>
                  ) - {errorData.internationalization?.["wiki-page-deleted"]?.replace(
                    "{$datetime}",
                    new Date(deletedPage.page_deleted_at).toLocaleString()
                  )}
                </label>
                <br />
              {/each}
            </fieldset>

            <textarea
              name="comments"
              class="page-restore-comments"
              placeholder={errorData.internationalization?.[
                "wiki-page-revision-comments"
              ]}
              bind:value={$restoreForm.comments}></textarea>

            <div class="buttons">
              <input
                class="btn btn-primary"
                onclick={() => (showRestoreAction = false)}
                type="button"
                value={errorData.internationalization?.cancel}
              />
              <input
                class="btn btn-primary"
                type="submit"
                value={errorData.internationalization?.restore}
              />
            </div>
          </form>
        </div>
      {:else}
        <h2 class="page-restore-header">
          {errorData.internationalization?.["wiki-page-restore"]}
        </h2>

        <form
          id="page-restore"
          class="page-restore"
          action="?/restore"
          method="POST"
          use:restoreEnhance
        >
          <fieldset>
            <legend>
              {errorData.internationalization?.["wiki-page-restore.select"]}
            </legend>
            {#each deletedPages as deletedPage (deletedPage.page_id)}
              <input
                id={`restore-page-id-${deletedPage.page_id}`}
                name="pageId"
                class="page-restore-id"
                checked={$restoreForm.pageId === deletedPage.page_id}
                type="radio"
                value={deletedPage.page_id}
                bind:group={$restoreForm.pageId}
              />
              <label for={`restore-page-id-${deletedPage.page_id}`}>
                <span class="page-restore-title">{deletedPage.title}</span
                >{#if deletedPage.alt_title}&nbsp;-&nbsp;<span
                    class="page-restore-alt-title">{deletedPage.alt_title}</span
                  >{/if} (<span class="page-restore-rating"
                  >{(deletedPage.rating > 0 ? "+" : "") + deletedPage.rating}</span
                >) - {errorData.internationalization?.["wiki-page-deleted"]?.replace(
                  "{$datetime}",
                  new Date(deletedPage.page_deleted_at).toLocaleString()
                )}
              </label>
              <br />
            {/each}
          </fieldset>

          <textarea
            name="comments"
            class="page-restore-comments"
            placeholder={errorData.internationalization?.["wiki-page-revision-comments"]}
            bind:value={$restoreForm.comments}></textarea>

          <div class="action-row page-restore-actions">
            <button
              class="action-button page-restore-button button-cancel clickable"
              onclick={() => (showRestoreAction = false)}
              type="button"
            >
              {errorData.internationalization?.cancel}
            </button>
            <button
              class="action-button page-restore-button button-restore clickable"
              type="submit"
            >
              {errorData.internationalization?.restore}
            </button>
          </div>
        </form>
      {/if}
    {/if}
  {/if}
{:else if errorData.view === "permissions"}
  UNTRANSLATED:Lacks permissions for page
  {@html errorData.compiled_body_html}
{:else}
  UNTRANSLATED:Fatal error: Unable to display view
{/if}

<style global lang="scss">
  .debug {
    width: 100%;
    height: 60vh;
  }

  .editor,
  .page-restore {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 100%;
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
