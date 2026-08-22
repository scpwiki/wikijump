<script lang="ts">
  import { errorPopupState, pageLayoutState } from "$lib/stores.svelte"
  import { Layout, PagePane, ToastType } from "$lib/types"
  import { toast } from "$lib/component/scripts/toasts"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { pagePaneState = $bindable(), data }: PageProps & { pagePaneState: PagePane } =
    $props()

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
          toast(ToastType.Success, data.internationalization!["wiki-page-layout.toast"]!)
          pagePaneState = PagePane.None
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

{#if pageLayoutState.current === Layout.WIKIDOT}
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
