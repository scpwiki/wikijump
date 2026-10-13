<script lang="ts">
  import "./responsive-tables.css"

  import { page } from "$app/state"
  import { goto } from "$app/navigation"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { Layout, PagePane } from "$lib/types"
  import {
    deletePermissionDeniedMessage,
    editPermissionDeniedDialog,
    editPermissionDeniedMessage,
    movePermissionDeniedMessage
  } from "$lib/wikidot/wikidot-locale"
  import { resolve } from "$app/paths"
  import {
    buildGeneratedPageStylesHead,
    getPageFontPreloadHrefs
  } from "$lib/generated-page-styles"
  import {
    isWikidotFragmentPage,
    openWikidotPrinterFriendly,
    requestWikidotPageDiscussion,
    wikidotPageDiscussionPath
  } from "$lib/wikidot/wikidot-page-actions"
  import {
    materializeWikidotRateActionHtml,
    updateWikidotRateWidget,
    wikidotLegacyActions
  } from "$lib/wikidot/wikidot-legacy-actions"
  import { wikidotSiteChanges } from "$lib/wikidot/wikidot-site-changes.js"
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
  import { wikidotEmailObfuscation } from "$lib/wikidot/wikidot-email-obfuscation.js"
  import { resolveWikidotHashMagicPagePane } from "$lib/wikidot/wikidot-hash-magic"
  import { onMount } from "svelte"

  import CurrentPageActions from "./CurrentPageActions.svelte"
  import CurrentPageMetadata from "./CurrentPageMetadata.svelte"
  import PageHead from "./PageHead.svelte"
  import PagePaneContent from "./PagePaneContent.svelte"
  import WikidotFoundPageTags from "./WikidotFoundPageTags.svelte"
  import WikidotPageInfoBreak from "./WikidotPageInfoBreak.svelte"

  import type { PageFile } from "$lib/server/deepwell/page-file"
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
  // Wikidot does not reveal its Files action area while the first file-list
  // request is pending or has failed (source SCP-9506, issue #2273).
  let wikidotFilesListReady = $state(false)
  let wikidotFilesPrefetched = $state<PageFile[] | null>(null)
  let wikidotFilesRequestId = 0
  // Wikidot leaves #action-area hidden and empty when a read-only Backlinks
  // or WhoWatches request fails (issue #2241): no result list, no error
  // dialog, no retry control. Each pane reports its read state; only a
  // reported failure hides the area. While the read is in flight the area
  // keeps its loading state and close control, preserving the abort-on-close
  // contract for hung reads.
  let wikidotBacklinksFailed = $state(false)
  let wikidotWatchersFailed = $state(false)

  function onAuxPaneReadStateChange(
    pane: "backlinks" | "watchers",
    readState: "loading" | "success" | "failure"
  ) {
    if (pane === "backlinks") wikidotBacklinksFailed = readState === "failure"
    else wikidotWatchersFailed = readState === "failure"
  }

  async function preloadWikidotFiles() {
    const requestId = ++wikidotFilesRequestId
    wikidotFilesListReady = false
    wikidotFilesPrefetched = null
    try {
      const response = await fetch("?/fileList", {
        method: "POST",
        body: JSON.stringify({
          siteId: data.site.site_id,
          pageId: data.page?.page_id,
          deleted: false
        })
      }).then((res) => res.text())
      if (requestId !== wikidotFilesRequestId || pagePaneState !== PagePane.File) return
      const result = deserialize<
        { res: PageFile[] },
        { message: string; code: string; data: Record<string, unknown> }
      >(response)
      if (result.type === "success" && Array.isArray(result.data?.res)) {
        wikidotFilesPrefetched = result.data.res
        wikidotFilesListReady = true
      } else {
        console.error("Wikidot Files list did not return an authoritative result")
      }
    } catch (error) {
      if (requestId === wikidotFilesRequestId) {
        console.error("Wikidot Files list request failed", error)
      }
    }
  }
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
  let actionAreaHidden = $derived(
    (!showSource && pagePaneState === PagePane.None && !editSection) ||
      (pagePaneState === PagePane.File && !wikidotFilesListReady) ||
      (pagePaneState === PagePane.Backlinks && wikidotBacklinksFailed) ||
      (pagePaneState === PagePane.Watchers && wikidotWatchersFailed)
  )
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
    showRevision
      ? revision?.compiled_body_html
      : materializeWikidotRateActionHtml(
          data.compiled_body_html,
          currentRateRegistry()?.actions ?? []
        )
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
   * The permission boundary shared by the option-bar write actions and the
   * `#_editpage` / `#_edittags` Hash Magic dispatch. Each action calls its
   * own server action (edit, move, or delete permission, matching the
   * authority the matching mutation enforces) and surfaces the matching
   * denial dialog before any writable form is mounted; it never grants,
   * denies, or authorizes from client state.
   */
  async function checkPagePermission(kind: "edit" | "move" | "delete"): Promise<boolean> {
    const routes = {
      edit: "?/editPermission",
      move: "?/movePermission",
      delete: "?/deletePermission"
    } as const
    const grantKeys = {
      edit: "can_edit",
      move: "can_move",
      delete: "can_delete"
    } as const
    const denialMessages = {
      edit: editPermissionDeniedMessage(data.site.locale),
      move: movePermissionDeniedMessage(),
      delete: deletePermissionDeniedMessage()
    } as const
    const res = await fetch(routes[kind], {
      method: "POST",
      body: ""
    }).then((res) => res.text())

    const result = deserialize<
      { res: Record<string, boolean> },
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
    const grantKey = grantKeys[kind]
    if (result.type === "success" && result.data?.res && !result.data.res[grantKey]) {
      // Anonymous EN edit denial renders the retained Wikidot Permission
      // error window (issue #2336). Other locales and actions keep the
      // localized generic denial; server failures keep their message above.
      const deniedDialog =
        kind === "edit" ? editPermissionDeniedDialog(data.site.locale) : null
      errorPopupState.current = deniedDialog
        ? { state: true, ...deniedDialog, data: null }
        : {
            state: true,
            message: denialMessages[kind],
            data: null
          }
      return false
    }
    return result.type === "success" && result.data?.res?.[grantKey] === true
  }

  async function checkEditPermission(): Promise<boolean> {
    return checkPagePermission("edit")
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
    if (pane === PagePane.File) {
      if (pageLayoutContext.current === Layout.WIKIDOT) {
        void preloadWikidotFiles()
      }
    }
    // A newly opened Backlinks/Watchers pane starts a fresh read, clearing
    // any failure inherited from its previous mount.
    if (pane === PagePane.Backlinks) wikidotBacklinksFailed = false
    if (pane === PagePane.Watchers) wikidotWatchersFailed = false
    pagePaneState = pane
  }

  /**
   * Opens a page pane after the checks its write surface requires. Append
   * and Parent write under the page `edit` permission; Move writes under
   * the `rename` permission and Delete under the `delete` permission, each
   * matching the authority the matching server mutation enforces. The gate
   * runs before the editable form (or protected revision data) is mounted.
   * The server still rechecks every mutation at submission time.
   */
  async function openPagePane(pane: PagePane) {
    if (pane === PagePane.Append || pane === PagePane.Parent) {
      if (!(await checkEditPermission())) return
    } else if (pane === PagePane.Move) {
      if (!(await checkPagePermission("move"))) return
    } else if (pane === PagePane.Delete) {
      if (!(await checkPagePermission("delete"))) return
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

  async function navigateWikidotDiscuss(event: MouseEvent) {
    event.preventDefault()
    const pageId = data.page?.page_id
    if (typeof pageId !== "number" || !Number.isSafeInteger(pageId) || pageId <= 0) {
      return
    }

    try {
      const response = await requestWikidotPageDiscussion(fetch, pageId)
      const discussionPath = wikidotPageDiscussionPath(response)
      if (discussionPath) {
        window.location.href = discussionPath
        return
      }
      errorPopupState.current = {
        state: true,
        title: data.internationalization?.error ?? "Error",
        leadIn: "Please note:",
        message: response.message ?? "Unable to create page discussion",
        data: null
      }
    } catch {
      errorPopupState.current = {
        state: true,
        title: data.internationalization?.error ?? "Error",
        leadIn: "Please note:",
        message: "Unable to create page discussion",
        data: null
      }
    }
  }

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
  let siteChangesActionParameters = $derived({ pageId: data.page.page_id })

  onMount(() => {
    // Hash Magic still applies under debug/norender documents (an edit hash on
    // a debug page is still an edit request); only the non-Wikidot layout and
    // the already-open /edit route skip the initial dispatch.
    if (pageLayoutContext.current !== Layout.WIKIDOT || data.options?.edit) {
      return
    }

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
      use:wikidotSiteChanges={siteChangesActionParameters}
      use:wikidotMembershipActions={membershipActionParameters}
      use:wikidotTabviews
      use:wikidotFoldableLists
      use:wikidotCollapsibles
      use:wikidotToc
      use:wikidotFootnotes
      use:wikidotEmailObfuscation
    >
      {@html renderedBodyHtml}
    </div>
  {/if}

  {#if showRevision}
    {#if revision?.tags?.length}
      <WikidotFoundPageTags hidden={dataFormEditing} tags={revision.tags} />
    {/if}
  {:else if data.page_revision?.tags?.length}
    <WikidotFoundPageTags hidden={dataFormEditing} tags={data.page_revision.tags} />
  {/if}

  <WikidotPageInfoBreak />

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
          type={wikidotPageActions ? undefined : "button"}
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
            type={wikidotPageActions ? undefined : "button"}
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
            onclick={() => void openEditTagsPane()}
            type={wikidotPageActions ? undefined : "button"}
          >
            {wikidotPageActions.tags}
          </a>
          {#if wikidotPageActions.showDiscuss && wikidotPageActions.discussHref}
            <a
              href={wikidotPageActions.discussHref}
              class="btn btn-default"
              id="discuss-button"
            >
              {wikidotPageActions.discuss}
            </a>
          {:else if wikidotPageActions.showDiscuss}
            <!-- svelte-ignore a11y_invalid_attribute -->
            <a
              id="discuss-button"
              class="btn btn-default"
              href="javascript:;"
              onclick={navigateWikidotDiscuss}
              type={wikidotPageActions ? undefined : "button"}
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
          type={wikidotPageActions ? undefined : "button"}
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
          type={wikidotPageActions ? undefined : "button"}
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
            type={wikidotPageActions ? undefined : "button"}
          >
            {wikidotPageActions.print}
          </a>
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a
            id="site-tools-button"
            class="btn btn-default"
            href="javascript:;"
            onclick={() => activatePagePane(PagePane.SiteTools)}
            type={wikidotPageActions ? undefined : "button"}
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
          type={wikidotPageActions ? undefined : "button"}
        >
          {(wikidotPageActions
            ? showPageOptions
              ? "-\u00a0"
              : "+\u00a0"
            : showPageOptions
              ? "- "
              : "+ ") +
            (wikidotPageActions?.options ?? data.internationalization?.options)}
        </a>
      </div>
    </div>

    <div
      id="page-options-bottom-2"
      class="page-options-bottom form-actions"
      style:display={showPageOptions ? null : "none"}
    >
      {#snippet appendControl()}
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="edit-append-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => openPagePane(PagePane.Append)}
          type={wikidotPageActions ? undefined : "button"}
        >
          {wikidotPageActions?.append ??
            data.internationalization?.["wiki-page-action-append"]}
        </a>
      {/snippet}
      {#snippet sectionsControl()}
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="edit-sections-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={toggleEditSections}
          type={wikidotPageActions ? undefined : "button"}
        >
          {wikidotPageActions?.editSections ??
            data.internationalization?.["wiki-page-action-edit-sections"]}
        </a>
      {/snippet}
      {#if wikidotPageActions}
        {@render sectionsControl()}
        {@render appendControl()}
      {:else}
        {@render appendControl()}
        {@render sectionsControl()}
      {/if}
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        id="edit-meta-button"
        class="btn btn-default"
        href="javascript:;"
        onclick={() => activatePagePane(PagePane.EditMeta)}
        type={wikidotPageActions ? undefined : "button"}
      >
        {wikidotPageActions?.editMeta ??
          data.internationalization?.["wiki-page-action-edit-meta"]}
      </a>
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        id="watchers-button"
        class="btn btn-default"
        href="javascript:;"
        onclick={() => activatePagePane(PagePane.Watchers)}
        type={wikidotPageActions ? undefined : "button"}
      >
        {wikidotPageActions?.watchers ??
          data.internationalization?.["wiki-page-action-watchers"]}
      </a>
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        id="backlinks-button"
        class="btn btn-default"
        href="javascript:;"
        onclick={() => activatePagePane(PagePane.Backlinks)}
        type={wikidotPageActions ? undefined : "button"}
      >
        {wikidotPageActions?.backlinks ??
          data.internationalization?.["wiki-page-action-backlinks"]}
      </a>
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        id="view-source-button"
        class="btn btn-default"
        href="javascript:;"
        onclick={() => (showSource = true)}
        type={wikidotPageActions ? undefined : "button"}
      >
        {wikidotPageActions?.viewSource ??
          data.internationalization?.["wiki-page-view-source"]}
      </a>
      {#if !wikidotPageActions}
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
      {/if}
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        id="parent-page-button"
        class="btn btn-default"
        href="javascript:;"
        onclick={() => openPagePane(PagePane.Parent)}
        type={wikidotPageActions ? undefined : "button"}
      >
        {wikidotPageActions?.parent ?? data.internationalization?.parents}
      </a>
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        id={wikidotPageActions ? "page-block-button" : "lock-page-button"}
        class="btn btn-default"
        href="javascript:;"
        onclick={() => {
          showSource = false
          pagePaneState = PagePane.Lock
        }}
        type={wikidotPageActions ? undefined : "button"}
      >
        {wikidotPageActions?.lock ?? data.internationalization?.["wiki-page-action-lock"]}
      </a>
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        id="rename-move-button"
        class="btn btn-default"
        href="javascript:;"
        onclick={() => openPagePane(PagePane.Move)}
        type={wikidotPageActions ? undefined : "button"}
      >
        {wikidotPageActions?.rename ??
          data.internationalization?.["wiki-page-action-rename-move"]}
      </a>
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        id="delete-button"
        class="btn btn-default"
        href="javascript:;"
        onclick={() => openPagePane(PagePane.Delete)}
        type={wikidotPageActions ? undefined : "button"}
      >
        {wikidotPageActions?.delete ?? data.internationalization?.delete}
      </a>
    </div>

    <!-- Wikidot hides this inline so theme #action-area rules cannot expose an empty pane. -->
    <div
      id="action-area"
      class:hidden={actionAreaHidden}
      style:display={actionAreaHidden ? "none" : undefined}
    >
      {#if (showSource || pagePaneState !== PagePane.None || editSection) && (pagePaneState !== PagePane.File || wikidotFilesListReady) && (pagePaneState !== PagePane.Backlinks || !wikidotBacklinksFailed) && (pagePaneState !== PagePane.Watchers || !wikidotWatchersFailed)}
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

      {#if pagePaneState !== PagePane.File || wikidotFilesListReady}
        <PagePaneContent
          {props}
          {setRevision}
          {setShowRevision}
          {showSource}
          wikidot
          initialFileList={wikidotFilesPrefetched}
          onFileListVisibilityChange={(ready) => (wikidotFilesListReady = ready)}
          onPaneReadStateChange={onAuxPaneReadStateChange}
          bind:pagePaneState
        />
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
