<script lang="ts">
  import { deserialize } from "$app/forms"
  import { resolve } from "$app/paths"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { Layout } from "$lib/types"

  import type { PageBacklinkView } from "$lib/server/deepwell/page"
  import type { PageProps } from "./$types"

  type BacklinksState =
    | { status: "loading" }
    | { status: "failure" }
    | { status: "success"; items: PageBacklinkView[] }

  let {
    data,
    onPaneReadStateChange
  }: PageProps & {
    onPaneReadStateChange?: (state: "loading" | "success" | "failure") => void
  } = $props()
  const pageLayoutContext = getPageLayoutContext()
  let wikidot = $derived(pageLayoutContext.current === Layout.WIKIDOT)
  let state = $state<BacklinksState>({ status: "loading" })
  let requestId = 0
  let requestController: AbortController | undefined

  function reportReadState(readState: "loading" | "success" | "failure") {
    onPaneReadStateChange?.(readState)
  }

  async function fetchBacklinks() {
    requestController?.abort()
    const currentRequestId = ++requestId
    const controller = new AbortController()
    requestController = controller
    state = { status: "loading" }
    reportReadState("loading")

    try {
      const response = await fetch("?/backlinks", {
        method: "POST",
        body: "",
        signal: controller.signal
      })
      const responseBody = await response.text()
      if (!response.ok) throw new Error("Backlinks request failed.")

      const result = deserialize<
        { res: PageBacklinkView[] },
        { message: string; code: string; data: Record<string, unknown> }
      >(responseBody)

      if (currentRequestId !== requestId) return
      if (result.type !== "success" || !Array.isArray(result.data?.res)) {
        state = { status: "failure" }
        reportReadState("failure")
        return
      }

      state = { status: "success", items: result.data.res }
      reportReadState("success")
    } catch {
      if (currentRequestId === requestId && !controller.signal.aborted) {
        state = { status: "failure" }
        reportReadState("failure")
      }
    } finally {
      if (requestController === controller) requestController = undefined
    }
  }

  $effect(() => {
    const pageId = data.page?.page_id
    void pageId
    void fetchBacklinks()

    return () => {
      requestId += 1
      requestController?.abort()
      requestController = undefined
    }
  })
</script>

<!--
  Source Wikidot leaves `#action-area` hidden and empty when a read-only
  Backlinks request fails (issue #2241): no result list, no error dialog, no
  retry control. The parent hides the action area once this pane reports its
  read failed, so a failed Wikidot-layout read renders nothing at all. While
  the read is in flight the pane keeps its loading state (and the parent's
  close control), and the native layout keeps the visible error/retry
  affordance.
-->
{#if !wikidot || state.status !== "failure"}
  <h1 class="page-backlinks-header">
    {data.wikidot_page_actions?.backlinks ?? "Backlinks"}
  </h1>

  <div id="page-backlinks-list" aria-live="polite">
    {#if state.status === "loading"}
      <p role="status">Loading…</p>
    {:else if state.status === "failure"}
      <p role="alert">
        Could not load backlinks.
        <button
          aria-label="Retry loading backlinks"
          onclick={() => void fetchBacklinks()}
          type="button"
        >
          Retry
        </button>
      </p>
    {:else if state.items.length === 0}
      <p>No pages link to this page.</p>
    {:else}
      <ul>
        {#each state.items as backlink (backlink.slug)}
          <li><a href={resolve(`/${backlink.slug}`, {})}>{backlink.title}</a></li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}
