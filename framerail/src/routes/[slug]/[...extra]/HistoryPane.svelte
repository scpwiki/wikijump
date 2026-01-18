<script lang="ts">
  import { page } from "$app/stores"
  import { invalidateAll } from "$app/navigation"
  import { onMount } from "svelte"
  import { useErrorPopup, usePageLayoutState } from "$lib/stores"
  import { Layout } from "$lib/types"
  let showErrorPopup = useErrorPopup()
  let pageLayout = usePageLayoutState()
  let revisionMap: Map<number, Record<string, any>> = new Map()
  let revision: Record<string, any> = {}
  let showRevisionSource = false
  export let setShowRevision: (val: boolean) => void
  export let setRevision: (rev: Record<string, any>) => void

  async function getRevision(
    revisionNumber: number,
    compiledHtml: boolean,
    wikitext: boolean
  ) {
    // Get cached revision if we have it
    let rev = revisionMap.get(revisionNumber)
    // Try to see if the cached revision already has the wanted data
    if (compiledHtml && rev?.compiled_body_html) {
      setRevision(rev)
      revision = rev
    } else if (wikitext && rev?.wikitext) {
      setRevision(rev)
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
          message: res.message,
          data: res.data
        })
      } else if (!rev) {
        // This is a revision we didn't even cache...?
        revisionMap.set(res.revision_number, res)
        setRevision(res)
      } else if (compiledHtml) {
        rev.compiled_body_html = res.compiled_body_html
        setRevision(rev)
        revision = rev
      } else if (wikitext) {
        rev.wikitext = res.wikitext
        setRevision(rev)
        revision = rev
      }
    }
  }

  async function rollbackRevision(revisionNumber: number, comments?: string) {
    let fdata = new FormData()
    fdata.set("site-id", $page.data.site.site_id)
    fdata.set("page-id", $page.data.page.page_id)
    fdata.set("revision-number", revisionNumber)
    fdata.set("last-revision-id", $page.data.page_revision.revision_id)
    if (comments !== undefined) fdata.set("comments", comments)
    let res = await fetch(`/${$page.data.page.slug}/rollback`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message,
        data: res.data
      })
    } else invalidateAll()
  }

  onMount(async () => {
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
        message: res.message,
        data: res.data
      })
    } else {
      revisionMap = new Map()
      res.forEach((rev) => {
        revisionMap.set(rev.revision_number, rev)
      })
    }
  })
</script>

{#if $pageLayout === Layout.WIKIDOT}
  <h1 class="page-revision-header">
    {$page.data.internationalization?.["wiki-page-revision-history"]}
  </h1>
  <div class="revision-list">
    <table class="page-history">
      <tr class="revision-header">
        <td class="revision-attribute revision-number">
          {$page.data.internationalization?.["wiki-page-revision-number"]}
        </td>
        <td class="revision-attribute action" />
        <td class="revision-attribute revision-type">
          {$page.data.internationalization?.["wiki-page-revision-type"]}
        </td>
        <td class="revision-attribute user">
          {$page.data.internationalization?.["wiki-page-revision-user"]}
        </td>
        <td class="revision-attribute created-at">
          {$page.data.internationalization?.["wiki-page-revision-created-at"]}
        </td>
        <td class="revision-attribute comments">
          {$page.data.internationalization?.["wiki-page-revision-comments"]}
        </td>
      </tr>
      <!-- Here we sort the list in descending order. -->
      {#each [...revisionMap].sort((a, b) => b[0] - a[0]) as [_, revisionItem] (revisionItem.revision_number)}
        <tr
          id={`revision-row-${revisionItem.revision_id}`}
          class="revision-row"
          data-id={revisionItem.revision_id}
        >
          <td class="revision-attribute revision-number">
            {revisionItem.revision_number}
          </td>
          <td class="revision-attribute action optionstd">
            {#if ["create", "regular"].includes(revisionItem.revision_type)}
              <!-- svelte-ignore a11y-invalid-attribute -->
              <a
                class="view-revision"
                href="javascript:;"
                type="button"
                on:click|stopPropagation={() => {
                  getRevision(revisionItem.revision_number, true, false).then(() => {
                    setShowRevision(true)
                    showRevisionSource = false
                  })
                }}>V</a
              >
              <!-- svelte-ignore a11y-invalid-attribute -->
              <a
                class="view-revision-source"
                href="javascript:;"
                type="button"
                on:click|stopPropagation={() => {
                  getRevision(revisionItem.revision_number, false, true).then(() => {
                    setShowRevision(false)
                    showRevisionSource = true
                  })
                }}>S</a
              >
              <!-- svelte-ignore a11y-invalid-attribute -->
              <a
                class="revision-rollback"
                href="javascript:;"
                type="button"
                on:click|stopPropagation={() => {
                  rollbackRevision(revisionItem.revision_number)
                }}>R</a
              >
            {/if}
          </td>
          <td class="revision-attribute revision-type">
            {$page.data.internationalization?.[
              `wiki-page-revision-type.${revisionItem.revision_type}`
            ]}
          </td>
          <td class="revision-attribute user">
            {revisionItem.user_id}
          </td>
          <td class="revision-attribute created-at">
            {new Date(revisionItem.created_at).toLocaleString()}
          </td>
          <td class="revision-attribute comments">
            {revisionItem.comments}
          </td>
        </tr>
      {/each}
    </table>
  </div>

  {#if showRevisionSource}
    <div id="history-subarea">
      <textarea class="page-source" readonly={true}>{revision.wikitext ?? ""}</textarea>
    </div>
  {/if}
{:else}
  <h2 class="page-revision-header">
    {$page.data.internationalization?.["wiki-page-revision-history"]}
  </h2>
  <div class="revision-list">
    <div class="revision-header">
      <div class="revision-attribute action" />
      <div class="revision-attribute revision-number">
        {$page.data.internationalization?.["wiki-page-revision-number"]}
      </div>
      <div class="revision-attribute revision-type">
        {$page.data.internationalization?.["wiki-page-revision-type"]}
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
          {#if ["create", "regular"].includes(revisionItem.revision_type)}
            <button
              class="action-button view-revision clickable"
              type="button"
              on:click|stopPropagation={() => {
                getRevision(revisionItem.revision_number, true, false).then(() => {
                  setShowRevision(true)
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
                  setShowRevision(false)
                  showRevisionSource = true
                })
              }}
            >
              {$page.data.internationalization?.["wiki-page-view-source"]}
            </button>
            <button
              class="action-button revision-rollback clickable"
              type="button"
              on:click|stopPropagation={() => {
                rollbackRevision(revisionItem.revision_number)
              }}
            >
              {$page.data.internationalization?.["wiki-page-revision-rollback"]}
            </button>
          {/if}
        </div>
        <div class="revision-attribute revision-number">
          {revisionItem.revision_number}
        </div>
        <div class="revision-attribute revision-type">
          {$page.data.internationalization?.[
            `wiki-page-revision-type.${revisionItem.revision_type}`
          ]}
        </div>
        <div class="revision-attribute created-at">
          {new Date(revisionItem.created_at).toLocaleString()}
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
    <textarea class="revision-source" readonly={true}>{revision.wikitext ?? ""}</textarea>
  {/if}
{/if}

<style lang="scss">
  textarea.revision-source {
    width: 100%;
    height: 60vh;
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
