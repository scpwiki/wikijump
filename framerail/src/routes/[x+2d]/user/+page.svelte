<script lang="ts">
  import Page from "./[slug]/page.svelte"
  import { page } from "$app/stores"
  import { useErrorPopup } from "$lib/stores"
  let showErrorPopup = useErrorPopup()

  let isEdit = false
  let avatarFiles: FileList

  async function saveEdit() {
    let form = document.getElementById("editor")
    let fsrc = new FormData(form)
    let fdata = new FormData()
    for (let [key, val] of fsrc.entries()) {
      if (val !== $page.data.user[key]) fdata.set(key, val)
    }
    let res = await fetch(`/-/user`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())
    if (res?.message) {
      showErrorPopup.set({
        state: true,
        message: res.message
      })
    } else {
      isEdit = false
      $page.data.user = res
    }
  }
</script>

{#if isEdit}
  <h1>UNTRANSLATED: Loaded user profile</h1>

  <textarea class="debug">{JSON.stringify($page, null, 2)}</textarea>

  <form id="editor" class="editor" method="POST" on:submit|preventDefault={saveEdit}>
    <label for="name">{$page.data.internationalization?.["user-profile-info.name"]}</label
    >
    <input
      name="name"
      class="user-attribute name"
      type="text"
      value={$page.data.user.name}
    />
    <label for="real-name"
      >{$page.data.internationalization?.["user-profile-info.real-name"]}</label
    >
    <input
      name="real-name"
      class="user-attribute real-name"
      type="text"
      value={$page.data.user.real_name}
    />
    <label for="email"
      >{$page.data.internationalization?.["user-profile-info.email"]}</label
    >
    <input
      name="email"
      class="user-attribute email"
      type="text"
      value={$page.data.user.email}
    />
    <label for="avatar"
      >{$page.data.internationalization?.["user-profile-info.avatar"]}</label
    >
    <input
      name="avatar"
      class="user-attribute avatar"
      accept="image/png,image/jpeg,image/bmp"
      type="file"
      bind:files={avatarFiles}
    />
    <label for="gender"
      >{$page.data.internationalization?.["user-profile-info.gender"]}</label
    >
    <input
      name="gender"
      class="user-attribute gender"
      type="text"
      value={$page.data.user.gender}
    />
    <label for="birthday"
      >{$page.data.internationalization?.["user-profile-info.birthday"]}</label
    >
    <input
      name="birthday"
      class="user-attribute birthday"
      type="date"
      value={$page.data.user.birthday}
    />
    <label for="location"
      >{$page.data.internationalization?.["user-profile-info.location"]}</label
    >
    <input
      name="location"
      class="user-attribute location"
      type="text"
      value={$page.data.user.location}
    />
    <label for="user-page"
      >{$page.data.internationalization?.["user-profile-info.user-page"]}</label
    >
    <input
      name="user-page"
      class="user-attribute user-page"
      type="text"
      value={$page.data.user.user_page}
    />
    <label for="biography"
      >{$page.data.internationalization?.["user-profile-info.biography"]}</label
    >
    <input
      name="biography"
      class="user-attribute biography"
      type="text"
      value={$page.data.user.biography}
    />
    <label for="locales"
      >{$page.data.internationalization?.["user-profile-info.locales"]}</label
    >
    <input
      name="locales"
      class="user-attribute locales"
      type="text"
      value={$page.data.user.locales?.join(" ")}
    />
    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-cancel clickable"
        type="button"
        on:click|stopPropagation={() => {
          isEdit = false
        }}
      >
        {$page.data.internationalization?.cancel}
      </button>
      <button
        class="action-button editor-button button-save clickable"
        type="submit"
        on:click
      >
        {$page.data.internationalization?.save}
      </button>
    </div>
  </form>
{:else}
  <Page />

  <div class="action-row editor-actions">
    <button
      class="action-button editor-button button-edit clickable"
      type="button"
      on:click|stopPropagation={() => {
        isEdit = true
      }}
    >
      {$page.data.internationalization?.edit}
    </button>
  </div>
{/if}
