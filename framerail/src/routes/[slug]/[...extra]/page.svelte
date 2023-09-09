<script lang="ts">
  export let data
  import { goto } from "$app/navigation"

  function navigateEdit() {
    goto(`/${data.page.slug}/edit`, {
      noScroll: true,
    })
  }

  function cancelEdit() {
    goto(`/${data.page.slug}`, {
      noScroll: true,
    })
  }

  async function saveEdit() {
    let form = document.getElementById("editor")
    let fdata = new FormData(form)
    fdata.set("site-id", data.site.siteId)
    fdata.set("page-id", data.page.pageId)
    await fetch(`/${data.page.slug}`, {
      method: "POST",
      body: fdata,
    })
    goto(`/${data.page.slug}`, {
      noScroll: true,
    })
  }
</script>

<h1>UNTRANSLATED:Loaded page</h1>
<p class="spin-yay">
  UNTRANSLATED:This is a generic page renderer loaded as a component.
</p>
<p>UNTRANSLATED:Response <textarea class="debug">{JSON.stringify(data, null, 2)}</textarea></p>

{@html data.compiledHtml}

{#if data.options?.edit}
  <form
    id="editor"
    class="editor"
    method="POST"
    on:submit|preventDefault={saveEdit} >
    <input
      class="editor-title"
      name="title"
      placeholder="title"
      value={data.pageRevision.title}
      />
    <input
      class="editor-alt-title"
      name="alt-title"
      placeholder="alternative title"
      value={data.pageRevision.altTitle}
      />
    <textarea class="editor-wikitext" name="wikitext">{data.wikitext}</textarea>
    <input
      class="editor-tags"
      name="tags"
      placeholder="tags"
      value={data.pageRevision.tags.join(" ")}
      />
    <div class="editor-actions">
      <button
        type="button"
        class="editor-button button-cancel"
        on:click|stopPropagation={cancelEdit} >
        UT:Cancel
      </button>
      <button
        type="submit"
        class="editor-button button-save"
        on:click|stopPropagation >
        UT:Save
      </button>
    </div>
  </form>
{:else}
  <div class="editor-actions">
    <button
      class="edit-button"
      on:click={navigateEdit} >
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
