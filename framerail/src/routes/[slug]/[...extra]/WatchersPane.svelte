<script lang="ts">
  import { deserialize } from "$app/forms"

  import WatchersList from "./WatchersList.svelte"

  import type { UserInfo } from "$lib/types"
  import type { PageProps } from "./$types"

  type WatchersState =
    | { status: "loading" }
    | { status: "failure" }
    | { status: "success"; items: UserInfo[] }

  let { data }: PageProps = $props()
  let state = $state<WatchersState>({ status: "loading" })
  let requestId = 0
  let requestController: AbortController | undefined

  async function getWatchers(pageId: number | undefined) {
    requestController?.abort()
    const currentRequestId = ++requestId
    const controller = new AbortController()
    requestController = controller
    state = { status: "loading" }

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
        return
      }

      state = { status: "success", items: result.data.res }
    } catch {
      if (currentRequestId === requestId && !controller.signal.aborted) {
        state = { status: "failure" }
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
