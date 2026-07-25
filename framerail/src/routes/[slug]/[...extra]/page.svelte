<script lang="ts">
  import { page } from "$app/state"
  import { goto } from "$app/navigation"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { Layout, PagePane } from "$lib/types"
  import { resolve } from "$app/paths"
  import { buildWikidotPageTagsHtml } from "$lib/wikidot/wikidot-page-tags"
  import {
    buildGeneratedPageStylesHead,
    getPageFontPreloadHrefs
  } from "$lib/generated-page-styles"
  import {
    buildWikidotDiscussButtonHtml,
    isWikidotFragmentPage
  } from "$lib/wikidot/wikidot-page-actions"
  import { wikidotTabviews } from "$lib/wikidot/wikidot-tabviews"
  import { extractWikidotStyleFrameStylesheets } from "$lib/wikidot/wikidot-styleframe"

  import CurrentPageActions from "./CurrentPageActions.svelte"
  import CurrentPageMetadata from "./CurrentPageMetadata.svelte"
  import PageHead from "./PageHead.svelte"
  import PagePaneContent from "./PagePaneContent.svelte"

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
  let EditorPane = $state<typeof import("./EditorPane.svelte").default>()
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

  function activatePagePane(pane: PagePane) {
    showSource = false
    pagePaneState = pane
  }

  $effect(() => {
    if (data.options?.edit) {
      void ensureEditorPane()
    }

    if (data.options?.history) {
      pagePaneState = PagePane.History
    }
  })
</script>

<PageHead
  {compiledBodyStylesHead}
  fontPreloadHrefs={pageFontPreloadHrefs}
  siteName={data.site.name}
  styleFrameStylesheets={bodyStyleFrameStylesheets}
  title={data.page_revision?.title}
  wikidot={pageLayoutContext.current === Layout.WIKIDOT}
/>

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
            {@html buildWikidotDiscussButtonHtml(wikidotPageActions.discuss)}
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

      <PagePaneContent
        {props}
        {setRevision}
        {setShowRevision}
        {showSource}
        wikidot
        bind:pagePaneState
      />
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

  <CurrentPageMetadata {data} {revision} {showRevision} />

  {#if data.options?.edit}
    {#if EditorPane}
      <EditorPane {...props} />
    {:else}
      <p class="pane-loading" aria-live="polite">Loading…</p>
    {/if}
  {:else}
    <CurrentPageActions {activatePagePane} {data} {navigateEdit} bind:showSource />
  {/if}

  <PagePaneContent
    {props}
    {setRevision}
    {setShowRevision}
    {showSource}
    wikidot={false}
    bind:pagePaneState
  />
{/if}

<style global lang="scss">
  @use "./page";
</style>
