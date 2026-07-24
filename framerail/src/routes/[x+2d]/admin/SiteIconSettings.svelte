<script lang="ts">
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data }: { data: PageProps["data"] } = $props()

  const { form, enhance } = superForm(
    untrack(() => data.siteIconsForm),
    {
      dataType: "json",
      resetForm: false,
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$form,
          siteId: data.site.site_id
        })
      },
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data?.res) {
          data.site = result.data.res
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

  $effect(() => {
    $form.siteId = data.site.site_id
    $form.faviconSource = data.site.favicon_source ?? ""
    $form.iosIconSource = data.site.ios_icon_source ?? ""
    $form.windowsTileSource = data.site.windows_tile_source ?? ""
  })
</script>

<section id="site-icons" class="admin-section">
  <h2>Icons</h2>
  <p>
    Set this site's favicon, iOS icon, and Windows 8 tile. Each accepts the address of an
    existing image. Leave a field empty to declare no icon.
  </p>

  <form class="editor site-icons-editor" action="?/siteIcons" method="POST" use:enhance>
    <label for="favicon-source">Favicon</label>
    <input
      id="favicon-source"
      name="faviconSource"
      type="text"
      bind:value={$form.faviconSource}
    />
    <label for="ios-icon-source">iOS icon</label>
    <input
      id="ios-icon-source"
      name="iosIconSource"
      type="text"
      bind:value={$form.iosIconSource}
    />
    <label for="windows-tile-source">Windows 8 tile</label>
    <input
      id="windows-tile-source"
      name="windowsTileSource"
      type="text"
      bind:value={$form.windowsTileSource}
    />
    <input name="siteId" type="hidden" bind:value={$form.siteId} />
    <div class="action-row editor-actions">
      <button
        id="sm-site-icons-save"
        class="action-button editor-button button-save clickable"
        type="submit"
      >
        Save
      </button>
    </div>
  </form>
</section>
