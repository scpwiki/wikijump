<script lang="ts">
  export let data
  import { goto, invalidateAll } from "$app/navigation"

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
    goto(`/${data.page.slug}/edit`, {
      noScroll: true
    })
  }

  function cancelEdit() {
    goto(`/${data.page.slug}`, {
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
  {@html data.compiledHtml}
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
  .page-tags-container {
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

  .editor {
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

  .editor-actions {
    width: 100%;
    display: flex;
    flex-direction: row;
    justify-content: flex-end;
    align-items: stretch;
    gap: 10px;
  }
</style>
