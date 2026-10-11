<script lang="ts">
  import { deserialize } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { onDestroy } from "svelte"
  import { SvelteMap } from "svelte/reactivity"

  import type { PageFile } from "$lib/server/deepwell/page-file"
  import type { FileRevisionModel, Optional } from "$lib/types"
  import type { SvelteMap as SvelteMapType } from "svelte/reactivity"
  import type { PageProps } from "./$types"
  import type { FileAction } from "./file-pane-state"

  let {
    data,
    activeFileAction = $bindable(),
    fileEditId = $bindable(),
    fileMap,
    historyRequestId,
    wikidot,
    getFileList
  }: {
    data: PageProps["data"]
    activeFileAction: FileAction | null
    fileEditId: number
    fileMap: SvelteMapType<number, PageFile>
    historyRequestId: number
    wikidot: boolean
    getFileList: (deleted?: boolean) => Promise<void>
  } = $props()

  const fileRevisionMap = new SvelteMap<number, FileRevisionModel>()
  let requestedHistoryRequestId = $state(-1)
  // Only the most recently requested file history may update the pane. A
  // slower response for an earlier file must not replace the revisions or
  // Revert controls of the file the reader selected last.
  let fileHistoryRequestId = 0

  onDestroy(() => {
    fileHistoryRequestId += 1
  })

  let canRollbackFile = $state(false)
  // Accessible name for the revision-table scroll region, so a screen reader
  // can tell which pane's revisions it is walking into.
  let fileHistoryRegionLabel = $derived(
    data.internationalization?.["wiki-page-file-history"] ?? "File history"
  )

  async function loadFileHistory(fileId: number) {
    requestedHistoryRequestId = historyRequestId
    const requestId = ++fileHistoryRequestId
    const res = await fetch("?/fileHistory", {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        fileId
      })
    }).then((response) => response.text())

    if (requestId !== fileHistoryRequestId) return

    const result = deserialize<
      { res: FileRevisionModel[] },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      fileRevisionMap.clear()
      result.data.res.forEach((revision) => {
        fileRevisionMap.set(revision.revision_number, revision)
      })
      activeFileAction = "history"
    }
  }

  async function loadRollbackPermission(requestId: number) {
    canRollbackFile = false
    try {
      const res = await fetch("?/editPermission", {
        method: "POST",
        body: ""
      }).then((response) => response.text())

      const result = deserialize<
        { res: { can_edit: boolean } },
        { message: string; code: string; data: Record<string, unknown> }
      >(res)

      if (activeFileAction !== "history" || historyRequestId !== requestId) return
      canRollbackFile = result.type === "success" && result.data?.res?.can_edit === true
    } catch {
      // Permission lookup failures must not leave a rollback action available.
      if (activeFileAction === "history" && historyRequestId === requestId) {
        canRollbackFile = false
      }
    }
  }

  async function rollbackFileRevision(revisionNumber: number, comments?: string) {
    const res = await fetch("?/fileRollback", {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        fileId: fileEditId,
        revisionNumber,
        lastRevisionId: fileMap.get(fileEditId)?.revision_id,
        comments
      })
    }).then((response) => response.text())

    const result = deserialize<
      { res: Optional<PageFile> },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      activeFileAction = null
      requestedHistoryRequestId = -1
      fileRevisionMap.clear()
      await getFileList()
      await loadFileHistory(fileEditId)
      await invalidateAll()
    }
  }

  $effect(() => {
    if (
      activeFileAction === "history" &&
      fileEditId > 0 &&
      requestedHistoryRequestId !== historyRequestId
    ) {
      void loadFileHistory(fileEditId)
      void loadRollbackPermission(historyRequestId)
    } else if (activeFileAction !== "history") {
      requestedHistoryRequestId = -1
      canRollbackFile = false
    }
  })
</script>

{#if activeFileAction === "history"}
  <!--
    Source Wikidot renders the attachment File History as one wide row of
    columns, so a 320px phone cannot fit it. The Files list already contains
    its own `.file-list-scroll` region; without the equivalent containment
    here the table's intrinsic width (679px on a locally authored page, over
    1,000px on an imported article) leaks into the document and either adds
    whole-page sideways scrolling or is clipped by a theme that hides
    overflow, hiding revision metadata and the Revert controls (issue #2194).
    The region is a real, focusable scroll container so every column stays
    reachable by keyboard and touch.
  -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <div
    id="file-history-list"
    class="revision-list-scroll"
    role="region"
    aria-label={fileHistoryRegionLabel}
    tabindex="0"
  >
    <div class="revision-list">
      <div class="revision-header">
        <div class="revision-attribute action"></div>
        <div class="revision-attribute revision-number">
          {data.internationalization?.["wiki-page-revision-number"]}
        </div>
        <div class="revision-attribute revision-type">
          {data.internationalization?.["wiki-page-file-revision-type"]}
        </div>
        <div class="revision-attribute created-at">
          {data.internationalization?.["wiki-page-file.created-at"]}
        </div>
        <div class="revision-attribute user">
          {data.internationalization?.["wiki-page-revision-user"]}
        </div>
        <div class="revision-attribute page">
          {data.internationalization?.["wiki-page-file.page"]}
        </div>
        <div class="revision-attribute name">
          {data.internationalization?.["wiki-page-file.name"]}
        </div>
        <div class="revision-attribute mime">
          {data.internationalization?.["wiki-page-file.mime"]}
        </div>
        <div class="revision-attribute size">
          {data.internationalization?.["wiki-page-file.size"]}
        </div>
        <div class="revision-attribute comments">
          {data.internationalization?.["wiki-page-revision-comments"]}
        </div>
      </div>
      {#each [...fileRevisionMap].sort((a, b) => b[0] - a[0]) as [index, revisionItem] (index)}
        <div class="revision-row" data-id={revisionItem.revision_id}>
          <div class="revision-attribute action">
            {#if canRollbackFile && ["create", "regular"].includes(revisionItem.revision_type)}
              {#if wikidot}
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="btn btn-primary btn-sm btn-small"
                  href="javascript:;"
                  onclick={() => {
                    fileEditId = revisionItem.file_id
                    void rollbackFileRevision(revisionItem.revision_number)
                  }}
                >
                  {data.internationalization?.["wiki-page-revision-rollback"]}
                </a>
              {:else}
                <button
                  class="action-button revision-rollback clickable"
                  onclick={() => {
                    fileEditId = revisionItem.file_id
                    void rollbackFileRevision(revisionItem.revision_number)
                  }}
                  type="button"
                >
                  {data.internationalization?.["wiki-page-revision-rollback"]}
                </button>
              {/if}
            {/if}
          </div>
          <div class="revision-attribute revision-number">
            {revisionItem.revision_number}
          </div>
          <div class="revision-attribute revision-type">
            {data.internationalization?.[
              `wiki-page-file-revision-type.${revisionItem.revision_type}`
            ]}
          </div>
          <div class="revision-attribute created-at">
            {new Date(revisionItem.created_at).toLocaleString()}
          </div>
          <div class="revision-attribute user">
            {revisionItem.user_id}
          </div>
          <div class="revision-attribute page">
            {#if data.page?.page_id === revisionItem.page_id && data.site.site_id === revisionItem.site_id}
              <a href={`/${data.page.slug}`}>{data.page.slug}</a>
            {:else}
              <span>—</span>
            {/if}
          </div>
          <div class="revision-attribute name">
            {revisionItem.name}
          </div>
          <div class="revision-attribute mime">
            {revisionItem.mime}
          </div>
          <div class="revision-attribute size">
            {revisionItem.size}
          </div>
          <div class="revision-attribute comments">
            {revisionItem.comments}
          </div>
        </div>
      {/each}
    </div>
  </div>
{/if}

<style lang="scss">
  /*
    The revision table keeps its source `display: table` geometry, which does
    not establish a scroll container, so it is wrapped in a block-level
    `.revision-list-scroll` instead. Everything is expressed in CSS against
    the available width, so the containment re-applies by itself after async
    history loading, when switching files, and on viewport changes.
  */
  .revision-list-scroll {
    box-sizing: border-box;
    width: 100%;
    min-width: 0;
    max-width: 100%;
    overflow-x: auto;
    overscroll-behavior-x: contain;
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
