<script lang="ts">
  import { deserialize } from "$app/forms"
  import { resolve } from "$app/paths"

  import type { PageBacklinkView } from "$lib/server/deepwell/page"
  import type { PageProps } from "./$types"

  type BacklinksState =
    | { status: "loading" }
    | { status: "failure" }
    | { status: "success"; items: PageBacklinkView[] }

  let { data }: PageProps = $props()
  let state = $state<BacklinksState>({ status: "loading" })
  let requestId = 0
  let requestController: AbortController | undefined

  async function fetchBacklinks() {
    requestController?.abort()
    const currentRequestId = ++requestId
    const controller = new AbortController()
    requestController = controller
    state = { status: "loading" }

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
    void pageId
    void fetchBacklinks()

    return () => {
      requestId += 1
      requestController?.abort()
      requestController = undefined
    }
  })
</script>

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
