<script lang="ts">
  import { goto } from "$app/navigation"
  import { errorPopupState, pageLayoutState } from "$lib/stores.svelte"
  import { Layout } from "$lib/types"
  import { resolve } from "$app/paths"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data, params }: PageProps = $props()

  function cancelEdit() {
    const options: string[] = Object.entries({
      norender: data.options.no_render,
      noredirect: data.options.no_redirect
    })
      .filter(([, enabled]) => enabled)
      .map(([key]) => `/${key}`)

    goto(resolve(`/${params.slug}${options.join("")}`, {}), {
      noScroll: true
    })
  }

  const { form, enhance } = superForm(
    untrack(() => data.forms.pageEditForm),
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
          goto(resolve(`/${params.slug}`, {}), {
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

  $form.title = untrack(() => data.page_revision?.title ?? "")
  $form.altTitle = untrack(() => data.page_revision?.alt_title ?? "")
  $form.wikitext = untrack(() => data.wikitext)
  $form.tags = untrack(() => data.page_revision?.tags?.join(" ") ?? "")
  $form.comments = untrack(() => data.page_revision?.comments ?? "")
</script>

{#if pageLayoutState.current === Layout.WIKIDOT}
  <h1 class="page-edit-header">
    {data.internationalization?.["wiki-page-edit"]}
  </h1>
{:else}
  <h2 class="page-edit-header">
    {data.internationalization?.["wiki-page-edit"]}
  </h2>
{/if}

<form id="editor" class="editor" action="?/edit" method="POST" use:enhance>
  <input
    name="title"
    class="editor-title"
    placeholder={data.internationalization?.title}
    type="text"
    bind:value={$form.title}
  />
  <input
    name="altTitle"
    class="editor-alt-title"
    placeholder={data.internationalization?.["alt-title"]}
    type="text"
    bind:value={$form.altTitle}
  />
  <textarea name="wikitext" class="editor-wikitext" bind:value={$form.wikitext}
  ></textarea>
  <input
    name="tags"
    class="editor-tags"
    placeholder={data.internationalization?.tags}
    type="text"
    bind:value={$form.tags}
  />
  <textarea
    name="comments"
    class="editor-comments"
    placeholder={data.internationalization?.["wiki-page-revision-comments"]}
    bind:value={$form.comments}
  ></textarea>
  {#if pageLayoutState.current === Layout.WIKIDOT}
    <div class="buttons alignleft">
      <input
        name="cancel"
        class="btn btn-danger"
        onclick={cancelEdit}
        type="button"
        value={data.internationalization?.cancel}
      />
      <input
        name="save"
        class="btn btn-primary"
        type="submit"
        value={data.internationalization?.save}
      />
    </div>
  {:else}
    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-cancel clickable"
        onclick={cancelEdit}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button class="action-button editor-button button-save clickable" type="submit">
        {data.internationalization?.save}
      </button>
    </div>
  {/if}
</form>

<style lang="scss">
  .editor-actions {
    padding: 0 0 2em;
  }

  .editor {
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
</style>
