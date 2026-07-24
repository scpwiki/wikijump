<script lang="ts">
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data }: { data: PageProps["data"] } = $props()

  const { form: templateFormData, enhance: enhanceTemplate } = superForm(
    untrack(() => data.templateForm),
    {
      dataType: "json",
      resetForm: false,
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$templateFormData,
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

  function loadTemplateCategory(categoryId: number) {
    const category = data.categories.find(
      (candidate) => candidate.category_id === categoryId
    )
    if (!category) return
    $templateFormData.categoryId = category.category_id
    $templateFormData.templatePageId = data.pageTemplates.some(
      (template) => template.page_id === category.template_page_id
    )
      ? category.template_page_id
      : null
  }

  function handleTemplateCategoryChange() {
    loadTemplateCategory($templateFormData.categoryId)
  }

  $effect(() => {
    if (
      data.categories.length > 0 &&
      !data.categories.some(
        (category) => category.category_id === $templateFormData.categoryId
      )
    ) {
      loadTemplateCategory(data.categories[0].category_id)
    }
  })
</script>

<section id="template-settings" class="admin-section">
  <h2>Templates</h2>
  <p>
    Assign a page from the <code>template:</code> category as the initial source for new pages
    in a category.
  </p>

  {#if data.categories.length > 0}
    <form
      class="editor template-editor"
      action="?/template"
      method="POST"
      use:enhanceTemplate
    >
      <label for="sm-template-cats">Category</label>
      <select
        id="sm-template-cats"
        name="categoryId"
        onchange={handleTemplateCategoryChange}
        bind:value={$templateFormData.categoryId}
      >
        {#each data.categories as category (category.category_id)}
          <option value={category.category_id}>{category.slug}</option>
        {/each}
      </select>

      <label for="sm-templates-list">Template</label>
      <select
        id="sm-templates-list"
        name="templatePageId"
        bind:value={$templateFormData.templatePageId}
      >
        <option value={null}>no default template</option>
        {#each data.pageTemplates as template (template.page_id)}
          <option value={template.page_id}>{template.title}</option>
        {/each}
      </select>

      {#if data.pageTemplates.length === 0}
        <p class="settings-note">
          No visible template pages are available. You can still clear this category's
          assignment by saving <q>no default template</q>.
        </p>
      {/if}

      {#if $templateFormData.templatePageId !== null}
        {@const selectedTemplate = data.pageTemplates.find(
          (template) => template.page_id === $templateFormData.templatePageId
        )}
        {#if selectedTemplate}
          <div
            id={`sm-template-preview-${selectedTemplate.page_id}`}
            class="template-preview"
          >
            <h3>Template preview:</h3>
            <pre>{selectedTemplate.wikitext}</pre>
          </div>
        {/if}
      {/if}

      <input name="siteId" type="hidden" bind:value={$templateFormData.siteId} />
      <div class="action-row editor-actions">
        <button
          id="sm-templates-save"
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
