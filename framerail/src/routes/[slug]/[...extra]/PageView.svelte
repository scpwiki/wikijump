<script lang="ts">
  import { page } from "$app/state"
  import { goto } from "$app/navigation"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { Layout, PagePane } from "$lib/types"
  import { editPermissionDeniedMessage } from "$lib/wikidot/wikidot-locale"
  import { resolve } from "$app/paths"
  import {
    buildGeneratedPageStylesHead,
    getPageFontPreloadHrefs
  } from "$lib/generated-page-styles"
  import {
    buildWikidotDiscussButtonHtml,
    isWikidotFragmentPage,
    openWikidotPrinterFriendly
  } from "$lib/wikidot/wikidot-page-actions"
  import {
    updateWikidotRateWidget,
    wikidotLegacyActions
  } from "$lib/wikidot/wikidot-legacy-actions"
  import {
    requestLegacyRate,
    requestLegacyRateCancel,
    requestLegacySetTags
  } from "$lib/wikidot/wikidot-legacy-action-request"
  import { wikidotMembershipActions } from "$lib/wikidot/wikidot-membership-actions"
  import { wikidotGalleryLightbox } from "$lib/wikidot/wikidot-gallery-lightbox"
  import {
    requestMembershipApplication,
    requestMembershipEmailInvitation,
    requestMembershipJoin,
    requestMembershipPassword
  } from "$lib/wikidot/wikidot-membership-action-request"
  import { toggleWikidotEditSections } from "$lib/wikidot/wikidot-edit-sections"
  import { wikidotTabviews } from "$lib/wikidot/wikidot-tabviews"
  import { wikidotFoldableLists } from "$lib/wikidot/wikidot-foldable-lists.js"
  import { wikidotCollapsibles } from "$lib/wikidot/wikidot-collapsibles"
  import { wikidotToc } from "$lib/wikidot/wikidot-toc"
  import { wikidotFootnotes } from "$lib/wikidot/wikidot-footnotes"
  import { resolveWikidotHashMagicPagePane } from "$lib/wikidot/wikidot-hash-magic"
  import { onMount } from "svelte"

  import CurrentPageActions from "./CurrentPageActions.svelte"
  import CurrentPageMetadata from "./CurrentPageMetadata.svelte"
  import PageHead from "./PageHead.svelte"
  import PagePaneContent from "./PagePaneContent.svelte"
  import WikidotFoundPageTags from "./WikidotFoundPageTags.svelte"

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
  let EditSectionPane = $state<typeof import("./EditSectionPane.svelte").default>()
  let editSection = $state<{
    index: number
    level: number
    start: number
    end: number
  }>()
  let wikidotPageActions = $derived(data.wikidot_page_actions)
  let wikidotPageWatch = $derived(data.wikidot_page_watch)
  let dataFormEditing = $derived(!!data.options?.edit && !!data.data_form)
  let isDirectWikidotFragmentPage = $derived(
    pageLayoutContext.current === Layout.WIKIDOT &&
      !data.options?.debug &&
      !data.options?.no_render &&
      !showRevision &&
      isWikidotFragmentPage(data.page_revision?.tags)
  )
  let compiledBodyStyles = $derived(
    data.options?.debug || data.options?.no_render
      ? []
      : showRevision
        ? (revision?.compiled_body_styles ?? [])
        : (data.compiled_body_styles ?? [])
  )
  let compiledBodyStylesHead = $derived(
    pageLayoutContext.current === Layout.WIKIDOT
      ? ""
      : buildGeneratedPageStylesHead(compiledBodyStyles)
  )
  let renderedBodyHtml = $derived(
    showRevision ? revision?.compiled_body_html : data.compiled_body_html
  )
  let pageFontPreloadHrefs = $derived(
    pageLayoutContext.current === Layout.WIKIDOT
      ? []
      : getPageFontPreloadHrefs(data.site.locale, renderedBodyHtml, [
          data.page_revision?.title,
          ...compiledBodyStyles
        ])
  )

  function removeWikidotHtmlHydrationMarkers() {
    const pageContent = document.querySelector<HTMLElement>("#page-content")
    if (!pageContent) return

    // Svelte wraps a server-rendered {@html} value in empty boundary comments
    // for hydration. Keep those anchors through hydration, then remove only
    // the two boundary nodes so imported Wikidot content has no framework DOM.
    const firstChild = pageContent.firstChild
    if (firstChild?.nodeType === Node.COMMENT_NODE) {
      firstChild.remove()
    }

    const lastChild = pageContent.lastChild
    if (lastChild?.nodeType === Node.COMMENT_NODE) {
      lastChild.remove()
    }
  }

  /**
   * The single edit-permission boundary shared by the option-bar Edit
   * click and the `#_editpage` / `#_edittags` Hash Magic dispatch. It
   * calls the same server action and surfaces the same localized denial
   * dialog; it never grants, denies, or authorizes from hash text.
   */
  async function checkEditPermission(): Promise<boolean> {
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
      return false
    }
    if (result.type === "success" && result.data?.res && !result.data.res.can_edit) {
      errorPopupState.current = {
        state: true,
        message: editPermissionDeniedMessage(data.site.locale),
        data: null
      }
      return false
    }
    return result.type === "success" && result.data?.res?.can_edit === true
  }

  async function navigateEdit() {
    // Check edit permission first
    if (!(await checkEditPermission())) return

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

  async function openEditTagsPane() {
    // The Hash Magic tag command reuses the edit-permission boundary before
    // mounting the tag editor, so an unauthorized actor receives the same
    // denial dialog instead of an editable form.
    if (!(await checkEditPermission())) return
    activatePagePane(PagePane.Tags)
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

  async function ensureEditSectionPane() {
    EditSectionPane ??= (await import("./EditSectionPane.svelte")).default
  }

  function activatePagePane(pane: PagePane) {
    showSource = false
    editSection = undefined
    pagePaneState = pane
  }

  /**
   * Opens a page pane after the checks its write surface requires. Append
   * and Parent write under the page `edit` permission, so they are gated
   * before their editable form (or protected revision data) is mounted.
   * Move and Delete are not gated here: Deepwell exposes no move or delete
   * permission signal to the client, so they keep their existing behavior.
   * The server still rechecks every mutation at submission time.
   */
  async function openPagePane(pane: PagePane) {
    if (pane === PagePane.Append || pane === PagePane.Parent) {
      if (!(await checkEditPermission())) return
    }
    activatePagePane(pane)
  }

  function closeEditSection() {
    editSection = undefined
  }

  function toggleEditSections() {
    const pageContent = document.querySelector<HTMLElement>("#page-content")
    if (!pageContent || showRevision) return

    const visible = toggleWikidotEditSections(pageContent, data.wikitext, (section) => {
      showSource = false
      pagePaneState = PagePane.None
      editSection = section
      void ensureEditSectionPane()
    })
    if (!visible) closeEditSection()
  }

  const legacyRequestRuntime = { fetch, deserialize }

  function currentRateRegistry() {
    const registry = data.rate_actions
    return registry &&
      data.page &&
      data.page_revision &&
      registry.site_id === data.site.site_id &&
      registry.page_id === data.page.page_id &&
      registry.revision_id === data.page_revision.revision_id
      ? registry
      : null
  }

  async function setLegacyTags(actionIndex: number, actionFingerprint: string) {
    if (!data.page || !data.page_revision || showRevision) {
      throw new Error("This set-tags action is not available for the displayed revision.")
    }
    await requestLegacySetTags(legacyRequestRuntime, {
      pageId: data.page.page_id,
      lastRevisionId: data.page_revision.revision_id,
      actionIndex,
      actionFingerprint
    })
    window.location.reload()
  }

  async function rateFromLegacyWidget(
    actionIndex: number,
    actionFingerprint: string,
    _value: number,
    element: HTMLElement
  ) {
    const registry = currentRateRegistry()
    if (!registry) throw new Error("This page cannot be rated.")
    const score = await requestLegacyRate(legacyRequestRuntime, {
      pageId: registry.page_id,
      lastRevisionId: registry.revision_id,
      actionIndex,
      actionFingerprint
    })
    updateWikidotRateWidget(element, score?.score)
  }

  async function cancelLegacyRating(
    actionIndex: number,
    actionFingerprint: string,
    element: HTMLElement
  ) {
    const registry = currentRateRegistry()
    if (!registry) throw new Error("This page cannot be rated.")
    const score = await requestLegacyRateCancel(legacyRequestRuntime, {
      pageId: registry.page_id,
      lastRevisionId: registry.revision_id,
      actionIndex,
      actionFingerprint
    })
    updateWikidotRateWidget(element, score?.score)
  }

  const legacyActionRuntime = {
    edit: navigateEdit,
    history: () => activatePagePane(PagePane.History),
    source: () => {
      showSource = true
      pagePaneState = PagePane.None
    },
    print: () => openWikidotPrinterFriendly(page.url.pathname),
    setTags: setLegacyTags,
    rate: rateFromLegacyWidget,
    cancelRate: cancelLegacyRating,
    error: (error: unknown) => {
      errorPopupState.current = {
        state: true,
        message: error instanceof Error ? error.message : "Legacy page action failed.",
        data: null
      }
    }
  }

  function requireCurrentMembershipAction(
    pageId: number,
    revisionId: number,
    label: string
  ) {
    if (
      !data.page ||
      !data.page_revision ||
      showRevision ||
      data.page.page_id !== pageId ||
      data.page_revision.revision_id !== revisionId
    ) {
      throw new Error(`${label} is not available for the displayed revision.`)
    }
  }

  function joinFromLegacyControl(
    pageId: number,
    revisionId: number,
    actionIndex: number,
    actionFingerprint: string
  ) {
    requireCurrentMembershipAction(pageId, revisionId, "This Join action")
    return requestMembershipJoin(legacyRequestRuntime, {
      pageId,
      lastRevisionId: revisionId,
      actionIndex,
      actionFingerprint
    })
  }

  function applicationFromLegacyControl(
    pageId: number,
    revisionId: number,
    actionIndex: number,
    actionFingerprint: string,
    comment: string
  ) {
    requireCurrentMembershipAction(pageId, revisionId, "This membership application")
    return requestMembershipApplication(legacyRequestRuntime, {
      pageId,
      lastRevisionId: revisionId,
      actionIndex,
      actionFingerprint,
      comment
    })
  }

  function passwordFromLegacyControl(
    pageId: number,
    revisionId: number,
    actionIndex: number,
    actionFingerprint: string,
    password: string
  ) {
    requireCurrentMembershipAction(pageId, revisionId, "This membership password action")
    return requestMembershipPassword(legacyRequestRuntime, {
      pageId,
      lastRevisionId: revisionId,
      actionIndex,
      actionFingerprint,
      password
    })
  }

  function invitationFromLegacyControl(
    pageId: number,
    revisionId: number,
    actionIndex: number,
    actionFingerprint: string
  ) {
    requireCurrentMembershipAction(pageId, revisionId, "This membership invitation")
    return requestMembershipEmailInvitation(legacyRequestRuntime, {
      pageId,
      lastRevisionId: revisionId,
      actionIndex,
      actionFingerprint
    })
  }

  const membershipActionRuntime = {
    application: applicationFromLegacyControl,
    invitation: invitationFromLegacyControl,
    join: joinFromLegacyControl,
    password: passwordFromLegacyControl,
    reload: () => window.location.reload(),
    error: legacyActionRuntime.error
  }

  let legacyActionParameters = $derived({
    actions: showRevision ? [] : (data.legacy_actions ?? []),
    rateActions: showRevision ? [] : (currentRateRegistry()?.actions ?? []),
    runtime: legacyActionRuntime
  })
  let membershipActionParameters = $derived({
    actions: showRevision ? [] : (data.membership_actions ?? []),
    runtime: membershipActionRuntime
  })

  onMount(() => {
    // Hash Magic still applies under debug/norender documents (an edit hash on
    // a debug page is still an edit request); only the non-Wikidot layout and
    // the already-open /edit route skip the initial dispatch.
    if (pageLayoutContext.current !== Layout.WIKIDOT || data.options?.edit) return

    switch (resolveWikidotHashMagicPagePane(window.location.href)) {
      case "history":
        activatePagePane(PagePane.History)
        break
      case "files":
        activatePagePane(PagePane.File)
        break
      case "edit-page":
        void navigateEdit()
        break
      case "edit-tags":
        void openEditTagsPane()
        break
    }
  })

  $effect(() => {
    if (data.options?.edit) {
      void ensureEditorPane()
    }

    if (data.options?.history) {
      pagePaneState = PagePane.History
    }
  })

  $effect(() => {
    if (
      pageLayoutContext.current !== Layout.WIKIDOT ||
      data.options?.debug ||
      data.options?.no_render ||
      renderedBodyHtml === undefined
    ) {
      return
    }

    queueMicrotask(removeWikidotHtmlHydrationMarkers)
  })
</script>

<PageHead
  {compiledBodyStylesHead}
  fontPreloadHrefs={pageFontPreloadHrefs}
  metaTags={showRevision ? [] : (data.meta_tags ?? [])}
  siteName={data.site.name}
  title={data.page_revision?.title}
/>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  {#if data.options?.debug}
    <h2 class:hidden={dataFormEditing}>UNTRANSLATED:Debug Response</h2>
  {:else if showRevision}
    <div id="page-title" class:hidden={dataFormEditing}>{revision?.title}</div>
  {:else}
    <div id="page-title" class:hidden={dataFormEditing}>
      {data.page_revision?.title}
    </div>
  {/if}

  {#if data.options?.debug}
    <div id="page-content" class:hidden={dataFormEditing} use:wikidotTabviews>
      <textarea class="debug">{JSON.stringify(page, null, 2)}</textarea>
    </div>
  {:else if data.options?.no_render}
    <div id="page-content" class:hidden={dataFormEditing} use:wikidotTabviews>
      {data.internationalization?.["wiki-page-no-render"]}
      <textarea class="page-source" readonly={true}>{data.wikitext}</textarea>
    </div>
  {:else}
    <div
      id="page-content"
      class:hidden={dataFormEditing}
      use:wikidotGalleryLightbox={showRevision ? revision?.wikitext : data.wikitext}
      use:wikidotLegacyActions={legacyActionParameters}
      use:wikidotMembershipActions={membershipActionParameters}
      use:wikidotTabviews
      use:wikidotFoldableLists
      use:wikidotCollapsibles
      use:wikidotToc
      use:wikidotFootnotes
    >
      {@html showRevision ? revision?.compiled_body_html : data.compiled_body_html}
    </div>
  {/if}

  {#if showRevision}
    {#if revision?.tags?.length}
      <WikidotFoundPageTags hidden={dataFormEditing} tags={revision.tags} />
    {/if}
  {:else if data.page_revision?.tags?.length}
    <WikidotFoundPageTags hidden={dataFormEditing} tags={data.page_revision.tags} />
  {/if}

  {#if data.options?.edit}
    <div id="page-options-container" class:hidden={dataFormEditing}>
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
          <a
            id="tags-button"
            class="btn btn-default"
            href="javascript:;"
            onclick={() => activatePagePane(PagePane.Tags)}
            type="button"
          >
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
          onclick={(event) => {
            event.preventDefault()
            activatePagePane(PagePane.File)
          }}
          type="button"
        >
          {wikidotPageActions?.files ?? data.internationalization?.files}
        </a>
        {#if wikidotPageActions}
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a
            id="print-button"
            class="btn btn-default"
            href="javascript:;"
            onclick={() => openWikidotPrinterFriendly(page.url.pathname)}
            type="button"
          >
            {wikidotPageActions.print}
          </a>
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a
            id="site-tools-button"
            class="btn btn-default"
            href="javascript:;"
            onclick={() => activatePagePane(PagePane.SiteTools)}
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
          id="edit-append-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => openPagePane(PagePane.Append)}
          type="button"
        >
          {wikidotPageActions?.append ?? "Append"}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="edit-sections-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={toggleEditSections}
          type="button"
        >
          Edit Sections
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="edit-meta-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.EditMeta)}
          type="button"
        >
          Edit Meta
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="watchers-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.Watchers)}
          type="button"
        >
          Watchers
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="backlinks-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => activatePagePane(PagePane.Backlinks)}
          type="button"
        >
          {wikidotPageActions?.backlinks ?? "Backlinks"}
        </a>
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
          onclick={() => openPagePane(PagePane.Parent)}
          type="button"
        >
          {data.internationalization?.parents}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="lock-page-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.Lock
          }}
          type="button"
        >
          {data.internationalization?.["wiki-page-lock"]}
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

    <div
      id="action-area"
      class:hidden={!showSource && pagePaneState === PagePane.None && !editSection}
    >
      {#if showSource || pagePaneState !== PagePane.None || editSection}
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          class="action-area-close btn btn-danger"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.None
            closeEditSection()
          }}
          type="button"
        >
          {data.internationalization?.close}
        </a>
      {/if}

      {#if editSection}
        {#if EditSectionPane}
          {#key editSection.index}
            <EditSectionPane {...props} close={closeEditSection} section={editSection} />
          {/key}
        {:else}
          <p class="pane-loading" aria-live="polite">Loading…</p>
        {/if}
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
    <CurrentPageActions
      activatePagePane={openPagePane}
      {data}
      {navigateEdit}
      bind:showSource
    />
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
