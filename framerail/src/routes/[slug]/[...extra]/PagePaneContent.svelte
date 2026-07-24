<script lang="ts">
  import { PagePane } from "$lib/types"

  import type { PageRevisionModelFiltered } from "$lib/server/deepwell/page"
  import type { Optional } from "$lib/types"
  import type { PageProps } from "./$types"

  let {
    props,
    pagePaneState = $bindable(),
    showSource,
    wikidot,
    setRevision,
    setShowRevision
  }: {
    props: PageProps
    pagePaneState: PagePane
    showSource: boolean
    wikidot: boolean
    setRevision: (revision: Optional<PageRevisionModelFiltered>) => void
    setShowRevision: (state: boolean) => void
  } = $props()

  let { data } = $derived(props)
  let DeletePane = $state<typeof import("./DeletePane.svelte").default>()
  let FilePane = $state<typeof import("./FilePane.svelte").default>()
  let HistoryPane = $state<typeof import("./HistoryPane.svelte").default>()
  let LayoutPane = $state<typeof import("./LayoutPane.svelte").default>()
  let MovePane = $state<typeof import("./MovePane.svelte").default>()
  let ParentPane = $state<typeof import("./ParentPane.svelte").default>()
  let VotePane = $state<typeof import("./VotePane.svelte").default>()

  async function ensurePagePane(pane: PagePane) {
    switch (pane) {
      case PagePane.Delete:
        DeletePane ??= (await import("./DeletePane.svelte")).default
        break
      case PagePane.File:
        FilePane ??= (await import("./FilePane.svelte")).default
        break
      case PagePane.History:
        HistoryPane ??= (await import("./HistoryPane.svelte")).default
        break
      case PagePane.Layout:
        LayoutPane ??= (await import("./LayoutPane.svelte")).default
        break
      case PagePane.Move:
        MovePane ??= (await import("./MovePane.svelte")).default
        break
      case PagePane.Parent:
        ParentPane ??= (await import("./ParentPane.svelte")).default
        break
      case PagePane.Vote:
        VotePane ??= (await import("./VotePane.svelte")).default
        break
    }
  }

  $effect(() => {
    if (pagePaneState !== PagePane.None) void ensurePagePane(pagePaneState)
  })
</script>

{#snippet paneContent()}
  {#if pagePaneState === PagePane.Move}
    {#if MovePane}
      <MovePane bind:pagePaneState {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Layout}
    {#if LayoutPane}
      <LayoutPane bind:pagePaneState {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Parent}
    {#if ParentPane}
      <ParentPane bind:pagePaneState {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Vote}
    {#if VotePane}
      <VotePane {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.File}
    {#if FilePane}
      <FilePane {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.History}
    {#if HistoryPane}
      <HistoryPane {setRevision} {setShowRevision} {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Delete}
    {#if DeletePane}
      <DeletePane bind:pagePaneState {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {/if}
{/snippet}

{#if wikidot}
  {#if showSource}
    <h1 class="page-source-header">
      {data.internationalization?.["wiki-page-source"]}
    </h1>
    <div class="page-source">{data.wikitext ?? ""}</div>
  {:else}
    {@render paneContent()}
  {/if}
{:else}
  {#if showSource}
    <h2 class="page-source-header">
      {data.internationalization?.["wiki-page-source"]}
    </h2>
    <textarea class="page-source" readonly={true}>{data.wikitext ?? ""}</textarea>
  {/if}
  {@render paneContent()}
{/if}
