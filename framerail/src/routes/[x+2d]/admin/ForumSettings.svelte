<script lang="ts">
  import { discussionFormValues } from "$lib/admin-forum.js"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data }: { data: PageProps["data"] } = $props()

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
    $forumNestingFormData.siteId = data.site.site_id
    $forumNestingFormData.maxNestLevel = data.site.forum_max_nest_level
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
</script>

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
