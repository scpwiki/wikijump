<script lang="ts">
  import { deserialize } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState, pageLayoutState } from "$lib/stores.svelte"
  import { Layout } from "$lib/types"
  import { SvelteMap } from "svelte/reactivity"
  import { fileProxy, superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"
  import type { PageFile, PageFileDelete } from "$lib/server/deepwell/pageFile"
  import type { FileRevisionModel, Optional } from "$lib/types"

  let { data }: PageProps = $props()

  type FileAction = "upload" | "edit" | "move" | "restore" | "history"
  let activeFileAction = $state<FileAction | null>(null)

  let fileMap = new SvelteMap<number, PageFile>()
  let fileEditId = $state<number>(0)
  let fileRevisionMap = new SvelteMap<number, FileRevisionModel>()

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

  const {
    form: uploadForm,
    enhance: uploadEnhance,
    reset: uploadReset
  } = superForm(
    untrack(() => data.forms.fileUploadForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        const submitForm = {
          ...$uploadForm,
          siteId: data.site.site_id,
          pageId: data.page?.page_id
        }
        jsonData(submitForm)
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          uploadReset()
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
  const uploadFile = fileProxy(uploadForm, "file")

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

  async function handleFileHistory(fileId: number) {
    const res = await fetch("?/fileHistory", {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        fileId
      })
    }).then((res) => res.text())

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
      result.data.res.forEach((rev) => {
        fileRevisionMap.set(rev.revision_number, rev)
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
    }).then((res) => res.text())

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
      await getFileList()
      activeFileAction = null
      fileRevisionMap.clear()
      await handleFileHistory(fileEditId)
      await invalidateAll()
    }
  }

  $effect(() => {
    getFileList(false)
  })
</script>

{#if pageLayoutState.current === Layout.WIKIDOT}
  <h1 class="page-file-header">
    {data.internationalization?.["wiki-page-file"]}
  </h1>
{:else}
  <h2 class="page-file-header">
    {data.internationalization?.["wiki-page-file"]}
  </h2>
{/if}

<div class="file-panel">
  {#if pageLayoutState.current === Layout.WIKIDOT}
    <div class="buttons">
      <input
        class="btn btn-primary"
        onclick={() => (activeFileAction = "upload")}
        type="button"
        value={data.internationalization?.upload}
      />
      <input
        class="btn btn-default"
        onclick={() => getFileList(true)}
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
        onclick={() => getFileList(true)}
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
        {#if pageLayoutState.current !== Layout.WIKIDOT}
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
          {#if pageLayoutState.current !== Layout.WIKIDOT}
            <div class="file-attribute mime">
              {file.mime}
            </div>
          {/if}
          <div class="file-attribute size">
            {file.size}
          </div>
          <div class="file-attribute action">
            {#if pageLayoutState.current === Layout.WIKIDOT}
              {#if file.revision_type === "delete"}
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="btn btn-primary btn-sm btn-small"
                  href="javascript:;"
                  onclick={() => {
                    fileEditId = file.file_id
                    activeFileAction = "restore"
                  }}
                >
                  {data.internationalization?.restore}
                </a>
              {:else}
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="btn btn-primary btn-sm btn-small"
                  href="javascript:;"
                  onclick={() => {
                    activeFileAction = "history"
                    handleFileHistory(file.file_id)
                  }}
                >
                  {data.internationalization?.history}
                </a>
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="btn btn-primary btn-sm btn-small"
                  href="javascript:;"
                  onclick={() => {
                    fileEditId = file.file_id
                    activeFileAction = "move"
                  }}
                >
                  {data.internationalization?.move}
                </a>
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="btn btn-primary btn-sm btn-small"
                  href="javascript:;"
                  onclick={() => {
                    fileEditId = file.file_id
                    activeFileAction = "edit"
                  }}
                >
                  {data.internationalization?.edit}
                </a>
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="btn btn-primary btn-sm btn-small"
                  href="javascript:;"
                  onclick={() => {
                    deleteFile(file.file_id, file.revision_id)
                  }}
                >
                  {data.internationalization?.delete}
                </a>
              {/if}
            {:else if file.revision_type === "delete"}
              <button
                class="action-button restore-file clickable"
                onclick={() => {
                  fileEditId = file.file_id
                  activeFileAction = "restore"
                }}
                type="button"
              >
                {data.internationalization?.restore}
              </button>
            {:else}
              <button
                class="action-button file-history clickable"
                onclick={() => {
                  activeFileAction = "history"
                  handleFileHistory(file.file_id)
                }}
                type="button"
              >
                {data.internationalization?.history}
              </button>
              <button
                class="action-button move-file clickable"
                onclick={() => {
                  fileEditId = file.file_id
                  activeFileAction = "move"
                }}
                type="button"
              >
                {data.internationalization?.move}
              </button>
              <button
                class="action-button edit-file clickable"
                onclick={() => {
                  fileEditId = file.file_id
                  activeFileAction = "edit"
                }}
                type="button"
              >
                {data.internationalization?.edit}
              </button>
              <button
                class="action-button delete-file clickable"
                onclick={() => deleteFile(file.file_id, file.revision_id)}
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

  {#if activeFileAction === "upload"}
    <form
      id="file-upload"
      class="file-upload"
      action="?/fileUpload"
      enctype="multipart/form-data"
      method="POST"
      use:uploadEnhance
    >
      <div class="file-form-field">
        <label for="file">
          {data.internationalization?.["wiki-page-file-upload.select"]}
        </label>
        <input
          name="file"
          class="file-attribute file"
          type="file"
          bind:files={$uploadFile}
        />
      </div>
      <div class="file-form-field">
        <label for="name">
          {data.internationalization?.["wiki-page-file-upload.name"]}
        </label>
        <input
          name="name"
          class="file-attribute name"
          placeholder={$uploadFile?.[0]?.name}
          type="text"
          bind:value={$uploadForm.name}
        />
      </div>
      <textarea
        name="comments"
        class="file-form-field file-comments"
        placeholder={data.internationalization?.["wiki-page-revision-comments"]}
        bind:value={$uploadForm.comments}></textarea>
      {#if pageLayoutState.current === Layout.WIKIDOT}
        <div class="buttons">
          <input
            class="btn btn-default"
            onclick={() => {
              uploadReset()
              activeFileAction = null
            }}
            type="button"
            value={data.internationalization?.cancel}
          />
          <input
            class="btn btn-primary"
            type="submit"
            value={data.internationalization?.upload}
          />
        </div>
      {:else}
        <div class="action-row file-upload-actions">
          <button
            class="action-button file-upload-button button-cancel clickable"
            onclick={() => {
              uploadReset()
              activeFileAction = null
            }}
            type="button"
          >
            {data.internationalization?.cancel}
          </button>
          <button
            class="action-button file-upload-button button-upload clickable"
            type="submit"
          >
            {data.internationalization?.upload}
          </button>
        </div>
      {/if}
    </form>
  {/if}

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
      {#if pageLayoutState.current === Layout.WIKIDOT}
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
      {#if pageLayoutState.current === Layout.WIKIDOT}
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
      {#if pageLayoutState.current === Layout.WIKIDOT}
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
      <!-- Here we sort the list in descending order. -->
      {#each [...fileRevisionMap].sort((a, b) => b[0] - a[0]) as [index, revisionItem] (index)}
        <div class="revision-row" data-id={revisionItem.revision_id}>
          <div class="revision-attribute action">
            {#if ["create", "regular"].includes(revisionItem.revision_type)}
              {#if pageLayoutState.current === Layout.WIKIDOT}
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  class="btn btn-primary btn-sm btn-small"
                  href="javascript:;"
                  onclick={() => {
                    fileEditId = revisionItem.file_id
                    rollbackFileRevision(revisionItem.revision_number)
                  }}
                >
                  {data.internationalization?.["wiki-page-revision-rollback"]}
                </a>
              {:else}
                <button
                  class="action-button revision-rollback clickable"
                  onclick={() => {
                    fileEditId = revisionItem.file_id
                    rollbackFileRevision(revisionItem.revision_number)
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
</div>

<style lang="scss">
  .file-upload,
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
