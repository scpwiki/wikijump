<script lang="ts">
  import { goto, invalidateAll } from "$app/navigation"
  import { errorPopupState, pageLayoutState } from "$lib/stores.svelte"
  import { DeleteOptions, Layout, PagePane } from "$lib/types"
  import { resolve } from "$app/paths"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { pagePaneState = $bindable(), data }: PageProps & { pagePaneState: PagePane } =
    $props()

  const { form, enhance } = superForm(
    untrack(() => data.forms.pageDeleteForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$form,
          siteId: data.site.site_id,
          pageId: data.page?.page_id,
          lastRevisionId: data.page_revision?.revision_id
        }
        jsonData(submitForm)
      },
      onResult: async ({ result, cancel }) => {
        if (result.type === "success" && result.data) {
          if (result.data.option === DeleteOptions.Move) {
            cancel()
            goto(resolve(`/${result.data.res.new_slug}`, {}), {
              noScroll: true
            })
            pagePaneState = PagePane.None
          } else if (result.data.option === DeleteOptions.Delete) {
            pagePaneState = PagePane.None
            invalidateAll()
          }
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

  $form.option = DeleteOptions.Move
  if ($form.option === DeleteOptions.Move) {
    $form.newSlug = `deleted:${untrack(() => data.page?.slug)}`
  }
</script>

{#if pageLayoutState.current === Layout.WIKIDOT}
  <h1 class="page-delete-header">
    {data.internationalization?.["wiki-page-delete"]}
  </h1>
{:else}
  <h2 class="page-delete-header">
    {data.internationalization?.["wiki-page-delete"]}
  </h2>
{/if}

<form id="page-delete" class="page-delete" action="?/delete" method="POST" use:enhance>
  <div>
    <input
      id="page-delete-option-move"
      name="option"
      type="radio"
      value={DeleteOptions.Move}
      bind:group={$form.option}
    />
    <label class="page-delete-option option-move" for="page-delete-option-move">
      {data.internationalization?.["wiki-page-move"]}
    </label>
  </div>
  <div>
    <input
      id="page-delete-option-delete"
      name="option"
      type="radio"
      value={DeleteOptions.Delete}
      bind:group={$form.option}
    />
    <label class="page-delete-option option-delete" for="page-delete-option-delete">
      {data.internationalization?.["wiki-page-delete"]}
    </label>
  </div>

  {#if pageLayoutState.current === Layout.WIKIDOT}
    {#if $form.option === DeleteOptions.Move}
      <input
        name="new-slug"
        class="page-move-new-slug"
        placeholder={data.internationalization?.["wiki-page-move.new-slug"]}
        type="text"
        bind:value={$form.newSlug}
      />
      <textarea
        name="comments"
        class="page-move-comments"
        placeholder={data.internationalization?.["wiki-page-revision-comments"]}
        bind:value={$form.comments}
      ></textarea>
    {/if}
    <div class="buttons">
      <input
        class="btn btn-danger"
        onclick={() => (pagePaneState = PagePane.None)}
        type="button"
        value={data.internationalization?.cancel}
      />
      <input
        class="btn btn-primary"
        type="submit"
        value={data.internationalization?.confirm}
      />
    </div>
  {:else}
    {#if $form.option === DeleteOptions.Move}
      <input
        name="new-slug"
        class="page-move-new-slug"
        placeholder={data.internationalization?.["wiki-page-move.new-slug"]}
        type="text"
        bind:value={$form.newSlug}
      />
    {/if}
    <textarea
      name="comments"
      class="page-move-comments"
      placeholder={data.internationalization?.["wiki-page-revision-comments"]}
      bind:value={$form.comments}
    ></textarea>
    <div class="action-row page-delete-actions">
      <button
        class="action-button page-delete-button button-cancel clickable"
        onclick={() => (pagePaneState = PagePane.None)}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button
        class="action-button page-delete-button button-confirm clickable"
        type="submit"
      >
        {data.internationalization?.confirm}
      </button>
    </div>
  {/if}
</form>

<style lang="scss">
  .page-delete {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 100%;
    padding: 0 0 2em;
  }
</style>
