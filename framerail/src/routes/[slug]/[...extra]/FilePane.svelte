<script lang="ts">
  import { deserialize } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { Layout } from "$lib/types"
  import { SvelteMap } from "svelte/reactivity"
  import { fileProxy, superForm } from "sveltekit-superforms"

  import FileHistoryPanel from "./FileHistoryPanel.svelte"
  import FileList from "./FileList.svelte"
  import FileUploadPanel from "./FileUploadPanel.svelte"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"
  import type { PageFile, PageFileDelete } from "$lib/server/deepwell/page-file"
  import type { FileAction } from "./file-pane-state"

  let { data }: PageProps = $props()

  const pageLayoutContext = getPageLayoutContext()

  let activeFileAction = $state<FileAction | null>(null)

  let fileMap = new SvelteMap<number, PageFile>()
  let fileEditId = $state<number>(0)
  let historyRequestId = $state(0)

  function openFileHistory(fileId: number) {
    fileEditId = fileId
    activeFileAction = "history"
    historyRequestId += 1
  }

  async function getFileList(deleted = false) {
    const res = await fetch("?/fileList", {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        deleted
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: PageFile[] },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      fileMap.clear()
      result.data.res.forEach((file: PageFile) => {
        fileMap.set(file.file_id, file)
      })
    }
  }

  async function deleteFile(fileId: number, lastRevisionId: number) {
    const res = await fetch("?/fileDelete", {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        fileId,
        lastRevisionId
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: PageFileDelete },
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
      await getFileList()
    }
  }

  const {
    form: editForm,
    enhance: editEnhance,
    reset: editReset
  } = superForm(
    untrack(() => data.forms.fileEditForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$editForm,
          siteId: data.site.site_id,
          pageId: data.page?.page_id,
          fileId: fileEditId,
          lastRevisionId: fileMap.get(fileEditId)?.revision_id
        }
        jsonData(submitForm)
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          activeFileAction = null
          await getFileList()
        }
        if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data.message,
            data: result.data.data
          }
        }
      }
    }
  )
  const editFile = fileProxy(editForm, "file")

  const {
    form: moveForm,
    enhance: moveEnhance,
    reset: moveReset
  } = superForm(
    untrack(() => data.forms.fileMoveForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$moveForm,
          siteId: data.site.site_id,
          pageId: data.page?.page_id,
          fileId: fileEditId,
          lastRevisionId: fileMap.get(fileEditId)?.revision_id
        }
        jsonData(submitForm)
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          activeFileAction = null
          await getFileList()
        }
        if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data.message,
            data: result.data.data
          }
        }
      }
    }
  )

  const {
    form: restoreForm,
    enhance: restoreEnhance,
    reset: restoreReset
  } = superForm(
    untrack(() => data.forms.fileRestoreForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$restoreForm,
          siteId: data.site.site_id,
          pageId: data.page?.page_id,
          fileId: fileEditId
        }
        jsonData(submitForm)
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          activeFileAction = null
          await getFileList()
          await invalidateAll()
        }
        if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data.message,
            data: result.data.data
          }
        }
      }
    }
  )

  $effect(() => {
    getFileList(false)
  })
</script>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  <h1 class="page-file-header">
    {data.internationalization?.["wiki-page-file"]}
  </h1>
{:else}
  <h2 class="page-file-header">
    {data.internationalization?.["wiki-page-file"]}
  </h2>
{/if}

<div class="file-panel">
  <FileList
    {data}
    {deleteFile}
    {fileMap}
    {getFileList}
    {openFileHistory}
    wikidot={pageLayoutContext.current === Layout.WIKIDOT}
    bind:activeFileAction
    bind:fileEditId
  />

  <FileUploadPanel
    {data}
    {getFileList}
    wikidot={pageLayoutContext.current === Layout.WIKIDOT}
    bind:activeFileAction
  />

  {#if activeFileAction === "edit"}
    <form
      id="file-edit"
      class="file-edit"
      action="?/fileEdit"
      enctype="multipart/form-data"
      method="POST"
      use:editEnhance
    >
      <div class="file-form-field">
        <label for="file">
          {data.internationalization?.["wiki-page-file-upload.select"]}
        </label>
        <input
          name="file"
          class="file-attribute file"
          type="file"
          bind:files={$editFile}
        />
      </div>
      <div class="file-form-field">
        <label for="name">
          {data.internationalization?.["wiki-page-file-upload.name"]}
        </label>
        <input
          name="name"
          class="file-attribute name"
          placeholder={fileMap.get(fileEditId)?.name}
          type="text"
          bind:value={$editForm.name}
        />
      </div>
      <textarea
        name="comments"
        class="file-form-field file-comments"
        placeholder={data.internationalization?.["wiki-page-revision-comments"]}
        bind:value={$editForm.comments}></textarea>
      {#if pageLayoutContext.current === Layout.WIKIDOT}
        <div class="buttons">
          <input
            class="btn btn-default"
            onclick={() => {
              editReset()
              activeFileAction = null
            }}
            type="button"
            value={data.internationalization?.cancel}
          />
          <input
            class="btn btn-primary"
            type="submit"
            value={data.internationalization?.save}
          />
        </div>
      {:else}
        <div class="action-row file-edit-actions">
          <button
            class="action-button file-edit-button button-cancel clickable"
            onclick={() => {
              editReset()
              activeFileAction = null
            }}
            type="button"
          >
            {data.internationalization?.cancel}
          </button>
          <button
            class="action-button file-edit-button button-save clickable"
            type="submit"
          >
            {data.internationalization?.save}
          </button>
        </div>
      {/if}
    </form>
  {/if}

  {#if activeFileAction === "move"}
    <form
      id="file-move"
      class="file-move"
      action="?/fileMove"
      method="POST"
      use:moveEnhance
    >
      <input
        name="destinationPage"
        class="file-move-destination-page"
        placeholder={data.internationalization?.["wiki-page-file-move-destination-page"]}
        type="text"
        bind:value={$moveForm.destinationPage}
      />
      <textarea
        name="comments"
        class="file-move-comments"
        placeholder={data.internationalization?.["wiki-page-revision-comments"]}
        bind:value={$moveForm.comments}></textarea>
      {#if pageLayoutContext.current === Layout.WIKIDOT}
        <div class="buttons">
          <input
            class="btn btn-default"
            onclick={() => {
              moveReset()
              activeFileAction = null
            }}
            type="button"
            value={data.internationalization?.cancel}
          />
          <input
            class="btn btn-primary"
            type="submit"
            value={data.internationalization?.move}
          />
        </div>
      {:else}
        <div class="action-row file-move-actions">
          <button
            class="action-button file-move-button button-cancel clickable"
            onclick={() => {
              moveReset()
              activeFileAction = null
            }}
            type="button"
          >
            {data.internationalization?.cancel}
          </button>
          <button
            class="action-button file-move-button button-move clickable"
            type="submit"
          >
            {data.internationalization?.move}
          </button>
        </div>
      {/if}
    </form>
  {/if}

  {#if activeFileAction === "restore"}
    <form
      id="file-restore"
      class="file-restore"
      action="?/fileRestore"
      method="POST"
      use:restoreEnhance
    >
      <input
        name="newPage"
        class="file-restore-new-page"
        placeholder={data.internationalization?.["wiki-page-file-restore.new-page"]}
        type="text"
        bind:value={$restoreForm.newPage}
      />
      <input
        name="newName"
        class="file-restore-new-name"
        placeholder={data.internationalization?.["wiki-page-file-restore.new-name"]}
        type="text"
        bind:value={$restoreForm.newName}
      />
      <textarea
        name="comments"
        class="file-restore-comments"
        placeholder={data.internationalization?.["wiki-page-revision-comments"]}
        bind:value={$restoreForm.comments}></textarea>
      {#if pageLayoutContext.current === Layout.WIKIDOT}
        <div class="buttons">
          <input
            class="btn btn-default"
            onclick={() => {
              restoreReset()
              activeFileAction = null
            }}
            type="button"
            value={data.internationalization?.cancel}
          />
          <input
            class="btn btn-primary"
            type="submit"
            value={data.internationalization?.restore}
          />
        </div>
      {:else}
        <div class="action-row file-restore-actions">
          <button
            class="action-button file-restore-button button-cancel clickable"
            onclick={() => {
              restoreReset()
              activeFileAction = null
            }}
            type="button"
          >
            {data.internationalization?.cancel}
          </button>
          <button
            class="action-button file-restore-button button-restore clickable"
            type="submit"
          >
            {data.internationalization?.restore}
          </button>
        </div>
      {/if}
    </form>
  {/if}

  <FileHistoryPanel
    {data}
    {fileMap}
    {getFileList}
    {historyRequestId}
    wikidot={pageLayoutContext.current === Layout.WIKIDOT}
    bind:activeFileAction
    bind:fileEditId
  />
</div>

<style lang="scss">
  .file-edit,
  .file-move,
  .file-restore {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 100%;
  }
</style>
