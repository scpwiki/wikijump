<script lang="ts">
  import { licenseFormValues, licenseOptionsFor } from "$lib/admin/admin-license.js"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data }: { data: PageProps["data"] } = $props()

  const { form: licenseFormData, enhance: enhanceLicense } = superForm(
    untrack(() => data.licenseForm),
    {
      dataType: "json",
      resetForm: false,
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$licenseFormData,
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

  const licenseCanInherit = $derived(
    data.categories.find(
      (category) => category.category_id === $licenseFormData.categoryId
    )?.slug !== "_default"
  )

  function loadLicenseCategory(categoryId: number) {
    const category = data.categories.find(
      (candidate) => candidate.category_id === categoryId
    )
    if (!category) return
    const defaultCategory = data.categories.find(
      (candidate) => candidate.slug === "_default"
    )
    const defaultLicense = defaultCategory?.license ?? data.site.license
    const values = licenseFormValues(
      category,
      defaultLicense,
      defaultCategory?.license_other
    )
    $licenseFormData.categoryId = values.categoryId
    $licenseFormData.inherit = values.inherit
    $licenseFormData.license = values.license
    $licenseFormData.licenseOther = values.licenseOther
  }

  function handleLicenseCategoryChange() {
    loadLicenseCategory($licenseFormData.categoryId)
  }

  $effect(() => {
    if (
      data.categories.length > 0 &&
      !data.categories.some(
        (category) => category.category_id === $licenseFormData.categoryId
      )
    ) {
      loadLicenseCategory(data.categories[0].category_id)
    }
  })
</script>

<section id="license-settings" class="admin-section">
  <h2>License</h2>
  <p>Set up a license for your Wiki.</p>

  {#if data.categories.length > 0}
    <form
      class="editor license-editor"
      action="?/license"
      method="POST"
      use:enhanceLicense
    >
      <label for="sm-license-cats">Category</label>
      <select
        id="sm-license-cats"
        name="categoryId"
        onchange={handleLicenseCategoryChange}
        bind:value={$licenseFormData.categoryId}
      >
        {#each data.categories as category (category.category_id)}
          <option value={category.category_id}>{category.slug}</option>
        {/each}
      </select>

      <div id="sm-license-noind" class:default-category={!licenseCanInherit}>
        <label class="checkbox-label" for="sm-license-noin">
          <input
            id="sm-license-noin"
            name="inherit"
            disabled={!licenseCanInherit}
            type="checkbox"
            bind:checked={$licenseFormData.inherit}
          />
          Inherit from <code>_default</code>
        </label>
      </div>

      <div
        id="sm-license-list"
        class="license-fields"
        class:inherited={$licenseFormData.inherit}
      >
        <label for="sm-license-lic">License</label>
        <select id="sm-license-lic" name="license" bind:value={$licenseFormData.license}>
          {#each licenseOptionsFor($licenseFormData.license) as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>

        {#if $licenseFormData.license === "other"}
          <label for="sm-other-license-text">License description</label>
          <textarea
            id="sm-other-license-text"
            name="licenseOther"
            maxlength="300"
            rows="5"
            bind:value={$licenseFormData.licenseOther}></textarea>
          <p id="sm-other-license-left" class="settings-note">
            {300 - $licenseFormData.licenseOther.length} characters left
          </p>
        {/if}
      </div>

      <input name="siteId" type="hidden" bind:value={$licenseFormData.siteId} />

      <div class="action-row editor-actions">
        <button
          id="sm-license-save"
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
