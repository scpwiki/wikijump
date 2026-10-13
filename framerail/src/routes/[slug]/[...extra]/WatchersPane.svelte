<script lang="ts">
  import { deserialize } from "$app/forms"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { Layout } from "$lib/types"

  import WatchersList from "./WatchersList.svelte"

  import type { UserInfo } from "$lib/types"
  import type { PageProps } from "./$types"

  type WatchersState =
    | { status: "loading" }
    | { status: "failure" }
    | { status: "success"; items: UserInfo[] }

  let {
    data,
    onPaneReadStateChange
  }: PageProps & {
    onPaneReadStateChange?: (state: "loading" | "success" | "failure") => void
  } = $props()
  const pageLayoutContext = getPageLayoutContext()
  let wikidot = $derived(pageLayoutContext.current === Layout.WIKIDOT)
  let state = $state<WatchersState>({ status: "loading" })
  let requestId = 0
  let requestController: AbortController | undefined

  function reportReadState(readState: "loading" | "success" | "failure") {
    onPaneReadStateChange?.(readState)
  }

  async function getWatchers(pageId: number | undefined) {
    requestController?.abort()
    const currentRequestId = ++requestId
    const controller = new AbortController()
    requestController = controller
    state = { status: "loading" }
    reportReadState("loading")

    try {
      const response = await fetch("?/watchers", {
        method: "POST",
        body: JSON.stringify({ pageId }),
        signal: controller.signal
      })
      const responseBody = await response.text()
      if (!response.ok) throw new Error("Watchers request failed.")

      const result = deserialize<
        { res: UserInfo[] },
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
    void getWatchers(pageId)

    return () => {
      requestId += 1
      requestController?.abort()
      requestController = undefined
    }
  })
</script>

<!--
  Source Wikidot leaves `#action-area` hidden and empty when a read-only
  WhoWatches request fails (issue #2241): no roster, no error dialog, no
  retry control, and never a fabricated empty list. The parent hides the
  action area once this pane reports its read failed, so a failed
  Wikidot-layout read renders nothing at all. While the read is in flight
  the pane keeps its loading state (and the parent's close control), and
  the native layout keeps the visible error/retry affordance.
-->
{#if !wikidot || state.status !== "failure"}
  <h1 class="page-watchers-header">Watchers</h1>
  {#if state.status === "loading"}
    <p role="status">Loading…</p>
  {:else if state.status === "failure"}
    <p role="alert">
      Could not load watchers.
      <button
        aria-label="Retry loading watchers"
        onclick={() => void getWatchers(data.page?.page_id)}
        type="button"
      >
        Retry
      </button>
    </p>
  {:else}
    <WatchersList watchers={state.items} />
  {/if}
{/if}
