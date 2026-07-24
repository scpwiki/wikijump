<script lang="ts">
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { untrack } from "svelte"
  import { fileProxy, superForm } from "sveltekit-superforms"

  import type { PageProps } from "./$types"
  import type { FileAction } from "./file-pane-state"

  let {
    data,
    activeFileAction = $bindable(),
    wikidot,
    getFileList
  }: {
    data: PageProps["data"]
    activeFileAction: FileAction | null
    wikidot: boolean
    getFileList: (deleted?: boolean) => Promise<void>
  } = $props()

  const {
    form: uploadForm,
    enhance: uploadEnhance,
    reset: uploadReset
  } = superForm(
    untrack(() => data.forms.fileUploadForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$uploadForm,
          siteId: data.site.site_id,
          pageId: data.page?.page_id
        })
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
</script>

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
    {#if wikidot}
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

<style lang="scss">
  .file-upload {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 100%;
  }
</style>
