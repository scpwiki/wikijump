<script lang="ts">
  import Page from "./[slug]/PageView.svelte"

  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { acceptedSnapshot, partialPatch } from "$lib/user-profile-draft.js"
  import { invalidateAll } from "$app/navigation"
  import { fileProxy, superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"
  import type { userEditSchema } from "$lib/server/load/user"
  import type { InferOutput } from "valibot"

  let { data }: PageProps = $props()

  let isEdit = $state<boolean>(false)

  // avatar will always be undefined if not uploaded a new avatar
  type checkFormType = Omit<InferOutput<typeof userEditSchema>, "avatar">

  // The last accepted account state. It is the baseline for partial patches and
  // the value shown in the profile view, so it changes only after the server
  // confirms a save. Rejected drafts never become part of it.
  let lastSubmitted = $state<checkFormType>(untrack(() => acceptedSnapshot(data.user)))

  // Cancel and reopening the editor always start from the accepted state.
  const discardDraft = () => {
    $form = { ...lastSubmitted, avatar: undefined }
  }

  const { form, enhance } = superForm(
    untrack(() => data.userEditForm),
    {
      dataType: "json",
      onSubmit: ({ jsonData }) => {
        const { avatar, ...rest } = $form
        jsonData(partialPatch(rest, lastSubmitted, avatar))
      },
      onResult: async ({ result, cancel }) => {
        if (result.type === "success" && result.data) {
          isEdit = false
          await invalidateAll()
          cancel()
          // Adopt what the server persisted, including normalized values, rather
          // than the client payload.
          lastSubmitted = acceptedSnapshot(data.user)
          discardDraft()
        }
        if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data?.message,
            data: result.data?.data
          }
        }
      }
    }
  )
  const avatar = fileProxy(form, "avatar")

  // Only update the form once when page is loaded
  $form = untrack(() => ({
    ...lastSubmitted,
    avatar: undefined
  }))
</script>

{#if isEdit}
  <h1>{data.internationalization?.["user-profile-info"]}</h1>

  <form
    id="editor"
    class="editor"
    action="?/userEdit"
    enctype="multipart/form-data"
    method="POST"
    use:enhance
  >
    <label for="name">{data.internationalization?.["user-profile-info.name"]}</label>
    <input
      id="name"
      name="name"
      class="user-attribute name"
      type="text"
      bind:value={$form.name}
    />
    <label for="real-name"
      >{data.internationalization?.["user-profile-info.real-name"]}</label
    >
    <input
      id="real-name"
      name="realName"
      class="user-attribute real-name"
      type="text"
      bind:value={$form.realName}
    />
    <label for="email">{data.internationalization?.["user-profile-info.email"]}</label>
    <input
      id="email"
      name="email"
      class="user-attribute email"
      type="text"
      bind:value={$form.email}
    />
    <label for="avatar">{data.internationalization?.["user-profile-info.avatar"]}</label>
    <input
      id="avatar"
      name="avatar"
      class="user-attribute avatar"
      accept="image/png,image/jpeg,image/bmp"
      type="file"
      bind:files={$avatar}
    />
    <label for="gender">{data.internationalization?.["user-profile-info.gender"]}</label>
    <input
      id="gender"
      name="gender"
      class="user-attribute gender"
      type="text"
      bind:value={$form.gender}
    />
    <label for="birthday"
      >{data.internationalization?.["user-profile-info.birthday"]}</label
    >
    <input
      id="birthday"
      name="birthday"
      class="user-attribute birthday"
      type="date"
      bind:value={$form.birthday}
    />
    <label for="location"
      >{data.internationalization?.["user-profile-info.location"]}</label
    >
    <input
      id="location"
      name="location"
      class="user-attribute location"
      type="text"
      bind:value={$form.location}
    />
    <label for="website">{data.internationalization?.["user-profile-info.website"]}</label
    >
    <input
      id="website"
      name="website"
      class="user-attribute website"
      type="text"
      bind:value={$form.website}
    />
    <label for="user-page"
      >{data.internationalization?.["user-profile-info.user-page"]}</label
    >
    <input
      id="user-page"
      name="userPage"
      class="user-attribute user-page"
      type="text"
      bind:value={$form.userPage}
    />
    <label for="biography"
      >{data.internationalization?.["user-profile-info.biography"]}</label
    >
    <input
      id="biography"
      name="biography"
      class="user-attribute biography"
      type="text"
      bind:value={$form.biography}
    />
    <label for="locales">{data.internationalization?.["user-profile-info.locales"]}</label
    >
    <input
      id="locales"
      name="locales"
      class="user-attribute locales"
      type="text"
      bind:value={$form.locales}
    />
    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-cancel clickable"
        onclick={() => {
          isEdit = false
          discardDraft()
        }}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button class="action-button editor-button button-save clickable" type="submit">
        {data.internationalization?.save}
      </button>
    </div>
  </form>
{:else}
  <Page {data} userData={{ ...lastSubmitted, avatar: $form.avatar }} />

  <div class="action-row editor-actions">
    <button
      class="action-button editor-button button-edit clickable"
      onclick={() => (isEdit = true)}
      type="button"
    >
      {data.internationalization?.edit}
    </button>
  </div>
{/if}
