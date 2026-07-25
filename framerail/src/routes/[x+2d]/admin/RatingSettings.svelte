<script lang="ts">
  import { ratingFormValues } from "$lib/admin-rating.js"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data }: { data: PageProps["data"] } = $props()

  const { form: ratingFormData, enhance: enhanceRating } = superForm(
    untrack(() => data.ratingForm),
    {
      dataType: "json",
      resetForm: false,
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$ratingFormData,
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

  const ratingCanInherit = $derived(
    data.categories.find(
      (category) => category.category_id === $ratingFormData.categoryId
    )?.slug !== "_default"
  )

  function loadRatingCategory(categoryId: number) {
    const category = data.categories.find(
      (candidate) => candidate.category_id === categoryId
    )
    if (!category) return
    const defaultCategory = data.categories.find(
      (candidate) => candidate.slug === "_default"
    )
    const values = ratingFormValues(category, defaultCategory)
    $ratingFormData.categoryId = values.categoryId
    $ratingFormData.inherit = values.inherit
    $ratingFormData.enabled = values.enabled
    $ratingFormData.permission = values.permission
    $ratingFormData.visibility = values.visibility
    $ratingFormData.ratingType = values.ratingType
  }

  function handleRatingCategoryChange() {
    loadRatingCategory($ratingFormData.categoryId)
  }

  $effect(() => {
    if (
      data.categories.length > 0 &&
      !data.categories.some(
        (category) => category.category_id === $ratingFormData.categoryId
      )
    ) {
      loadRatingCategory(data.categories[0].category_id)
    }
  })
</script>

<section id="page-rating-settings" class="admin-section">
  <h2>Page rating</h2>
  <p>Configure page rating behavior by category.</p>

  {#if data.categories.length > 0}
    <form class="editor rating-editor" action="?/rating" method="POST" use:enhanceRating>
      <label for="sm-pagerate-cats">Category</label>
      <select
        id="sm-pagerate-cats"
        name="categoryId"
        onchange={handleRatingCategoryChange}
        bind:value={$ratingFormData.categoryId}
      >
        {#each data.categories as category (category.category_id)}
          <option value={category.category_id}>{category.slug}</option>
        {/each}
      </select>

      <div class:default-category={!ratingCanInherit}>
        <label class="checkbox-label" for="sm-pagerate-inherit">
          <input
            id="sm-pagerate-inherit"
            name="inherit"
            disabled={!ratingCanInherit}
            type="checkbox"
            bind:checked={$ratingFormData.inherit}
          />
          Inherit from <code>_default</code>
        </label>
      </div>

      <div class="rating-fields" class:inherited={$ratingFormData.inherit}>
        <label class="checkbox-label" for="sm-pagerate-enabled">
          <input
            id="sm-pagerate-enabled"
            name="enabled"
            type="checkbox"
            bind:checked={$ratingFormData.enabled}
          />
          Enable page rating
        </label>

        <label for="sm-pagerate-permission">Who can rate pages?</label>
        <select
          id="sm-pagerate-permission"
          name="permission"
          bind:value={$ratingFormData.permission}
        >
          <option value="registered">Registered users</option>
          <option value="members">Site members</option>
        </select>

        <label for="sm-pagerate-visibility">Votes</label>
        <select
          id="sm-pagerate-visibility"
          name="visibility"
          bind:value={$ratingFormData.visibility}
        >
          <option value="visible">Visible</option>
          <option value="anonymous">Anonymous</option>
        </select>

        <label for="sm-pagerate-type">Rating type</label>
        <select
          id="sm-pagerate-type"
          name="ratingType"
          bind:value={$ratingFormData.ratingType}
        >
          <option value="plus">+ only</option>
          <option value="plus_minus">+/-</option>
          <option value="stars">Stars</option>
        </select>
      </div>

      <input name="siteId" type="hidden" bind:value={$ratingFormData.siteId} />
      <div class="action-row editor-actions">
        <button
          id="sm-pagerate-save"
          class="action-button editor-button button-save clickable"
          type="submit"
        >
          Save changes
        </button>
      </div>
    </form>
  {:else}
    <p>No page categories are available.</p>
  {/if}
</section>
