<script lang="ts">
  import { goto } from "$app/navigation"
  import { errorPopupState, pageLayoutState } from "$lib/stores.svelte"
  import { Layout, PagePane } from "$lib/types"
  import { resolve } from "$app/paths"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { pagePaneState = $bindable(), data }: PageProps & { pagePaneState: PagePane } =
    $props()

  const { form, enhance } = superForm(
    untrack(() => data.forms.pageMoveForm),
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
          cancel()
          goto(resolve(`/${result.data.res.new_slug}`, {}), {
            noScroll: true
          })
          pagePaneState = PagePane.None
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

{#if pageLayoutState.current === Layout.WIKIDOT}
  <h1 class="page-move-header">
    {data.internationalization?.["wiki-page-move"]}
  </h1>
{:else}
  <h2 class="page-move-header">
    {data.internationalization?.["wiki-page-move"]}
  </h2>
{/if}

<form id="page-move" class="page-move" action="?/move" method="POST" use:enhance>
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
    bind:value={$form.comments}></textarea>
  {#if pageLayoutState.current === Layout.WIKIDOT}
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
        value={data.internationalization?.move}
      />
    </div>
  {:else}
    <div class="action-row page-move-actions">
      <button
        class="action-button page-move-button button-cancel clickable"
        onclick={() => (pagePaneState = PagePane.None)}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button class="action-button page-move-button button-move clickable" type="submit">
        {data.internationalization?.move}
      </button>
    </div>
  {/if}
</form>

<style lang="scss">
  .page-move {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 100%;
    padding: 0 0 2em;
  }
</style>
