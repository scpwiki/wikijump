<script lang="ts">
  import { deserialize } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
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

  async function loadFileHistory(fileId: number) {
    requestedHistoryRequestId = historyRequestId
    const res = await fetch("?/fileHistory", {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        fileId
      })
    }).then((response) => response.text())

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
    } else if (activeFileAction !== "history") {
      requestedHistoryRequestId = -1
    }
  })
</script>

{#if activeFileAction === "history"}
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
          {#if ["create", "regular"].includes(revisionItem.revision_type)}
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
          {revisionItem.page_id}
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
{/if}

<style lang="scss">
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
