<script lang="ts">
  import { navigationFormValues } from "$lib/admin/admin-navigation.js"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data }: { data: PageProps["data"] } = $props()

  const { form: navigationFormData, enhance: enhanceNavigation } = superForm(
    untrack(() => data.navigationForm),
    {
      dataType: "json",
      resetForm: false,
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$navigationFormData,
          siteId: data.site.site_id
        })
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          const updatedCategory = result.data.res
          const categoryIndex = data.categories.findIndex(
            (category) => category.category_id === updatedCategory?.category_id
          )
          if (categoryIndex !== -1) data.categories[categoryIndex] = updatedCategory
        }
        if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data?.message,
            data: result.data?.data
          }
        }
      }
    }
  )

  function loadNavigationCategory(categoryId: number) {
    const category = data.categories.find(
      (candidate) => candidate.category_id === categoryId
    )
    if (!category) return
    const values = navigationFormValues(category, data.site)
    $navigationFormData.categoryId = values.categoryId
    $navigationFormData.inherit = values.inherit
    $navigationFormData.topBarPage = values.topBarPage
    $navigationFormData.sideBarPage = values.sideBarPage
  }

  function handleNavigationCategoryChange() {
    loadNavigationCategory($navigationFormData.categoryId)
  }

  $effect(() => {
    if (
      data.categories.length > 0 &&
      !data.categories.some(
        (category) => category.category_id === $navigationFormData.categoryId
      )
    ) {
      loadNavigationCategory(data.categories[0].category_id)
    }
  })
</script>

<section id="navigation-settings" class="admin-section">
  <h2>Navigation elements</h2>
  <p>
    Choose which navigation elements (<em>top-bar</em> and <em>side-bar</em>) should
    appear on pages within a specified category.
  </p>

  {#if data.categories.length > 0}
    <form
      class="editor navigation-editor"
      action="?/navigation"
      method="POST"
      use:enhanceNavigation
    >
      <label for="sm-nav-cats">Choose the category:</label>
      <select
        id="sm-nav-cats"
        name="categoryId"
        onchange={handleNavigationCategoryChange}
        bind:value={$navigationFormData.categoryId}
      >
        {#each data.categories as category (category.category_id)}
          <option value={category.category_id}>{category.slug}</option>
        {/each}
      </select>

      <div id="sm-nav-noind">
        <label class="checkbox-label" for="sm-nav-noin">
          <input
            id="sm-nav-noin"
            name="inherit"
            type="checkbox"
            bind:checked={$navigationFormData.inherit}
          />
          No individual nav elements
        </label>
      </div>

      <div
        id="sm-nav-list"
        class="navigation-fields"
        class:inherited={$navigationFormData.inherit}
      >
        <label for="sm-nav-top-bar">Top-bar:</label>
        <input
          id="sm-nav-top-bar"
          name="topBarPage"
          type="text"
          bind:value={$navigationFormData.topBarPage}
        />

        <label for="sm-nav-side-bar">Side-bar:</label>
        <input
          id="sm-nav-side-bar"
          name="sideBarPage"
          type="text"
          bind:value={$navigationFormData.sideBarPage}
        />
      </div>

      <input name="siteId" type="hidden" bind:value={$navigationFormData.siteId} />

      <div class="action-row editor-actions">
        <button
          id="sm-nav-save"
          class="action-button editor-button button-save clickable"
          type="submit"
        >
          Save changes
        </button>
      </div>
    </form>
    <p class="settings-note">
      <strong>NOTE:</strong> if the chosen pages do not exist no navigation elements will be
      displayed.
    </p>
  {:else}
    <p>No page categories are available.</p>
  {/if}
</section>
