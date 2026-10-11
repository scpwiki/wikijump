<script lang="ts">
  import { page } from "$app/state"
  import { deserialize } from "$app/forms"
  import { resolve } from "$app/paths"
  import { goto, invalidateAll } from "$app/navigation"
  import { Layout } from "$lib/types"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { pageMutationDestinationSlug } from "$lib/page-mutation-destination"
  import { createPageDeniedMessage } from "$lib/wikidot/wikidot-locale"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"
  import type { PageErrorData } from "$lib/page-error-data"

  import DataFormEditor from "./[slug]/[...extra]/DataFormEditor.svelte"

  import type { PageDeletedGet } from "$lib/server/deepwell/page"
  let { errorData }: { errorData: PageErrorData } = $props()

  const pageLayoutContext = getPageLayoutContext()

  let showRestoreAction = $state<boolean>(false)
  let deletedPages = $state<PageDeletedGet[]>([])

  function cancelCreate() {
    goto(resolve(`/${page.params.slug}`, {}), {
      noScroll: true
    })
  }

  // The create decision comes from Deepwell's missing view. A denied viewer gets
  // the permission dialog on the missing page and no editor, and never navigates.
  const createDenied = (): boolean => errorData.can_create !== true

  function showCreateDenied() {
    errorPopupState.current = {
      state: true,
      message: createPageDeniedMessage(),
      data: null
    }
  }

  function startCreate(event: MouseEvent) {
    event.preventDefault()
    if (createDenied()) {
      showCreateDenied()
      return
    }
    goto(resolve(`/${missingPageSlug}/edit/true`, {}), {
      noScroll: true
    })
  }

  $effect(() => {
    if (errorData.view === "missing" && errorData.options?.edit && createDenied()) {
      showCreateDenied()
    }
  })

  const { form: editForm, enhance: editEnhance } = superForm(
    untrack(() => errorData.forms.pageEditForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$editForm,
          siteId: errorData.site.site_id,
          slug: page.params.slug ?? page.error?.site.default_page
        }
        jsonData(submitForm)
      },
      onResult: async ({ result, cancel }) => {
        if (result.type === "success" && result.data) {
          cancel()
          const destinationSlug = pageMutationDestinationSlug({
            creating: true,
            requestedSlug: missingPageSlug,
            responseSlug: result.data.res?.slug
          })
          if (!destinationSlug) {
            errorPopupState.current = {
              state: true,
              message: "Page creation did not return its assigned slug.",
              data: {}
            }
            return
          }
          goto(resolve(`/${destinationSlug}`, {}), {
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

  let activeTemplatePageId = $state<number | null>(
    untrack(() => errorData.selected_template_page_id ?? null)
  )
  let selectedTemplatePageId = $state<number | null>(
    untrack(() => errorData.selected_template_page_id ?? null)
  )
  let activeTemplateSource = $state<string>(untrack(() => $editForm.wikitext))
  const missingPageSlug = $derived(
    page.params.slug ?? page.error?.site.default_page ?? ""
  )
  const missingPageName = $derived(
    missingPageSlug.includes(":")
      ? missingPageSlug.slice(missingPageSlug.indexOf(":") + 1)
      : missingPageSlug
  )
  const missingPageTitle = $derived(
    missingPageName.charAt(0).toUpperCase() + missingPageName.slice(1)
  )

  function handlePageTemplateChange() {
    if (
      $editForm.wikitext !== activeTemplateSource &&
      // Wikidot uses the browser's native confirmation before replacing edited source.
      // eslint-disable-next-line no-alert
      !globalThis.confirm(
        "It seems you have already changed the page.\nChanging the initial template now will reset the edited page.\nDo you want to change the initial content?"
      )
    ) {
      selectedTemplatePageId = activeTemplatePageId
      return
    }

    const template = errorData.page_templates?.find(
      (candidate) => candidate.page_id === selectedTemplatePageId
    )
    activeTemplatePageId = selectedTemplatePageId
    activeTemplateSource = template?.wikitext ?? ""
    $editForm.wikitext = activeTemplateSource
  }

  async function getDeleted() {
    const res = await fetch(`?/deletedGet`, {
      method: "POST",
      body: ""
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

{#if errorData.view === "missing"}
  {#if errorData.options?.edit && !createDenied()}
    {#if errorData.data_form}
      <div id="action-area">
        {#key `create:${missingPageSlug}`}
          <DataFormEditor
            creating={true}
            definition={errorData.data_form.definition}
            editForm={errorData.forms.pageEditForm}
            initialParent={errorData.options.parent ?? ""}
            initialSource=""
            initialTags={errorData.options.tags ?? ""}
            initialTitle={missingPageTitle}
            initialValues={errorData.data_form.values}
            pagepaths={errorData.data_form.pagepaths}
            siteId={errorData.site.site_id}
            slug={missingPageSlug}
          />
        {/key}
      </div>
    {:else}
      {#if pageLayoutContext.current === Layout.WIKIDOT}
        <h1 class="page-create-header">
          {errorData.internationalization?.["wiki-page-create"]}
        </h1>
      {:else}
        <h2 class="page-create-header">
          {errorData.internationalization?.["wiki-page-create"]}
        </h2>
      {/if}

      <form id="editor" class="editor" action="?/edit" method="POST" use:editEnhance>
        {#if errorData.page_templates?.length}
          <div class="page-template-selector">
            <label for="page-templates">Initial template:</label>
            <select
              id="page-templates"
              onchange={handlePageTemplateChange}
              bind:value={selectedTemplatePageId}
            >
              <option value={null}>no template</option>
              {#each errorData.page_templates as template (template.page_id)}
                <option value={template.page_id}>{template.title}</option>
              {/each}
            </select>
          </div>
        {/if}
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
        {#if pageLayoutContext.current === Layout.WIKIDOT}
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
            <button
              class="action-button editor-button button-save clickable"
              type="submit"
            >
              {errorData.internationalization?.save}
            </button>
          </div>
        {/if}
      </form>
    {/if}
  {:else}
    <div id="page-content">
      <p id="404-message">
        The page <em>{missingPageSlug}</em> you want to access does not exist.
      </p>
      <ul id="create-it-now-link">
        <li>
          <a href={resolve(`/${missingPageSlug}/edit/true`, {})} onclick={startCreate}
            >Create page</a
          >
        </li>
      </ul>
      {@html errorData.compiled_body_html}
    </div>

    {#if pageLayoutContext.current === Layout.WIKIDOT}
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
      {#if pageLayoutContext.current === Layout.WIKIDOT}
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
