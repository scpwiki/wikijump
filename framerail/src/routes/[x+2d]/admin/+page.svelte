<script lang="ts">
  import { invalidateAll } from "$app/navigation"
  import { navigationFormValues } from "$lib/admin-navigation.js"
  import { errorPopupState } from "$lib/stores.svelte"
  import { Layout } from "$lib/types"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data }: PageProps = $props()

  let isEdit = $state<boolean>(false)

  const { form, enhance } = superForm(
    untrack(() => data.adminForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$form,
          siteId: data.site.site_id,
          action: "edit"
        }
        jsonData(submitForm)
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          isEdit = false
          await invalidateAll()
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

  function handleEdit() {
    isEdit = true
    $form.name = data.site.name
    $form.slug = data.site.slug
    $form.tagline = data.site.tagline
    $form.description = data.site.description
    $form.defaultPage = data.site.default_page
    $form.locale = data.site.locale
    $form.layout = data.site.layout ?? Layout.WIKIJUMP
  }
</script>

<h1>Site manager</h1>

{#if isEdit}
  <form id="editor" class="editor" action="?/site" method="POST" use:enhance>
    <label for="name">
      {data.internationalization?.["site-info.name"]}
    </label>
    <input name="name" class="site-attribute name" type="text" bind:value={$form.name} />

    <label for="slug">
      {data.internationalization?.["site-info.slug"]}
    </label>
    <input name="slug" class="site-attribute slug" type="text" bind:value={$form.slug} />

    <label for="tagline">
      {data.internationalization?.["site-info.tagline"]}
    </label>
    <input
      name="tagline"
      class="site-attribute tagline"
      type="text"
      bind:value={$form.tagline}
    />

    <label for="description">
      {data.internationalization?.["site-info.description"]}
    </label>
    <input
      name="description"
      class="site-attribute description"
      type="text"
      bind:value={$form.description}
    />

    <label for="default-page">
      {data.internationalization?.["site-info.default-page"]}
    </label>
    <input
      name="defaultPage"
      class="site-attribute default-page"
      type="text"
      bind:value={$form.defaultPage}
    />

    <label for="locale">
      {data.internationalization?.["site-info.locale"]}
    </label>
    <input
      name="locale"
      class="site-attribute locale"
      type="text"
      bind:value={$form.locale}
    />

    <label for="layout">
      {data.internationalization?.["site-info.layout"]}
    </label>
    <select name="layout" class="site-attribute layout" bind:value={$form.layout}>
      <option value={null}>
        {data.internationalization?.["wiki-page-layout.default"]}
      </option>
      {#each Object.values(Layout) as layoutOption (layoutOption)}
        <option value={layoutOption}>
          {data.internationalization?.[`wiki-page-layout.${layoutOption}`]}
        </option>
      {/each}
    </select>

    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-cancel clickable"
        onclick={() => (isEdit = false)}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button class="action-button editor-button button-save clickable" type="submit">
        {data.internationalization?.save}
      </button>
    </div>
  </form>
{:else}
  <div class="site-info" data-id={data.site.site_id}>
    {#if data.site.name}
      <div class="site-attribute name">
        <span class="site-attribute-label">
          {data.internationalization?.["site-info.name"]}
        </span>
        <span class="site-attribute-value">{data.site.name}</span>
      </div>
    {/if}

    {#if data.site.slug}
      <div class="site-attribute slug">
        <span class="site-attribute-label">
          {data.internationalization?.["site-info.slug"]}
        </span>
        <span class="site-attribute-value">{data.site.slug}</span>
      </div>
    {/if}

    {#if data.site.tagline}
      <div class="site-attribute tagline">
        <span class="site-attribute-label">
          {data.internationalization?.["site-info.tagline"]}
        </span>
        <span class="site-attribute-value">{data.site.tagline}</span>
      </div>
    {/if}

    {#if data.site.description}
      <div class="site-attribute description">
        <span class="site-attribute-label">
          {data.internationalization?.["site-info.description"]}
        </span>
        <span class="site-attribute-value">{data.site.description}</span>
      </div>
    {/if}

    {#if data.site.default_page}
      <div class="site-attribute default-page">
        <span class="site-attribute-label">
          {data.internationalization?.["site-info.default-page"]}
        </span>
        <span class="site-attribute-value">{data.site.default_page}</span>
      </div>
    {/if}

    {#if data.site.locale}
      <div class="site-attribute locale">
        <span class="site-attribute-label">
          {data.internationalization?.["site-info.locale"]}
        </span>
        <span class="site-attribute-value">{data.site.locale}</span>
      </div>
    {/if}

    {#if data.site.layout}
      <div class="site-attribute layout">
        <span class="site-attribute-label">
          {data.internationalization?.["site-info.layout"]}
        </span>
        <span class="site-attribute-value" data-value={data.site.layout}>
          {data.internationalization?.[`wiki-page-layout.${data.site.layout}`]}
        </span>
      </div>
    {/if}
  </div>

  <div class="action-row editor-actions">
    <button
      class="action-button editor-button button-edit clickable"
      onclick={handleEdit}
      type="button"
    >
      {data.internationalization?.edit}
    </button>
  </div>
{/if}

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

<style global lang="scss">
  .site-info {
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

  .action-row {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: stretch;
    justify-content: flex-end;
    width: 100%;
  }

  .admin-section {
    margin-top: 2rem;
  }

  .checkbox-label {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .navigation-fields {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .navigation-fields.inherited {
    display: none;
  }
</style>
