<script lang="ts">
  import { deserialize } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"

  import { Layout } from "$lib/types"
  import { SvelteMap } from "svelte/reactivity"

  import type { PageProps } from "./$types"
  import type {
    PageRevisionModelFiltered,
    CreatePageRevisionOutput
  } from "$lib/server/deepwell/page"
  import type { Optional } from "$lib/types"

  interface Props extends PageProps {
    setShowRevision: (val: boolean) => void
    setRevision: (rev: Optional<PageRevisionModelFiltered>) => void
  }

  let { setShowRevision, setRevision, data }: Props = $props()

  const pageLayoutContext = getPageLayoutContext()

  let revisionMap = new SvelteMap<number, PageRevisionModelFiltered>()
  let revision = $state<Optional<PageRevisionModelFiltered>>(undefined)
  let showRevisionSource = $state<boolean>(false)

  async function fetchHistory() {
    const res = await fetch("?/history", {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: PageRevisionModelFiltered[] },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      revisionMap.clear()
      result.data.res.forEach((rev) => {
        revisionMap.set(rev.revision_number, rev)
      })
    }
  }

  async function getRevision(
    revisionNumber: number,
    compiledHtml: boolean,
    wikitext: boolean
  ) {
    const rev = revisionMap.get(revisionNumber)
    if (compiledHtml && rev?.compiled_body_html) {
      setRevision(rev)
      revision = rev
    } else if (wikitext && rev?.wikitext) {
      setRevision(rev)
      revision = rev
    } else {
      const res = await fetch("?/revision", {
        method: "POST",
        body: JSON.stringify({
          siteId: data.site.site_id,
          pageId: data.page?.page_id,
          revisionNumber,
          compiledHtml,
          wikitext
        })
      }).then((res) => res.text())

      const result = deserialize<
        { res: Optional<PageRevisionModelFiltered> },
        { message: string; code: string; data: Record<string, unknown> }
      >(res)

      if (result.type === "failure" && result.data?.message) {
        errorPopupState.current = {
          state: true,
          message: result.data.message,
          data: result.data
        }
      } else if (result.type === "success" && result.data?.res) {
        if (!rev) {
          // This is a revision we didn't even cache...?
          revisionMap.set(revisionNumber, result.data.res)
          setRevision(result.data.res)
        } else if (compiledHtml) {
          rev.compiled_body_html = result.data.res.compiled_body_html
          setRevision(rev)
          revision = rev
        } else if (wikitext) {
          rev.wikitext = result.data.res.wikitext
          setRevision(rev)
          revision = rev
        }
      }
    }
  }

  async function rollbackRevision(revisionNumber: number, comments?: string) {
    const res = await fetch("?/rollback", {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        revisionNumber,
        lastRevisionId: data.page_revision?.revision_id,
        comments
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: Optional<CreatePageRevisionOutput> },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      invalidateAll()
    }
  }

  $effect(() => {
    fetchHistory()
  })
</script>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  <h1 class="page-revision-header">
    {data.internationalization?.["wiki-page-revision-history"]}
  </h1>
  <div class="revision-list">
    <table class="page-history">
      <tbody>
        <tr class="revision-header">
          <td class="revision-attribute revision-number">
            {data.internationalization?.["wiki-page-revision-number"]}
          </td>
          <td class="revision-attribute action"></td>
          <td class="revision-attribute revision-type">
            {data.internationalization?.["wiki-page-revision-type"]}
          </td>
          <td class="revision-attribute user">
            {data.internationalization?.["wiki-page-revision-user"]}
          </td>
          <td class="revision-attribute created-at">
            {data.internationalization?.["wiki-page-revision-created-at"]}
          </td>
          <td class="revision-attribute comments">
            {data.internationalization?.["wiki-page-revision-comments"]}
          </td>
        </tr>
        <!-- Here we sort the list in descending order. -->
        {#each [...revisionMap].sort((a, b) => b[0] - a[0]) as [, revisionItem] (revisionItem.revision_number)}
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
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="view-revision"
                  href="javascript:;"
                  onclick={(event) => {
                    event.stopPropagation()
                    getRevision(revisionItem.revision_number, true, false).then(() => {
                      setShowRevision(true)
                      showRevisionSource = false
                    })
                  }}
                  type="button"
                >
                  V
                </a>
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="view-revision-source"
                  href="javascript:;"
                  onclick={(event) => {
                    event.stopPropagation()
                    getRevision(revisionItem.revision_number, false, true).then(() => {
                      setShowRevision(false)
                      showRevisionSource = true
                    })
                  }}
                  type="button"
                >
                  S
                </a>
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="revision-rollback"
                  href="javascript:;"
                  onclick={(event) => {
                    event.stopPropagation()
                    rollbackRevision(revisionItem.revision_number)
                  }}
                  type="button"
                >
                  R
                </a>
              {/if}
            </td>
            <td class="revision-attribute revision-type">
              {data.internationalization?.[
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
      </tbody>
    </table>
  </div>

  {#if showRevisionSource}
    <div id="history-subarea">
      <textarea class="page-source" readonly={true}>{revision?.wikitext ?? ""}</textarea>
    </div>
  {/if}
{:else}
  <h2 class="page-revision-header">
    {data.internationalization?.["wiki-page-revision-history"]}
  </h2>
  <div class="revision-list">
    <div class="revision-header">
      <div class="revision-attribute action"></div>
      <div class="revision-attribute revision-number">
        {data.internationalization?.["wiki-page-revision-number"]}
      </div>
      <div class="revision-attribute revision-type">
        {data.internationalization?.["wiki-page-revision-type"]}
      </div>
      <div class="revision-attribute created-at">
        {data.internationalization?.["wiki-page-revision-created-at"]}
      </div>
      <div class="revision-attribute user">
        {data.internationalization?.["wiki-page-revision-user"]}
      </div>
      <div class="revision-attribute comments">
        {data.internationalization?.["wiki-page-revision-comments"]}
      </div>
    </div>
    <!-- Here we sort the list in descending order. -->
    {#each [...revisionMap].sort((a, b) => b[0] - a[0]) as [, revisionItem] (revisionItem.revision_number)}
      <div class="revision-row" data-id={revisionItem.revision_id}>
        <div class="revision-attribute action">
          {#if ["create", "regular"].includes(revisionItem.revision_type)}
            <button
              class="action-button view-revision clickable"
              onclick={(event) => {
                event.stopPropagation()
                getRevision(revisionItem.revision_number, true, false).then(() => {
                  setShowRevision(true)
                  showRevisionSource = false
                })
              }}
              type="button"
            >
              {data.internationalization?.view}
            </button>
            <button
              class="action-button view-revision-source clickable"
              onclick={(event) => {
                event.stopPropagation()
                getRevision(revisionItem.revision_number, false, true).then(() => {
                  setShowRevision(false)
                  showRevisionSource = true
                })
              }}
              type="button"
            >
              {data.internationalization?.["wiki-page-view-source"]}
            </button>
            <button
              class="action-button revision-rollback clickable"
              onclick={(event) => {
                event.stopPropagation()
                rollbackRevision(revisionItem.revision_number)
              }}
              type="button"
            >
              {data.internationalization?.["wiki-page-revision-rollback"]}
            </button>
          {/if}
        </div>
        <div class="revision-attribute revision-number">
          {revisionItem.revision_number}
        </div>
        <div class="revision-attribute revision-type">
          {data.internationalization?.[
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
    <textarea class="revision-source" readonly={true}>{revision?.wikitext ?? ""}</textarea
    >
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
