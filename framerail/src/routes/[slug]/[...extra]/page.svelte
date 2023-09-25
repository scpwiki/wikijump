<script lang="ts">
  export let data
  import { goto, invalidateAll } from "$app/navigation"
  let showMoveAction = false
  let moveInputNewSlugElem: HTMLInputElement

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
    await fetch(`/${data.page.slug}`, {
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
  }
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
    <textarea
      name="comments"
      class="editor-comments"
      placeholder="comments"
      type="text"
    />
    <div class="editor-actions">
      <button
        class="editor-button button-cancel clickable"
        type="button"
        on:click|stopPropagation={cancelEdit}
      >
        UT:Cancel
      </button>
      <button
        class="editor-button button-save clickable"
        type="submit"
        on:click|stopPropagation
      >
        UT:Save
      </button>
    </div>
  </form>
{:else}
  <div class="editor-actions">
    <button
      class="editor-button button-move clickable"
      type="button"
      on:click={() => {
        $: showMoveAction = true
      }}
    >
      UT:Move
    </button>
    <button
      class="editor-button button-delete clickable"
      type="button"
      on:click={handleDelete}
    >
      UT:Delete
    </button>
    <button
      class="editor-button button-edit clickable"
      type="button"
      on:click={navigateEdit}
    >
      UT:Edit
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
    <textarea
      name="comments"
      class="page-move-comments"
      placeholder="comments"
      type="text"
    />
    <div class="page-move-actions">
      <button
        class="page-move-button button-cancel clickable"
        type="button"
        on:click|stopPropagation={() => {
          $: showMoveAction = false
        }}
      >
        UT:Cancel
      </button>
      <button
        class="page-move-button button-move clickable"
        type="submit"
        on:click|stopPropagation
      >
        UT:Move
      </button>
    </div>
  </form>
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
    justify-content: flex-start;
    align-items: center;
    gap: 10px;
    padding: 0;
    margin: 0;
    list-style: none;
  }

  .editor,
  .page-move {
    width: 80vw;
    display: flex;
    flex-direction: column;
    justify-content: stretch;
    align-items: stretch;
    gap: 15px;
  }

  .editor-wikitext {
    height: 60vh;
  }

  .editor-actions,
  .page-move-actions {
    width: 100%;
    display: flex;
    flex-direction: row;
    justify-content: flex-end;
    align-items: stretch;
    gap: 10px;
  }
</style>
