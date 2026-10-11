<script lang="ts">
  import { goto } from "$app/navigation"
  import { deserialize } from "$app/forms"
  import { resolve } from "$app/paths"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { buildWikidotSectionEditSubmission } from "$lib/wikidot/wikidot-edit-sections"
  import { editPermissionDeniedMessage } from "$lib/wikidot/wikidot-locale"
  import { superForm } from "sveltekit-superforms"
  import { onMount, untrack } from "svelte"

  import type { PageProps } from "./$types"

  type EditSection = {
    index: number
    level: number
    start: number
    end: number
  }

  let {
    data,
    params,
    section,
    close
  }: PageProps & { section: EditSection; close: () => void } = $props()
  const originalSource = untrack(() => data.wikitext)
  const selectedSection = untrack(() => section)
  let permissionState = $state<"checking" | "allowed" | "denied">("checking")

  onMount(() => {
    let active = true

    const checkEditPermission = async () => {
      try {
        const response = await fetch("?/editPermission", {
          method: "POST",
          body: ""
        }).then((res) => res.text())
        const result = deserialize<
          { res: { can_edit: boolean } },
          { message: string; code: string; data: Record<string, unknown> }
        >(response)

        if (!active) return
        if (result.type === "success" && result.data?.res?.can_edit === true) {
          permissionState = "allowed"
          return
        }

        permissionState = "denied"
        errorPopupState.current = {
          state: true,
          message:
            result.type === "failure" && result.data?.message
              ? result.data.message
              : editPermissionDeniedMessage(data.site.locale),
          data: result.type === "failure" ? result.data : null
        }
      } catch {
        if (!active) return
        permissionState = "denied"
        errorPopupState.current = {
          state: true,
          message: editPermissionDeniedMessage(data.site.locale),
          data: null
        }
      }
    }

    void checkEditPermission()
    return () => {
      active = false
    }
  })

  const { form, enhance } = superForm(
    untrack(() => data.forms.pageEditForm),
    {
      dataType: "json",
      onSubmit: ({ jsonData }) => {
        jsonData(
          buildWikidotSectionEditSubmission(
            $form,
            {
              siteId: data.site.site_id,
              pageId: data.page?.page_id,
              revisionId: data.page_revision?.revision_id,
              title: data.page_revision?.title ?? "",
              altTitle: data.page_revision?.alt_title ?? "",
              tags: data.page_revision?.tags?.join(" ") ?? "",
              source: originalSource
            },
            selectedSection,
            $form.wikitext
          )
        )
      },
      onResult: async ({ result, cancel }) => {
        if (result.type === "success" && result.data) {
          cancel()
          goto(resolve(`/${params.slug}`, {}), { noScroll: true })
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

  $form.title = untrack(() => data.page_revision?.title ?? "")
  $form.altTitle = untrack(() => data.page_revision?.alt_title ?? "")
  $form.tags = untrack(() => data.page_revision?.tags?.join(" ") ?? "")
  $form.comments = untrack(() => data.page_revision?.comments ?? "")
  $form.wikitext = originalSource.slice(selectedSection.start, selectedSection.end)
</script>

<div id="edit-section-content">
  <h1 class="page-edit-header">Edit Section</h1>
  {#if permissionState === "checking"}
    <p role="status">Checking edit permission…</p>
  {:else if permissionState === "allowed"}
    <form id="editor" class="editor" action="?/edit" method="POST" use:enhance>
      <textarea name="wikitext" class="editor-wikitext" bind:value={$form.wikitext}
      ></textarea>
      <textarea
        name="comments"
        class="editor-comments"
        placeholder={data.internationalization?.["wiki-page-revision-comments"]}
        bind:value={$form.comments}></textarea>
      <div class="buttons alignleft">
        <input
          name="cancel"
          class="btn btn-danger"
          onclick={close}
          type="button"
          value={data.internationalization?.cancel}
        />
        <input
          name="save"
          class="btn btn-primary"
          type="submit"
          value={data.internationalization?.save}
        />
      </div>
    </form>
  {/if}
</div>

<style lang="scss">
  .editor {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
  }

  .editor-wikitext {
    height: 40vh;
  }
</style>
