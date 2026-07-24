<script lang="ts">
  import type { PageFile } from "$lib/server/deepwell/page-file"
  import type { SvelteMap } from "svelte/reactivity"
  import type { PageProps } from "./$types"
  import type { FileAction } from "./file-pane-state"

  let {
    data,
    fileMap,
    activeFileAction = $bindable(),
    fileEditId = $bindable(),
    wikidot,
    getFileList,
    deleteFile,
    openFileHistory
  }: {
    data: PageProps["data"]
    fileMap: SvelteMap<number, PageFile>
    activeFileAction: FileAction | null
    fileEditId: number
    wikidot: boolean
    getFileList: (deleted?: boolean) => Promise<void>
    deleteFile: (fileId: number, lastRevisionId: number) => Promise<void>
    openFileHistory: (fileId: number) => void
  } = $props()

  function openFileAction(fileId: number, action: FileAction) {
    fileEditId = fileId
    activeFileAction = action
  }
</script>

{#if wikidot}
  <div class="buttons">
    <input
      class="btn btn-primary"
      onclick={() => (activeFileAction = "upload")}
      type="button"
      value={data.internationalization?.upload}
    />
    <input
      class="btn btn-default"
      onclick={() => void getFileList(true)}
      type="button"
      value={data.internationalization?.restore}
    />
  </div>
{:else}
  <div class="action-row file-action">
    <button
      class="action-button upload-file clickable"
      onclick={() => (activeFileAction = "upload")}
      type="button"
    >
      {data.internationalization?.upload}
    </button>
    <button
      class="action-button deleted-file clickable"
      onclick={() => void getFileList(true)}
      type="button"
    >
      {data.internationalization?.restore}
    </button>
  </div>
{/if}

{#if fileMap.size > 0}
  <div class="file-list">
    <div class="file-list-header">
      <div class="file-attribute name">
        {data.internationalization?.["wiki-page-file.name"]}
      </div>
      <div class="file-attribute created-at">
        {data.internationalization?.["wiki-page-file.created-at"]}
      </div>
      <div class="file-attribute updated-at">
        {data.internationalization?.["wiki-page-file.updated-at"]}
      </div>
      {#if !wikidot}
        <div class="file-attribute mime">
          {data.internationalization?.["wiki-page-file.mime"]}
        </div>
      {/if}
      <div class="file-attribute size">
        {data.internationalization?.["wiki-page-file.size"]}
      </div>
      <div class="file-attribute action"></div>
    </div>
    {#each [...fileMap].sort((a, b) => b[0] - a[0]) as [id, file] (id)}
      <div class="file-row" data-id={id}>
        <div class="file-attribute name">
          <a
            href={`//${data.site_file_domain}/-/file/${data.page?.slug}/${file.name}`}
            rel="external"
          >
            {file.name}
          </a>
        </div>
        <div class="file-attribute created-at">
          {new Date(file.file_created_at).toLocaleString()}
        </div>
        <div class="file-attribute updated-at">
          {file.file_updated_at ? new Date(file.file_updated_at).toLocaleString() : "-"}
        </div>
        {#if !wikidot}
          <div class="file-attribute mime">
            {file.mime}
          </div>
        {/if}
        <div class="file-attribute size">
          {file.size}
        </div>
        <div class="file-attribute action">
          {#if wikidot}
            {#if file.revision_type === "delete"}
              <!-- svelte-ignore a11y_invalid_attribute -->
              <a
                class="btn btn-primary btn-sm btn-small"
                href="javascript:;"
                onclick={() => openFileAction(file.file_id, "restore")}
              >
                {data.internationalization?.restore}
              </a>
            {:else}
              <!-- svelte-ignore a11y_invalid_attribute -->
              <a
                class="btn btn-primary btn-sm btn-small"
                href="javascript:;"
                onclick={() => openFileHistory(file.file_id)}
              >
                {data.internationalization?.history}
              </a>
              <!-- svelte-ignore a11y_invalid_attribute -->
              <a
                class="btn btn-primary btn-sm btn-small"
                href="javascript:;"
                onclick={() => openFileAction(file.file_id, "move")}
              >
                {data.internationalization?.move}
              </a>
              <!-- svelte-ignore a11y_invalid_attribute -->
              <a
                class="btn btn-primary btn-sm btn-small"
                href="javascript:;"
                onclick={() => openFileAction(file.file_id, "edit")}
              >
                {data.internationalization?.edit}
              </a>
              <!-- svelte-ignore a11y_invalid_attribute -->
              <a
                class="btn btn-primary btn-sm btn-small"
                href="javascript:;"
                onclick={() => void deleteFile(file.file_id, file.revision_id)}
              >
                {data.internationalization?.delete}
              </a>
            {/if}
          {:else if file.revision_type === "delete"}
            <button
              class="action-button restore-file clickable"
              onclick={() => openFileAction(file.file_id, "restore")}
              type="button"
            >
              {data.internationalization?.restore}
            </button>
          {:else}
            <button
              class="action-button file-history clickable"
              onclick={() => openFileHistory(file.file_id)}
              type="button"
            >
              {data.internationalization?.history}
            </button>
            <button
              class="action-button move-file clickable"
              onclick={() => openFileAction(file.file_id, "move")}
              type="button"
            >
              {data.internationalization?.move}
            </button>
            <button
              class="action-button edit-file clickable"
              onclick={() => openFileAction(file.file_id, "edit")}
              type="button"
            >
              {data.internationalization?.edit}
            </button>
            <button
              class="action-button delete-file clickable"
              onclick={() => void deleteFile(file.file_id, file.revision_id)}
              type="button"
            >
              {data.internationalization?.delete}
            </button>
          {/if}
        </div>
      </div>
    {/each}
  </div>
{:else}
  <div class="file-list">
    <div class="file-list-message">
      {data.internationalization?.["wiki-page-file-no-files"]}
    </div>
  </div>
{/if}

<style lang="scss">
  .file-list {
    display: table;
    width: 100%;
    padding: 0 0 2em;

    .file-list-header,
    .file-row {
      display: table-row;

      .file-attribute {
        display: table-cell;
      }
    }
  }
</style>
