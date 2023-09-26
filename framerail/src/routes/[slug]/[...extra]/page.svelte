<script lang="ts">
  export let data
  import { goto, invalidateAll } from "$app/navigation"
  import { onMount } from "svelte"

  let showMoveAction = false
  let showHistory = false
  let moveInputNewSlugElem: HTMLInputElement
  let revisionList = []

  async function handleDelete() {
    let fdata = new FormData()
    fdata.set("site-id", data.site.siteId)
    fdata.set("page-id", data.page.pageId)
    await fetch(`/${data.page.slug}`, {
      method: "DELETE",
      body: fdata
    })
    invalidateAll()
  }

  function navigateEdit() {
    let options: string[] = []
    if (data.options.noRender) options.push("norender")
    options = options.map((opt) => `/${opt}`)
    goto(`/${data.page.slug}${options.join("")}/edit`, {
      noScroll: true
    })
  }

  function cancelEdit() {
    let options: string[] = []
    if (data.options.noRender) options.push("norender")
    options = options.map((opt) => `/${opt}`)
    goto(`/${data.page.slug}${options.join("")}`, {
      noScroll: true
    })
  }

  async function saveEdit() {
    let form = document.getElementById("editor")
    let fdata = new FormData(form)
    fdata.set("site-id", data.site.siteId)
    fdata.set("page-id", data.page.pageId)
    await fetch(`/${data.page.slug}/edit`, {
      method: "POST",
      body: fdata
    })
    goto(`/${data.page.slug}`, {
      noScroll: true
    })
  }

  async function handleMove() {
    let form = document.getElementById("page-move")
    let fdata = new FormData(form)
    let newSlug = fdata.get("new-slug")
    if (!newSlug) {
      moveInputNewSlugElem.style.outline = "1px solid red"
      return
    } else {
      moveInputNewSlugElem.style.outline = ""
    }
    fdata.set("site-id", data.site.siteId)
    fdata.set("page-id", data.page.pageId)
    await fetch(`/${data.page.slug}`, {
      method: "PUT",
      body: fdata
    })
    goto(`/${newSlug}`, {
      noScroll: true
    })
    showMoveAction = false
  }

  async function handleHistory() {
    let fdata = new FormData()
    fdata.set("site-id", data.site.siteId)
    fdata.set("page-id", data.page.pageId)
    revisionList = await fetch(`/${data.page.slug}/history`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    showHistory = true
  }

  onMount(() => {
    if (data?.options.history) handleHistory()
  })
</script>

<h1>UNTRANSLATED:Loaded page</h1>
<p class="spin-yay">
  UNTRANSLATED:This is a generic page renderer loaded as a component.
</p>
<p>
  UNTRANSLATED:Response <textarea class="debug">{JSON.stringify(data, null, 2)}</textarea>
</p>

<h2>{data.pageRevision.title}</h2>

<hr />

<div class="page-content">
  {#if data.options?.noRender}
    UNTRANSLATED: Content not shown.
  {:else}
    {@html data.compiledHtml}
  {/if}
</div>

<div class="page-tags-container">
  Tags
  <hr />
  <ul class="page-tags">
    {#each data.pageRevision.tags as tag}
      <li class="tag">{tag}</li>
    {/each}
  </ul>
</div>

{#if data.options?.edit}
  <form id="editor" class="editor" method="POST" on:submit|preventDefault={saveEdit}>
    <input
      name="title"
      class="editor-title"
      placeholder="title"
      type="text"
      value={data.pageRevision.title}
    />
    <input
      name="alt-title"
      class="editor-alt-title"
      placeholder="alternative title"
      type="text"
      value={data.pageRevision.altTitle}
    />
    <textarea name="wikitext" class="editor-wikitext">{data.wikitext}</textarea>
    <input
      name="tags"
      class="editor-tags"
      placeholder="tags"
      type="text"
      value={data.pageRevision.tags.join(" ")}
    />
    <textarea name="comments" class="editor-comments" placeholder="comments" />
    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-cancel clickable"
        type="button"
        on:click|stopPropagation={cancelEdit}
      >
        UT:Cancel
      </button>
      <button
        class="action-button editor-button button-save clickable"
        type="submit"
        on:click|stopPropagation
      >
        UT:Save
      </button>
    </div>
  </form>
{:else}
  <div class="action-row editor-actions">
    <button
      class="action-button editor-button button-move clickable"
      type="button"
      on:click={() => {
        $: showMoveAction = true
      }}
    >
      UT:Move
    </button>
    <button
      class="action-button editor-button button-delete clickable"
      type="button"
      on:click={handleDelete}
    >
      UT:Delete
    </button>
    <button
      class="action-button editor-button button-edit clickable"
      type="button"
      on:click={navigateEdit}
    >
      UT:Edit
    </button>
  </div>
  <div class="action-row other-actions">
    <button
      class="action-button button-history clickable"
      type="button"
      on:click={handleHistory}
    >
      UT:History
    </button>
  </div>
{/if}

{#if showMoveAction}
  <form
    id="page-move"
    class="page-move"
    method="PUT"
    on:submit|preventDefault={handleMove}
  >
    <input
      bind:this={moveInputNewSlugElem}
      name="new-slug"
      class="page-move-new-slug"
      placeholder="new slug"
      type="text"
    />
    <textarea name="comments" class="page-move-comments" placeholder="comments" />
    <div class="action-row page-move-actions">
      <button
        class="action-button page-move-button button-cancel clickable"
        type="button"
        on:click|stopPropagation={() => {
          $: showMoveAction = false
        }}
      >
        UT:Cancel
      </button>
      <button
        class="action-button page-move-button button-move clickable"
        type="submit"
        on:click|stopPropagation
      >
        UT:Move
      </button>
    </div>
  </form>
{/if}

{#if showHistory}
  <div class="revision-list">
    <div class="revision-header">
      <div class="revision-attribute revision-number">UT: Revision #</div>
      <div class="revision-attribute created-at">UT: Creation</div>
      <div class="revision-attribute user">UT: User</div>
      <div class="revision-attribute comments">UT: Comments</div>
    </div>
    {#each revisionList.reverse() as revision}
      <div class="revision-row" data-id={revision.revision_id}>
        <div class="revision-attribute revision-number">
          {revision.revision_number}
        </div>
        <div class="revision-attribute created-at">
          {revision.created_at}
        </div>
        <div class="revision-attribute user">
          {revision.user_id}
        </div>
        <div class="revision-attribute comments">
          {revision.comments}
        </div>
      </div>
    {/each}
  </div>
{/if}

<style global lang="scss">
  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .spin-yay {
    display: inline-block;
    animation: spin 2s linear infinite;
  }

  .debug {
    width: 80vw;
    height: 60vh;
  }

  .page-content,
  .page-tags-container,
  .editor-actions,
  .page-move {
    padding: 0 0 2em;
  }

  .page-tags {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    justify-content: flex-start;
    padding: 0;
    margin: 0;
    list-style: none;
  }

  .editor,
  .page-move {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 80vw;
  }

  .editor-wikitext {
    height: 60vh;
  }

  .action-row {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: stretch;
    justify-content: flex-end;
    width: 100%;
  }

  .revision-list {
    display: table;
    width: 100%;

    .revision-header,
    .revision-row {
      display: table-row;

      .revision-attribute {
        display: table-cell;
      }
    }
  }
</style>
