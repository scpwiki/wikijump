<script lang="ts">
  import { page } from "$app/stores"
  import { goto, invalidateAll } from "$app/navigation"
  import { onMount } from "svelte"
  import { useErrorPopup } from "$lib/stores"
  let showErrorPopup = useErrorPopup()

  let showMoveAction = false
  let showHistory = false
  let showSource = false
  let showRevision = false
  let showRevisionSource = false
  let moveInputNewSlugElem: HTMLInputElement
  let revisionMap: Map<number, Record<string, any>> = new Map()
  let revision: Record<string, any> = {}

  async function handleDelete() {
    let fdata = new FormData()
    fdata.set("site-id", $page.data.site.site_id)
    fdata.set("page-id", $page.data.page.page_id)
    let res = await fetch(`/${$page.data.page.slug}`, {
      method: "DELETE",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message
      })
    } else invalidateAll()
  }

  function navigateEdit() {
    let options: string[] = []
    if ($page.data.options.noRender) options.push("norender")
    options = options.map((opt) => `/${opt}`)
    goto(`/${$page.data.page.slug}${options.join("")}/edit`, {
      noScroll: true
    })
  }

  function cancelEdit() {
    let options: string[] = []
    if ($page.data.options.noRender) options.push("norender")
    options = options.map((opt) => `/${opt}`)
    goto(`/${$page.data.page.slug}${options.join("")}`, {
      noScroll: true
    })
  }

  async function saveEdit() {
    let form = document.getElementById("editor")
    let fdata = new FormData(form)
    fdata.set("site-id", $page.data.site.site_id)
    fdata.set("page-id", $page.data.page.page_id)
    let res = await fetch(`/${$page.data.page.slug}/edit`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message
      })
    } else {
      goto(`/${$page.data.page.slug}`, {
        noScroll: true
      })
    }
  }

  async function handleMove() {
    let form = document.getElementById("page-move")
    let fdata = new FormData(form)
    let newSlug = fdata.get("new-slug")
    if (!newSlug) {
      moveInputNewSlugElem.classList.add("error")
      return
    } else {
      moveInputNewSlugElem.classList.remove("error")
    }
    fdata.set("site-id", $page.data.site.site_id)
    fdata.set("page-id", $page.data.page.page_id)
    let res = await fetch(`/${$page.data.page.slug}/move`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message
      })
    } else {
      goto(`/${newSlug}`, {
        noScroll: true
      })
      showMoveAction = false
    }
  }

  async function handleHistory() {
    let fdata = new FormData()
    fdata.set("site-id", $page.data.site.site_id)
    fdata.set("page-id", $page.data.page.page_id)
    let res = await fetch(`/${$page.data.page.slug}/history`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message
      })
    } else {
      res.forEach((rev) => {
        revisionMap.set(rev.revision_number, rev)
      })
      showHistory = true
    }
  }

  async function getRevision(
    revisionNumber: number,
    compiledHtml: boolean,
    wikitext: boolean
  ) {
    // Get cached revision if we have it
    let rev = revisionMap.get(revisionNumber)
    // Try to see if the cached revision already has the wanted data
    if (compiledHtml && rev?.compiled_html) {
      revision = rev
    } else if (wikitext && rev?.wikitext) {
      revision = rev
    } else {
      // Request from server
      let fdata = new FormData()
      fdata.set("site-id", $page.data.site.site_id)
      fdata.set("page-id", $page.data.page.page_id)
      fdata.set("revision-number", revisionNumber)
      fdata.set("compiled-html", compiledHtml)
      fdata.set("wikitext", wikitext)
      let res = await fetch(`/${$page.data.page.slug}/revision`, {
        method: "POST",
        body: fdata
      }).then((res) => res.json())
      if (res?.message) {
        showErrorPopup.set({
          state: true,
          message: res.message
        })
      } else if (!rev) {
        // This is a revision we didn't even cache...?
        revisionMap.set(res.revision_number, res)
        revision = res
      } else if (compiledHtml) {
        rev.compiled_html = res.compiled_html
        revision = rev
      } else if (wikitext) {
        rev.wikitext = res.wikitext
        revision = rev
      }
    }
  }

  onMount(() => {
    if ($page.data?.options.history) handleHistory()
  })
</script>

<h1>UNTRANSLATED:Loaded page</h1>
<p>
  UNTRANSLATED:Response <textarea class="debug">{JSON.stringify($page, null, 2)}</textarea
  >
</p>

{#if showRevision}
  <h2>{revision.title}</h2>
{:else}
  <h2>{$page.data.page_revision.title}</h2>
{/if}

<hr />

<div class="page-content">
  {#if $page.data.options?.no_render}
    {$page.data.internationalization["wiki-page-no-render"]}
    <textarea class="page-source" readonly={true}>{$page.data.wikitext}</textarea>
  {:else if showRevision}
    {@html revision.compiled_html}
  {:else}
    {@html $page.data.compiled_html}
  {/if}
</div>

<div class="page-tags-container">
  {$page.data.internationalization?.tags}
  <hr />
  <ul class="page-tags">
    {#if showRevision}
      {#each revision.tags as tag}
        <li class="tag">{tag}</li>
      {/each}
    {:else}
      {#each $page.data.page_revision.tags as tag}
        <li class="tag">{tag}</li>
      {/each}
    {/if}
  </ul>
</div>

<div class="page-revision-container">
  {$page.data.internationalization["wiki-page-revision"]}
</div>

{#if $page.data.options?.edit}
  <form id="editor" class="editor" method="POST" on:submit|preventDefault={saveEdit}>
    <input
      name="title"
      class="editor-title"
      placeholder={$page.data.internationalization?.title}
      type="text"
      value={$page.data.page_revision.title}
    />
    <input
      name="alt-title"
      class="editor-alt-title"
      placeholder={$page.data.internationalization?.["alt-title"]}
      type="text"
      value={$page.data.page_revision.alt_title}
    />
    <textarea name="wikitext" class="editor-wikitext">{$page.data.wikitext}</textarea>
    <input
      name="tags"
      class="editor-tags"
      placeholder={$page.data.internationalization?.tags}
      type="text"
      value={$page.data.page_revision.tags.join(" ")}
    />
    <textarea
      name="comments"
      class="editor-comments"
      placeholder={$page.data.internationalization?.["wiki-page-revision-comments"]}
    />
    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-cancel clickable"
        type="button"
        on:click|stopPropagation={cancelEdit}
      >
        {$page.data.internationalization?.cancel}
      </button>
      <button
        class="action-button editor-button button-save clickable"
        type="submit"
        on:click|stopPropagation
      >
        {$page.data.internationalization?.save}
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
      {$page.data.internationalization?.move}
    </button>
    <button
      class="action-button editor-button button-delete clickable"
      type="button"
      on:click={handleDelete}
    >
      {$page.data.internationalization?.delete}
    </button>
    <button
      class="action-button editor-button button-edit clickable"
      type="button"
      on:click={navigateEdit}
    >
      {$page.data.internationalization?.edit}
    </button>
  </div>
  <div class="action-row other-actions">
    <button
      class="action-button button-source clickable"
      type="button"
      on:click={() => (showSource = true)}
    >
      {$page.data.internationalization?.["wiki-page-view-source"]}
    </button>
    <button
      class="action-button button-history clickable"
      type="button"
      on:click={handleHistory}
    >
      {$page.data.internationalization?.history}
    </button>
  </div>
{/if}

{#if showSource}
  <textarea class="page-source" readonly={true}>{$page.data.wikitext}</textarea>
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
      placeholder={$page.data.internationalization?.["wiki-page-move-new-slug"]}
      type="text"
    />
    <textarea
      name="comments"
      class="page-move-comments"
      placeholder={$page.data.internationalization?.["wiki-page-revision-comments"]}
    />
    <div class="action-row page-move-actions">
      <button
        class="action-button page-move-button button-cancel clickable"
        type="button"
        on:click|stopPropagation={() => {
          $: showMoveAction = false
        }}
      >
        {$page.data.internationalization?.cancel}
      </button>
      <button
        class="action-button page-move-button button-move clickable"
        type="submit"
        on:click|stopPropagation
      >
        {$page.data.internationalization?.move}
      </button>
    </div>
  </form>
{/if}

{#if showHistory}
  <div class="revision-list">
    <div class="revision-header">
      <div class="revision-attribute action" />
      <div class="revision-attribute revision-number">
        {$page.data.internationalization?.["wiki-page-revision-number"]}
      </div>
      <div class="revision-attribute created-at">
        {$page.data.internationalization?.["wiki-page-revision-created-at"]}
      </div>
      <div class="revision-attribute user">
        {$page.data.internationalization?.["wiki-page-revision-user"]}
      </div>
      <div class="revision-attribute comments">
        {$page.data.internationalization?.["wiki-page-revision-comments"]}
      </div>
    </div>
    <!-- Here we sort the list in descending order. -->
    {#each [...revisionMap].sort((a, b) => b[0] - a[0]) as [_, revisionItem] (revisionItem.revision_number)}
      <div class="revision-row" data-id={revisionItem.revision_id}>
        <div class="revision-attribute action">
          <button
            class="action-button view-revision clickable"
            type="button"
            on:click|stopPropagation={() => {
              getRevision(revisionItem.revision_number, true, false).then(() => {
                showRevision = true
                showRevisionSource = false
              })
            }}
          >
            {$page.data.internationalization?.view}
          </button>
          <button
            class="action-button view-revision-source clickable"
            type="button"
            on:click|stopPropagation={() => {
              getRevision(revisionItem.revision_number, false, true).then(() => {
                showRevision = false
                showRevisionSource = true
              })
            }}
          >
            {$page.data.internationalization?.["wiki-page-view-source"]}
          </button>
        </div>
        <div class="revision-attribute revision-number">
          {revisionItem.revision_number}
        </div>
        <div class="revision-attribute created-at">
          {revisionItem.created_at}
        </div>
        <div class="revision-attribute user">
          {revisionItem.user_id}
        </div>
        <div class="revision-attribute comments">
          {revisionItem.comments}
        </div>
      </div>
    {/each}
  </div>

  {#if showRevisionSource}
    <textarea class="revision-source" readonly={true}>{revision.wikitext}</textarea>
  {/if}
{/if}

<style global lang="scss">
  .debug {
    width: 80vw;
    height: 60vh;
  }

  .page-content,
  .page-tags-container,
  .page-revision-container,
  .editor-actions,
  .other-actions,
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

  .page-revision-container {
    text-align: right;
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

  .page-source,
  .revision-source {
    width: 80vw;
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
