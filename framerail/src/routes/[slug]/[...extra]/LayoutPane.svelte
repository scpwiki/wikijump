<script lang="ts">
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { errorPopupState, pageLayoutState } from "$lib/layout/stores.svelte"
  import { Layout, PagePane } from "$lib/types"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { pagePaneState = $bindable(), data }: PageProps & { pagePaneState: PagePane } =
    $props()

  const pageLayoutContext = getPageLayoutContext()

  const { form, enhance } = superForm(
    untrack(() => data.forms.layoutForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$form,
          siteId: data.site.site_id,
          pageId: data.page?.page_id
        }
        jsonData(submitForm)
      },
      onResult: async ({ result, cancel }) => {
        if (result.type === "success" && result.data) {
          pagePaneState = PagePane.None
          pageLayoutContext.current = result.data.layout
          pageLayoutState.current = result.data.layout
          cancel()
          window.location.reload()
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

  $form.layout = untrack(() => data.page?.layout ?? null)
</script>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  <h1 class="page-layout-header">
    {data.internationalization?.["wiki-page-layout"]}
  </h1>
{:else}
  <h2 class="page-layout-header">
    {data.internationalization?.["wiki-page-layout"]}
  </h2>
{/if}

<form id="page-layout" class="page-layout" action="?/layout" method="POST" use:enhance>
  <select name="layout" class="page-layout-select" bind:value={$form.layout}>
    <option value={null}>
      {data.internationalization?.["wiki-page-layout.default"]}
    </option>
    {#each Object.values(Layout) as layoutOption (layoutOption)}
      <option value={layoutOption}>
        {data.internationalization?.[`wiki-page-layout.${layoutOption}`]}
      </option>
    {/each}
  </select>
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
    <div class="action-row page-layout-actions">
      <button
        class="action-button page-layout-button button-cancel clickable"
        onclick={() => (pagePaneState = PagePane.None)}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button
        class="action-button page-layout-button button-save clickable"
        type="submit"
      >
        {data.internationalization?.save}
      </button>
    </div>
  {/if}
</form>

<style lang="scss">
  .page-layout {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 100%;
    padding: 0 0 2em;
  }
</style>
