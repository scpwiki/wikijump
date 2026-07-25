<script lang="ts">
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { Layout } from "$lib/types"
  import { untrack } from "svelte"
  import { superForm } from "sveltekit-superforms"

  import type { PageProps } from "./$types"

  let { data }: { data: PageProps["data"] } = $props()

  let isEdit = $state(false)

  const { form, enhance } = superForm(
    untrack(() => data.adminForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$form,
          siteId: data.site.site_id,
          action: "edit"
        })
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          isEdit = false
          await invalidateAll()
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
