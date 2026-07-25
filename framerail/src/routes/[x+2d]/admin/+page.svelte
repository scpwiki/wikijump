<script lang="ts">
  import { invalidateAll } from "$app/navigation"
  import { discussionFormValues } from "$lib/admin-forum.js"
  import { licenseFormValues, licenseOptionsFor } from "$lib/admin-license.js"
  import { navigationFormValues } from "$lib/admin-navigation.js"
  import { ratingFormValues } from "$lib/admin-rating.js"
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

  const { form: siteIconsFormData, enhance: enhanceSiteIcons } = superForm(
    untrack(() => data.siteIconsForm),
    {
      dataType: "json",
      resetForm: false,
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$siteIconsFormData,
          siteId: data.site.site_id
        })
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data?.res) {
          data.site = result.data.res
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

  const { form: forumNestingFormData, enhance: enhanceForumNesting } = superForm(
    untrack(() => data.forumNestingForm),
    {
      dataType: "json",
      resetForm: false,
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$forumNestingFormData,
          siteId: data.site.site_id
        })
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data?.res) {
          data.site = result.data.res
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

  const { form: discussionFormData, enhance: enhanceDiscussion } = superForm(
    untrack(() => data.discussionForm),
    {
      dataType: "json",
      resetForm: false,
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$discussionFormData,
          siteId: data.site.site_id
        })
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data?.res) {
          const updatedCategory = result.data.res
          const categoryIndex = data.categories.findIndex(
            (category) => category.category_id === updatedCategory.category_id
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
  const ratingCanInherit = $derived(
    data.categories.find(
      (category) => category.category_id === $ratingFormData.categoryId
    )?.slug !== "_default"
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

  function loadDiscussionCategory(categoryId: number) {
    const category = data.categories.find(
      (candidate) => candidate.category_id === categoryId
    )
    if (!category) return
    const values = discussionFormValues(category)
    $discussionFormData.categoryId = values.categoryId
    $discussionFormData.state = values.state
  }

  function handleDiscussionCategoryChange() {
    loadDiscussionCategory($discussionFormData.categoryId)
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

  $effect(() => {
    $forumNestingFormData.siteId = data.site.site_id
    $forumNestingFormData.maxNestLevel = data.site.forum_max_nest_level
    $siteIconsFormData.siteId = data.site.site_id
    $siteIconsFormData.faviconSource = data.site.favicon_source ?? ""
    $siteIconsFormData.iosIconSource = data.site.ios_icon_source ?? ""
    $siteIconsFormData.windowsTileSource = data.site.windows_tile_source ?? ""
  })

  $effect(() => {
    if (
      data.categories.length > 0 &&
      !data.categories.some(
        (category) => category.category_id === $discussionFormData.categoryId
      )
    ) {
      loadDiscussionCategory(data.categories[0].category_id)
    }
  })

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

<section id="site-icons" class="admin-section">
  <h2>Icons</h2>
  <p>
    Set this site's favicon, iOS icon, and Windows 8 tile. Each accepts the address of an
    existing image. Leave a field empty to declare no icon.
  </p>

  <form
    class="editor site-icons-editor"
    action="?/siteIcons"
    method="POST"
    use:enhanceSiteIcons
  >
    <label for="favicon-source">Favicon</label>
    <input
      id="favicon-source"
      name="faviconSource"
      type="text"
      bind:value={$siteIconsFormData.faviconSource}
    />
    <label for="ios-icon-source">iOS icon</label>
    <input
      id="ios-icon-source"
      name="iosIconSource"
      type="text"
      bind:value={$siteIconsFormData.iosIconSource}
    />
    <label for="windows-tile-source">Windows 8 tile</label>
    <input
      id="windows-tile-source"
      name="windowsTileSource"
      type="text"
      bind:value={$siteIconsFormData.windowsTileSource}
    />
    <input name="siteId" type="hidden" bind:value={$siteIconsFormData.siteId} />
    <div class="action-row editor-actions">
      <button
        id="sm-site-icons-save"
        class="action-button editor-button button-save clickable"
        type="submit"
      >
        Save
      </button>
    </div>
  </form>
</section>

<section id="forum-settings" class="admin-section">
  <h2>Forum settings</h2>
  <p>Choose the maximum nesting level for forum replies, from flat to ten levels.</p>

  <form
    class="editor forum-nesting-editor"
    action="?/forumNesting"
    method="POST"
    use:enhanceForumNesting
  >
    <label for="max-nest-level">Maximum nesting level</label>
    <select
      id="max-nest-level"
      name="maxNestLevel"
      bind:value={$forumNestingFormData.maxNestLevel}
    >
      {#each Array.from({ length: 11 }, (_, value) => value) as level (level)}
        <option value={level}>{level}</option>
      {/each}
    </select>
    <input name="siteId" type="hidden" bind:value={$forumNestingFormData.siteId} />
    <div class="action-row editor-actions">
      <button
        id="sm-forum-nesting-save"
        class="action-button editor-button button-save clickable"
        type="submit"
      >
        Save changes
      </button>
    </div>
  </form>
</section>

<section id="per-page-discussion-settings" class="admin-section">
  <h2>Per page discussion</h2>
  <p>Choose whether pages in each category receive a dedicated Discuss action.</p>

  {#if data.categories.length > 0}
    {@const discussionCategory = data.categories.find(
      (category) => category.category_id === $discussionFormData.categoryId
    )}
    <form
      class="editor discussion-editor"
      action="?/discussion"
      method="POST"
      use:enhanceDiscussion
    >
      <label for="sm-forum-perpage-cats">Category</label>
      <select
        id="sm-forum-perpage-cats"
        name="categoryId"
        onchange={handleDiscussionCategoryChange}
        bind:value={$discussionFormData.categoryId}
      >
        {#each data.categories as category (category.category_id)}
          <option value={category.category_id}>{category.slug}</option>
        {/each}
      </select>

      {#if discussionCategory?.slug !== "_default"}
        <label class="radio-label">
          <input
            name="state"
            type="radio"
            value="default"
            bind:group={$discussionFormData.state}
          />
          default
        </label>
      {/if}
      <label class="radio-label">
        <input
          id={`cat234-${$discussionFormData.categoryId}-e`}
          name="state"
          type="radio"
          value="enable"
          bind:group={$discussionFormData.state}
        />
        enable
      </label>
      <label class="radio-label">
        <input
          id={`cat234-${$discussionFormData.categoryId}-d`}
          name="state"
          type="radio"
          value="disable"
          bind:group={$discussionFormData.state}
        />
        disable
      </label>

      <input name="siteId" type="hidden" bind:value={$discussionFormData.siteId} />
      <div class="action-row editor-actions">
        <button
          id="sm-forum-perpage-save"
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

  .license-fields {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .license-fields.inherited {
    display: none;
  }

  .rating-fields {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .rating-fields.inherited,
  #page-rating-settings .default-category {
    display: none;
  }

  #sm-license-noind.default-category {
    display: none;
  }
</style>
