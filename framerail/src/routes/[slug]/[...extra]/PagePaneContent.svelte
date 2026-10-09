<script lang="ts">
  import { PagePane } from "$lib/types"
  import { wikidotPageSourceHtml } from "$lib/wikidot-history-contract.js"

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
  let AppendPane = $state<typeof import("./AppendPane.svelte").default>()
  let BacklinksPane = $state<typeof import("./BacklinksPane.svelte").default>()
  let DeletePane = $state<typeof import("./DeletePane.svelte").default>()
  let EditMetaPane = $state<typeof import("./EditMetaPane.svelte").default>()
  let FilePane = $state<typeof import("./FilePane.svelte").default>()
  let HistoryPane = $state<typeof import("./HistoryPane.svelte").default>()
  let LayoutPane = $state<typeof import("./LayoutPane.svelte").default>()
  let LockPane = $state<typeof import("./LockPane.svelte").default>()
  let MovePane = $state<typeof import("./MovePane.svelte").default>()
  let ParentPane = $state<typeof import("./ParentPane.svelte").default>()
  let SiteToolsPane = $state<typeof import("./SiteToolsPane.svelte").default>()
  let TagsPane = $state<typeof import("./TagsPane.svelte").default>()
  let VotePane = $state<typeof import("./VotePane.svelte").default>()
  let WatchersPane = $state<typeof import("./WatchersPane.svelte").default>()

  async function ensurePagePane(pane: PagePane) {
    switch (pane) {
      case PagePane.Append:
        AppendPane ??= (await import("./AppendPane.svelte")).default
        break
      case PagePane.Backlinks:
        BacklinksPane ??= (await import("./BacklinksPane.svelte")).default
        break
      case PagePane.Delete:
        DeletePane ??= (await import("./DeletePane.svelte")).default
        break
      case PagePane.File:
        FilePane ??= (await import("./FilePane.svelte")).default
        break
      case PagePane.EditMeta:
        EditMetaPane ??= (await import("./EditMetaPane.svelte")).default
        break
      case PagePane.History:
        HistoryPane ??= (await import("./HistoryPane.svelte")).default
        break
      case PagePane.Layout:
        LayoutPane ??= (await import("./LayoutPane.svelte")).default
        break
      case PagePane.Lock:
        LockPane ??= (await import("./LockPane.svelte")).default
        break
      case PagePane.Move:
        MovePane ??= (await import("./MovePane.svelte")).default
        break
      case PagePane.Parent:
        ParentPane ??= (await import("./ParentPane.svelte")).default
        break
      case PagePane.SiteTools:
        SiteToolsPane ??= (await import("./SiteToolsPane.svelte")).default
        break
      case PagePane.Tags:
        TagsPane ??= (await import("./TagsPane.svelte")).default
        break
      case PagePane.Vote:
        VotePane ??= (await import("./VotePane.svelte")).default
        break
      case PagePane.Watchers:
        WatchersPane ??= (await import("./WatchersPane.svelte")).default
        break
    }
  }

  $effect(() => {
    if (pagePaneState !== PagePane.None) void ensurePagePane(pagePaneState)
  })
</script>

{#snippet paneContent()}
  {#if pagePaneState === PagePane.Append}
    {#if AppendPane}
      <AppendPane bind:pagePaneState {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Backlinks}
    {#if BacklinksPane}
      <BacklinksPane {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.EditMeta}
    {#if EditMetaPane}
      <EditMetaPane {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Move}
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
  {:else if pagePaneState === PagePane.Tags}
    {#if TagsPane}
      <TagsPane bind:pagePaneState {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.SiteTools}
    {#if SiteToolsPane}
      <SiteToolsPane />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Lock}
    {#if LockPane}
      <LockPane bind:pagePaneState {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Vote}
    {#if VotePane}
      <VotePane {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else if pagePaneState === PagePane.Watchers}
    {#if WatchersPane}
      <WatchersPane {...props} />
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
      {#key `${data.site.site_id}:${data.page?.page_id ?? ""}`}
        <HistoryPane {setRevision} {setShowRevision} {...props} />
      {/key}
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
    <div class="page-source">{@html wikidotPageSourceHtml(data.wikitext ?? "")}</div>
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
