<script lang="ts">
  import { deserialize } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { Layout, PagePane } from "$lib/types"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let pageParents = $state<string>("")

  let { pagePaneState = $bindable(), data }: PageProps & { pagePaneState: PagePane } =
    $props()

  const pageLayoutContext = getPageLayoutContext()

  const { form, enhance } = superForm(
    untrack(() => data.forms.pageParentForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const { parents: formParents } = $form
        const newParents = formParents.split(" ").filter((p) => p)
        const oldParents = pageParents.split(" ").filter((p) => p)
        const removed: string[] = oldParents.filter((p) => !newParents.includes(p))
        const common: string[] = oldParents.filter((p) => newParents.includes(p))
        const added: string[] = newParents.filter((p) => !common.includes(p))

        const submitForm = {
          siteId: data.site.site_id,
          pageId: data.page?.page_id,
          addParents: added.length ? added : undefined,
          removeParents: removed.length ? removed : undefined
        }
        jsonData(submitForm)
      },
      onResult: async ({ result, cancel }) => {
        if (result.type === "success" && result.data) {
          cancel()
          pagePaneState = PagePane.None
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

  async function fetchParents() {
    const res = await fetch(`?/parentGet`, {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        slug: data.page?.slug
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: string[] },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data.data
      }
    } else if (result.type === "success" && result.data?.res) {
      pageParents = result.data.res.join(" ")
      $form.parents = pageParents
    }
  }

  $effect(() => {
    fetchParents()
  })
</script>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  <h1 class="page-parent-header">
    {data.internationalization?.["wiki-page-parent"]}
  </h1>
{:else}
  <h2 class="page-parent-header">
    {data.internationalization?.["wiki-page-parent"]}
  </h2>
{/if}

<form id="page-parent" class="page-parent" action="?/parentSet" method="POST" use:enhance>
  <input
    class="page-parent-new-parents"
    placeholder={data.internationalization?.parents}
    type="text"
    bind:value={$form.parents}
  />
  {#if pageLayoutContext.current === Layout.WIKIDOT}
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
        value={data.internationalization?.save}
      />
    </div>
  {:else}
    <div class="action-row page-parent-actions">
      <button
        class="action-button page-parent-button button-cancel clickable"
        onclick={() => (pagePaneState = PagePane.None)}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button
        class="action-button page-parent-button button-save clickable"
        type="submit"
      >
        {data.internationalization?.save}
      </button>
    </div>
  {/if}
</form>

<style lang="scss">
  .page-parent {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 100%;
    padding: 0 0 2em;
  }
</style>
