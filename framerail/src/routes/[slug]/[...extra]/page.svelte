<script lang="ts">
  import { page } from "$app/state"
  import { goto } from "$app/navigation"
  import { getPageLayoutContext } from "$lib/page-layout-context"
  import { errorPopupState } from "$lib/stores.svelte"
  import { Layout, PagePane } from "$lib/types"
  import { resolve } from "$app/paths"
  import { buildWikidotPageTagsHtml } from "$lib/wikidot-page-tags"
  import {
    buildGeneratedPageStylesHead,
    getPageFontPreloadHrefs
  } from "$lib/generated-page-styles"
  import { isWikidotFragmentPage } from "$lib/wikidot-page-actions"
  import { wikidotTabviews } from "$lib/wikidot-tabviews"
  import { extractWikidotStyleFrameStylesheets } from "$lib/wikidot-styleframe"

  import type { PageProps } from "./$types"
  import type { Optional } from "$lib/types"
  import type { PageRevisionModelFiltered } from "$lib/server/deepwell/page"
  import { deserialize } from "$app/forms"

  let props: PageProps = $props()
  let { data } = $derived(props)
  const pageLayoutContext = getPageLayoutContext()

  let showSource = $state<boolean>(false)
  let showPageOptions = $state<boolean>(false)
  let showRevision = $state<boolean>(false)
  let revision = $state<Optional<PageRevisionModelFiltered>>(undefined)
  let pagePaneState = $state<PagePane>(PagePane.None)
  let DeletePane = $state<typeof import("./DeletePane.svelte").default>()
  let EditorPane = $state<typeof import("./EditorPane.svelte").default>()
  let FilePane = $state<typeof import("./FilePane.svelte").default>()
  let HistoryPane = $state<typeof import("./HistoryPane.svelte").default>()
  let LayoutPane = $state<typeof import("./LayoutPane.svelte").default>()
  let MovePane = $state<typeof import("./MovePane.svelte").default>()
  let ParentPane = $state<typeof import("./ParentPane.svelte").default>()
  let VotePane = $state<typeof import("./VotePane.svelte").default>()
  let wikidotPageActions = $derived(data.wikidot_page_actions)
  let wikidotPageWatch = $derived(data.wikidot_page_watch)
  let isDirectWikidotFragmentPage = $derived(
    pageLayoutContext.current === Layout.WIKIDOT &&
      !data.options?.debug &&
      !data.options?.no_render &&
      !showRevision &&
      isWikidotFragmentPage(data.page_revision?.tags)
  )
  const breadcrumbSeparator = " » "
  let compiledBodyStyles = $derived(
    data.options?.debug || data.options?.no_render
      ? []
      : showRevision
        ? (revision?.compiled_body_styles ?? [])
        : (data.compiled_body_styles ?? [])
  )
  let compiledBodyStylesHead = $derived(buildGeneratedPageStylesHead(compiledBodyStyles))
  let renderedBodyHtml = $derived(
    showRevision ? revision?.compiled_body_html : data.compiled_body_html
  )
  let bodyStyleFrameStylesheets = $derived(
    extractWikidotStyleFrameStylesheets([renderedBodyHtml], page.url.origin)
  )
  let pageFontPreloadHrefs = $derived(
    pageLayoutContext.current === Layout.WIKIDOT
      ? []
      : getPageFontPreloadHrefs(data.site.locale, renderedBodyHtml, [
          data.page_revision?.title,
          ...compiledBodyStyles
        ])
  )

  async function navigateEdit() {
    // Check edit permission first
    const res = await fetch("?/editPermission", {
      method: "POST",
      body: ""
    }).then((res) => res.text())

    const result = deserialize<
      { res: { can_edit: boolean } },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      if (!result.data.res.can_edit) {
        errorPopupState.current = {
          state: true,
          message: "UNTRANSLATED:You don't have permission to edit this page",
          data: null
        }
      } else {
        // Permission granted, navigate to edit page
        const options: string[] = Object.entries({
          norender: data.options.no_render,
          noredirect: data.options.no_redirect,
          debug: data.options.debug
        })
          .filter(([, enabled]) => enabled)
          .map(([key]) => `/${key}`)

        goto(resolve(`/${data.page!.slug}${options.join("")}/edit`, {}), {
          noScroll: true
        })
      }
    }
  }

  function setShowRevision(state: boolean) {
    showRevision = state
  }

  function toggleShowPageOptions(state?: boolean) {
    if (state !== undefined) showPageOptions = state
    else showPageOptions = !showPageOptions
  }

  function setRevision(rev: Optional<PageRevisionModelFiltered>) {
    revision = rev
  }

  async function ensureEditorPane() {
    EditorPane ??= (await import("./EditorPane.svelte")).default
  }

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

  function activatePagePane(pane: PagePane) {
    showSource = false
    pagePaneState = pane
    void ensurePagePane(pane)
  }

  $effect(() => {
    if (data.options?.edit) {
      void ensureEditorPane()
    }

    if (data.options?.history) {
      pagePaneState = PagePane.History
      void ensurePagePane(PagePane.History)
    }
  })
</script>

<svelte:head>
  <title>{data.page_revision?.title} | {data.site.name}</title>
  {#each pageFontPreloadHrefs as fontHref (fontHref)}
    <link
      as="font"
      crossorigin="anonymous"
      href={fontHref}
      rel="preload"
      type="font/woff2"
    />
  {/each}
  {#if pageLayoutContext.current === Layout.WIKIDOT}
    {#each bodyStyleFrameStylesheets as stylesheet, index (`${stylesheet.priority}:${stylesheet.href}:${index}`)}
      <link
        data-wikidot-style-preloaded
        data-wikidot-style-priority={stylesheet.priority}
        href={stylesheet.href}
        rel="stylesheet"
      />
    {/each}
  {/if}
  {@html compiledBodyStylesHead}
</svelte:head>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  {#if data.options?.debug}
    <h2>UNTRANSLATED:Debug Response</h2>
  {:else if showRevision}
    <div id="page-title">{revision?.title}</div>
  {:else}
    <div id="page-title">{data.page_revision?.title}</div>
  {/if}

  {#if !data.options?.debug && !showRevision && data.wikidot_breadcrumbs?.length}
    <div id="breadcrumbs">
      {#each data.wikidot_breadcrumbs as breadcrumb, index (breadcrumb.slug)}
        {#if index > 0}
          <span class="breadcrumb-separator">{breadcrumbSeparator}</span>
        {/if}
        <a href={resolve(`/${breadcrumb.slug}`, {})}>{breadcrumb.title}</a>
      {/each}
    </div>
  {/if}

  <div id="page-content" use:wikidotTabviews>
    {#if data.options?.debug}
      <textarea class="debug">{JSON.stringify(page, null, 2)}</textarea>
    {:else if data.options?.no_render}
      {data.internationalization?.["wiki-page-no-render"]}
      <textarea class="page-source" readonly={true}>{data.wikitext}</textarea>
    {:else if showRevision}
      {@html revision?.compiled_body_html}
    {:else}
      {@html data.compiled_body_html}
    {/if}
  </div>

  {#if showRevision}
    {#if revision?.tags?.length}
      <div class="page-tags">
        <span>{@html buildWikidotPageTagsHtml(revision.tags)}</span>
      </div>
    {/if}
  {:else if data.page_revision?.tags?.length}
    <div class="page-tags">
      <span>{@html buildWikidotPageTagsHtml(data.page_revision.tags)}</span>
    </div>
  {/if}

  {#if data.options?.edit}
    <div id="page-options-container">
      <div id="page-info">
        {#if data.wikidot_page_info}
          {data.wikidot_page_info}
        {:else}
          {data.internationalization?.["wiki-page-revision"]}, {data
            .internationalization?.["wiki-page-last-edit"]}
        {/if}
      </div>
    </div>
    <div id="action-area">
      {#if EditorPane}
        <EditorPane {...props} />
      {:else}
        <p class="pane-loading" aria-live="polite">Loading…</p>
      {/if}
    </div>
  {:else}
    <div id="page-options-container">
      <div id="page-info">
        {#if data.wikidot_page_info}
          {data.wikidot_page_info}
        {:else}
          {data.internationalization?.["wiki-page-revision"]}, {data
            .internationalization?.["wiki-page-last-edit"]}
        {/if}
      </div>
      {#if wikidotPageWatch}
        <div class="page-watch-options">
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a href="javascript:;">{wikidotPageWatch.label}</a>
          [<a href={wikidotPageWatch.helpHref} rel="noopener noreferrer" target="_blank"
            >{wikidotPageWatch.helpLabel}</a
          >]
        </div>
      {/if}
      <div
        id="page-options-bottom"
        class="page-options-bottom"
        class:hidden={!!data.options?.edit}
      >
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="edit-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={navigateEdit}
          type="button"
        >
          {wikidotPageActions?.edit ?? data.internationalization?.edit}
        </a>
        {#if !isDirectWikidotFragmentPage && wikidotPageActions?.showRate !== false}
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a
            id="pagerate-button"
            class="btn btn-default"
            href="javascript:;"
            onclick={() => activatePagePane(PagePane.Vote)}
            type="button"
          >
            {#if wikidotPageActions?.ratingText}
              {wikidotPageActions.ratePrefix} (<span>{wikidotPageActions.ratingText}</span
              >)
            {:else}
              {wikidotPageActions?.rate ?? data.internationalization?.vote}
            {/if}
          </a>
        {/if}
        {#if wikidotPageActions}
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a id="tags-button" class="btn btn-default" href="javascript:;" type="button">
            {wikidotPageActions.tags}
          </a>
          {#if wikidotPageActions.showDiscuss}
            <!-- svelte-ignore a11y_invalid_attribute -->
            <a
              id="discuss-button"
              class="btn btn-default"
              href="javascript:;"
              type="button"
            >
              {wikidotPageActions.discuss}
            </a>
          {/if}
        {/if}
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="history-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.History)}
          type="button"
        >
          {wikidotPageActions?.history ?? data.internationalization?.history}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="files-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.File)}
          type="button"
        >
          {wikidotPageActions?.files ?? data.internationalization?.files}
        </a>
        {#if wikidotPageActions}
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a id="print-button" class="btn btn-default" href="javascript:;" type="button">
            {wikidotPageActions.print}
          </a>
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a
            id="site-tools-button"
            class="btn btn-default"
            href="javascript:;"
            type="button"
          >
            {wikidotPageActions.siteTools}
          </a>
        {/if}

        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="more-options-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => toggleShowPageOptions()}
          type="button"
        >
          {(showPageOptions ? "- " : "+ ") +
            (wikidotPageActions?.options ?? data.internationalization?.options)}
        </a>
      </div>
    </div>

    {#if showPageOptions}
      <div id="page-options-bottom-2" class="page-options-bottom form-actions">
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="view-source-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => (showSource = true)}
          type="button"
        >
          {data.internationalization?.["wiki-page-view-source"]}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="layout-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.Layout)}
          type="button"
        >
          {data.internationalization?.layout}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="parent-page-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.Parent)}
          type="button"
        >
          {data.internationalization?.parents}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="rename-move-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.Move)}
          type="button"
        >
          {data.internationalization?.move}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="delete-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.Delete)}
          type="button"
        >
          {data.internationalization?.delete}
        </a>
      </div>
    {/if}

    <div id="action-area" class:hidden={!showSource && pagePaneState === PagePane.None}>
      {#if showSource || pagePaneState !== PagePane.None}
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          class="action-area-close btn btn-danger"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.None
          }}
          type="button"
        >
          {data.internationalization?.close}
        </a>
      {/if}

      {#if showSource}
        <h1 class="page-source-header">
          {data.internationalization?.["wiki-page-source"]}
        </h1>
        <div class="page-source">{data.wikitext ?? ""}</div>
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
    </div>
  {/if}
{:else}
  {#if data.options?.debug}
    <h2>UNTRANSLATED:Debug Response</h2>
  {:else if showRevision}
    <h2 class="page-title">{revision?.title}</h2>
  {:else}
    <h2 class="page-title">{data.page_revision?.title}</h2>
  {/if}

  <hr />

  <div class="page-content">
    {#if data.options?.debug}
      <textarea class="debug">{JSON.stringify(page, null, 2)}</textarea>
    {:else if data.options?.no_render}
      {data.internationalization?.["wiki-page-no-render"]}
      <textarea class="page-source" readonly={true}>{data.wikitext}</textarea>
    {:else if showRevision}
      {@html revision?.compiled_body_html}
    {:else}
      {@html data.compiled_body_html}
    {/if}
  </div>

  <div class="page-tags-container">
    {data.internationalization?.tags}
    <hr />
    <ul class="page-tags">
      {#if showRevision}
        {#each revision?.tags as tag (tag)}
          <li class="tag">{tag}</li>
        {/each}
      {:else}
        {#each data.page_revision?.tags as tag (tag)}
          <li class="tag">{tag}</li>
        {/each}
      {/if}
    </ul>
  </div>

  <div class="page-meta-info-container">
    <div class="page-meta-info info-revision">
      {data.internationalization?.["wiki-page-revision"]}
    </div>
    <div class="page-meta-info info-last-edit">
      {data.internationalization?.["wiki-page-last-edit"]}
    </div>
  </div>

  {#if data.options?.edit}
    {#if EditorPane}
      <EditorPane {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else}
    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-move clickable"
        onclick={() => activatePagePane(PagePane.Move)}
        type="button"
      >
        {data.internationalization?.move}
      </button>
      <button
        class="action-button editor-button button-layout clickable"
        onclick={() => activatePagePane(PagePane.Layout)}
        type="button"
      >
        {data.internationalization?.layout}
      </button>
      <button
        class="action-button editor-button button-parents clickable"
        onclick={() => activatePagePane(PagePane.Parent)}
        type="button"
      >
        {data.internationalization?.parents}
      </button>
      <button
        class="action-button editor-button button-delete clickable"
        onclick={() => activatePagePane(PagePane.Delete)}
        type="button"
      >
        {data.internationalization?.delete}
      </button>
      <button
        class="action-button editor-button button-edit clickable"
        onclick={navigateEdit}
        type="button"
      >
        {data.internationalization?.edit}
      </button>
    </div>
    <div class="action-row other-actions">
      <button
        class="action-button button-source clickable"
        onclick={() => (showSource = true)}
        type="button"
      >
        {data.internationalization?.["wiki-page-view-source"]}
      </button>
      <button
        class="action-button button-history clickable"
        onclick={() => activatePagePane(PagePane.History)}
        type="button"
      >
        {data.internationalization?.history}
      </button>
      <button
        class="action-button button-vote clickable"
        onclick={() => activatePagePane(PagePane.Vote)}
        type="button"
      >
        {data.internationalization?.vote}
      </button>
      <button
        class="action-button button-files clickable"
        onclick={() => activatePagePane(PagePane.File)}
        type="button"
      >
        {data.internationalization?.files}
      </button>
    </div>
  {/if}

  {#if showSource}
    <h2 class="page-source-header">
      {data.internationalization?.["wiki-page-source"]}
    </h2>
    <textarea class="page-source" readonly={true}>{data.wikitext ?? ""}</textarea>
  {/if}

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
{/if}

<style global lang="scss">
  .debug {
    width: 100%;
    height: 60vh;
  }

  .page-content,
  .page-tags-container,
  .page-meta-info-container,
  .editor-actions,
  .other-actions {
    padding: 0 0 2em;
  }

  .sigma-esque-container .page-tags {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    justify-content: flex-start;
    padding: 0;
    margin: 0;
    list-style: none;
  }

  .page-meta-info-container {
    text-align: right;
  }

  textarea.page-source {
    width: 100%;
    height: 60vh;
  }

  div.page-source {
    width: calc(100% - 4em - 2px);
    height: fit-content;
    padding: 1em 2em;
    white-space: pre-wrap;
  }

  .action-row {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: stretch;
    justify-content: flex-end;
    width: 100%;
  }
</style>
