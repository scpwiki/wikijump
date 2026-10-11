<script lang="ts">
  import SigmaEsque from "$lib/sigma-esque/sigma-esque.svelte"
  import Wikidot from "$lib/sigma-esque/wikidot.svelte"
  import ErrorPopup from "$lib/popup/error.svelte"

  import { page } from "$app/state"
  import { onMount, setContext } from "svelte"
  import { pageLayoutState, errorPopupState } from "$lib/layout/stores.svelte"
  import { Layout } from "$lib/types"
  import {
    WIKIDOT_POWERED_BY,
    buildWikidotAccountLabels,
    buildWikidotFooterLinks,
    buildWikidotLicenseHtml,
    buildWikidotLoginLabels,
    isImportedWikidotView,
    shouldUseWikidotLicenseHtml
  } from "$lib/wikidot/wikidot-footer"
  import {
    PAGE_LAYOUT_CONTEXT_KEY,
    type PageLayoutContext
  } from "$lib/layout/page-layout-context"
  import { resolveShellLayout } from "$lib/layout/wikidot-shell"
  import {
    resolveCanonicalViewData,
    resolveCanonicalViewMetadata
  } from "$lib/view-data-decision.js"
  import { resolve } from "$app/paths"
  import {
    resolveWikidotSessionUserName,
    resolveWikidotSiteTagline,
    resolveWikidotSiteTitle,
    shouldUseSandboxWikidotChrome
  } from "$lib/wikidot/wikidot-chrome"
  import { buildGeneratedPageStylesHead } from "$lib/generated-page-styles"
  import {
    buildWikidotInlineStyleFrameHead,
    extractWikidotStyleFrameDeclarations
  } from "$lib/wikidot/wikidot-styleframe"
  import {
    IOS_ICON_DECLARATIONS,
    IOS_ICON_ROUTE_PREFIX,
    faviconDeclaration,
    hasIosIcons,
    windowsTileDeclaration
  } from "$lib/site-icons"
  import { installWikidotCategories } from "$lib/wikidot/wikidot-categories.js"
  import { installJavascriptAnchorGuard } from "$lib/wikidot/wikidot-javascript-anchors.js"
  import { installWikidotNewPageHelper } from "$lib/wikidot/wikidot-new-page-helper"
  import WikidotBottomToolbar from "$lib/wikidot/WikidotBottomToolbar.svelte"
  import WikidotLoginDialog from "$lib/wikidot/WikidotLoginDialog.svelte"
  import { isPlainPrimaryActivation } from "$lib/wikidot/wikidot-login-dialog.js"
  import {
    installWikidotSearchAll,
    submitWikidotTopSearch
  } from "$lib/wikidot/wikidot-search.js"
  import {
    builtInThemeHeadHtml,
    customThemeHeadHtml,
    normalizeGoogleAnalyticsSettings,
    normalizeThemeSetting,
    resolveThemePreviewUrl
  } from "$lib/site-settings.js"

  const WIKIDOT_SEARCH_CHROME_HTML =
    '<div id="search-top-box" class="form-search"><form id="search-top-box-form" class="input-append" action="dummy"><input id="search-top-box-input" name="query" class="text empty search-query" size="15" type="text" value="Search this site"/><input name="search" class="button btn" type="submit" value="Search" /></form></div>'

  let { children } = $props()

  // Imported Wikidot layouts open Sign in in place; the href stays the native
  // /-/login fallback for modified clicks and for activation before hydration.
  let wikidotSignInOpen = $state(false)
  function openWikidotSignIn(event: MouseEvent) {
    if (!isPlainPrimaryActivation(event)) return
    event.preventDefault()
    wikidotSignInOpen = true
  }

  function closeErrorPopup() {
    errorPopupState.current = {
      state: false,
      message: null,
      data: null
    }
  }

  function clearWikidotSearchPrompt(event: FocusEvent) {
    const input = event.currentTarget as HTMLInputElement
    if (input.classList.contains("empty")) {
      input.classList.remove("empty")
      input.value = ""
    }
  }

  function submitWikidotSearch(event: SubmitEvent) {
    submitWikidotTopSearch(event, window)
  }

  function resolveCurrentLayout() {
    if (page.route.id?.startsWith("/[x+2d]/")) {
      // this is a special page, use Wikijump layout
      return Layout.WIKIJUMP
    }

    return resolveShellLayout(resolveCanonicalViewData(page.error, page.data))
  }

  const currentLayout = $derived.by(resolveCurrentLayout)
  const isPrinterFriendlyView = $derived(page.route.id === "/printer--friendly/[...path]")
  const canonicalView = $derived(resolveCanonicalViewMetadata(page.error, page.data))
  const viewData = $derived(canonicalView.viewData)
  const wikidotLocale = $derived(canonicalView.locale)
  const wikidotFooterLinks = $derived(buildWikidotFooterLinks(wikidotLocale))
  const wikidotLoginLabels = $derived(buildWikidotLoginLabels(wikidotLocale))
  const wikidotAccountLabels = $derived(buildWikidotAccountLabels(wikidotLocale))
  const wikidotLicenseHtml = $derived(
    buildWikidotLicenseHtml({
      licenseName: canonicalView.licenseName,
      licenseUrl: canonicalView.licenseUrl,
      licenseKind: canonicalView.licenseKind,
      licenseHtml: canonicalView.licenseHtml,
      locale: wikidotLocale,
      sourceSite: canonicalView.sourceSite
    })
  )
  const isImportedWikidotLayout = $derived(isImportedWikidotView(viewData))
  const useWikidotLicenseHtml = $derived(
    shouldUseWikidotLicenseHtml(isImportedWikidotLayout, canonicalView.licenseKind)
  )
  const useSandboxWikidotChrome = $derived(shouldUseSandboxWikidotChrome(viewData))
  const showTopToolbar = $derived(
    currentLayout === Layout.WIKIDOT && viewData?.site_settings?.toolbars?.top === true
  )
  const showBottomToolbar = $derived(
    currentLayout === Layout.WIKIDOT && viewData?.site_settings?.toolbars?.bottom === true
  )
  const promotedSites = $derived(viewData?.promoted_sites ?? [])
  const analyticsSettings = $derived(
    currentLayout === Layout.WIKIDOT
      ? normalizeGoogleAnalyticsSettings(viewData?.site_settings?.google_analytics)
      : normalizeGoogleAnalyticsSettings(undefined)
  )
  const analyticsProfile = $derived(
    analyticsSettings.enabled ? analyticsSettings.profile : null
  )
  const effectiveTheme = $derived(normalizeThemeSetting(viewData?.theme))
  const themePreviewUrl = $derived(
    resolveThemePreviewUrl(
      viewData?.theme_previewer_no_ui,
      page.url.searchParams.get("theme_url")
    )
  )
  const customThemeHtml = $derived(customThemeHeadHtml(effectiveTheme))
  const builtInThemeHtml = $derived(builtInThemeHeadHtml(effectiveTheme))
  const wikidotSiteTitle = $derived(resolveWikidotSiteTitle(viewData))
  const iconSite = $derived(
    viewData?.site
      ? {
          ...viewData.site,
          from_wikidot: viewData.site.from_wikidot || isImportedWikidotLayout
        }
      : null
  )
  const siteFavicon = $derived(faviconDeclaration(iconSite))
  const siteHasIosIcons = $derived(hasIosIcons(viewData?.site ?? null))
  const siteWindowsTile = $derived(windowsTileDeclaration(iconSite))
  const wikidotSiteTagline = $derived(resolveWikidotSiteTagline(viewData))
  const wikidotSessionUserName = $derived(resolveWikidotSessionUserName(viewData))
  const styleFrameDeclarations = $derived(
    extractWikidotStyleFrameDeclarations(
      [
        viewData?.compiled_top_bar_html,
        viewData?.compiled_side_bar_html,
        viewData?.compiled_body_html
      ],
      page.url.origin
    )
  )
  const pageLayoutContext = $state<PageLayoutContext>({
    current: resolveCurrentLayout()
  })

  setContext(PAGE_LAYOUT_CONTEXT_KEY, pageLayoutContext)

  onMount(() => {
    let disposed = false
    let stop: (() => void) | undefined
    const uninstallSearchAll = installWikidotSearchAll(window)
    const uninstallJavascriptAnchorGuard = installJavascriptAnchorGuard(document)
    installWikidotCategories(window)
    installWikidotNewPageHelper(window)
    const wikidotSearchForm =
      document.querySelector<HTMLFormElement>("#search-top-box-form")
    const wikidotSearchInput = document.querySelector<HTMLInputElement>(
      "#search-top-box-input"
    )
    wikidotSearchForm?.addEventListener("submit", submitWikidotSearch)
    wikidotSearchInput?.addEventListener("focus", clearWikidotSearchPrompt)
    void import("$lib/wikidot/wikidot-code-highlighting").then((module) => {
      if (!disposed) stop = module.observeWikidotCodeBlocks(document)
    })
    return () => {
      disposed = true
      stop?.()
      uninstallSearchAll()
      uninstallJavascriptAnchorGuard()
      wikidotSearchForm?.removeEventListener("submit", submitWikidotSearch)
      wikidotSearchInput?.removeEventListener("focus", clearWikidotSearchPrompt)
    }
  })

  // Keep existing child components synchronized after hydration while the
  // top-level shell decision is available during SSR through request-local context.
  $effect.pre(() => {
    pageLayoutContext.current = currentLayout
    pageLayoutState.current = currentLayout
  })
</script>

{#if errorPopupState.current.state}
  <ErrorPopup exitPrompt={closeErrorPopup} />
{/if}

<svelte:head>
  <title>{viewData?.site?.name}</title>
  {#if analyticsProfile}
    <meta
      name="wikidot-site-analytics-profile"
      content={analyticsProfile}
      data-wikidot-site-analytics-valid="true"
    />
  {/if}
  {#if siteFavicon}
    <link href={siteFavicon.href} rel="shortcut icon" />
    <link href={siteFavicon.href} rel="icon" type={siteFavicon.type} />
  {:else}
    <link href="data:," rel="icon" />
  {/if}
  {#if siteHasIosIcons}
    {#each IOS_ICON_DECLARATIONS as iosIcon (iosIcon.filename)}
      <link
        href={`${IOS_ICON_ROUTE_PREFIX}${iosIcon.filename}`}
        rel="apple-touch-icon"
        sizes={iosIcon.sizes ?? undefined}
      />
    {/each}
  {/if}
  {#if siteWindowsTile}
    <meta name="msapplication-TileImage" content={siteWindowsTile.href} />
  {/if}
  {#if !isPrinterFriendlyView && currentLayout === Layout.WIKIDOT}
    <link href="/wikidot/styles/wikidot-base-165bc434fd1d.css" rel="stylesheet" />
    <link href="/wikidot/styles/pagerate-db0bffe086ed.css" rel="stylesheet" />
    {#if !viewData?.theme_previewer_blank}
      <link href="/wikidot/styles/sigma-fe5388a32e12.css" rel="stylesheet" />
      {#if effectiveTheme.type === "external"}
        <link data-wikidot-site-theme href={effectiveTheme.url} rel="stylesheet" />
      {:else if effectiveTheme.type === "custom"}
        {@html customThemeHtml}
      {:else}
        {@html builtInThemeHtml}
      {/if}
    {/if}
    {#if themePreviewUrl}
      <link data-wikidot-theme-preview href={themePreviewUrl} rel="stylesheet" />
    {/if}
    {#each styleFrameDeclarations as declaration, index (`${declaration.priority}:${declaration.kind}:${declaration.order}:${index}`)}
      {#if declaration.kind === "theme"}
        <link
          data-wikidot-style-preloaded
          data-wikidot-style-priority={declaration.priority}
          href={declaration.href}
          rel="stylesheet"
        />
      {:else}
        {@html buildWikidotInlineStyleFrameHead(declaration)}
      {/if}
    {/each}
    {@html buildGeneratedPageStylesHead(viewData?.compiled_body_styles ?? [])}
  {/if}
</svelte:head>

{#if isPrinterFriendlyView}
  {@render children?.()}
{:else if currentLayout === Layout.WIKIDOT}
  {#if showTopToolbar}
    <div id="navi-bar">
      <a href="http://www.wikidot.com"><span>Wikidot.com</span></a>
      <div class="new-site">
        <form action="http://www.wikidot.com/new-site" method="get">
          <input
            name="address"
            class="text empty"
            type="text"
            value="site-name"
          />.wikidot.com
        </form>
      </div>
      <div class="action-buttons">
        <span>Edit</span>
        <span>History</span>
        <span>Tags</span>
        <span>Source</span>
      </div>
      <a class="random-site" href={resolve("/random-site.php", {})}>Explore »</a>
    </div>
    <div id="navi-bar-shadow">&nbsp;</div>
  {/if}
  <Wikidot>
    {#snippet header()}
      <h1>
        <a class="active" href={resolve("/", {})}><span>{wikidotSiteTitle}</span></a>
      </h1>
      {#if wikidotSiteTagline}
        <h2>
          <span>{wikidotSiteTagline}</span>
        </h2>
      {/if}
      {@html WIKIDOT_SEARCH_CHROME_HTML}
      {#if useSandboxWikidotChrome && wikidotSessionUserName}
        <div class="login-status">
          <div class="btn-group logged-in">
            <button
              style:opacity={1}
              class="btn disabled user-karma-level-5"
              type="button"
            >
              <span class="printuser">{wikidotSessionUserName}</span>
            </button>
          </div>
        </div>
      {/if}
    {/snippet}

    {#snippet topBar()}
      {#if useSandboxWikidotChrome}
        <a class="navbar-brand" href={resolve("/", {})}>Home</a>
      {/if}
      {@html viewData?.compiled_top_bar_html ?? ""}
    {/snippet}

    {#snippet loginStatus()}
      {#if !useSandboxWikidotChrome && wikidotSessionUserName}
        <div id="login-status">
          <a id="my-account" href={resolve("/-/user", {})}>{wikidotSessionUserName}</a>
          <span class="printuser">{wikidotSessionUserName}</span>
          <!-- svelte-ignore a11y_invalid_attribute -- Wikidot-compatible legacy account toggle -->
          <a id="account-topbutton" href="javascript:;">&#9660;</a>
          <div id="account-options">
            <ul>
              <li>
                <a href={resolve("/-/user", {})}>{wikidotAccountLabels.myAccount}</a>
              </li>
              <li>
                <a href={resolve("/-/settings", {})}>{wikidotAccountLabels.settings}</a>
              </li>
              <li>
                <a href={resolve("/-/logout", {})}>{wikidotAccountLabels.signOut}</a>
              </li>
            </ul>
          </div>
        </div>
      {:else if !useSandboxWikidotChrome && !viewData?.user_session}
        <div id="login-status">
          <a class="login-status-create-account btn" href={resolve("/-/register", {})}
            >{wikidotLoginLabels.createAccount}</a
          >
          <span>{wikidotLoginLabels.or}</span>
          <a
            class="login-status-sign-in btn btn-primary"
            href={resolve("/-/login", {})}
            onclick={isImportedWikidotLayout ? openWikidotSignIn : undefined}
            >{wikidotLoginLabels.signIn}</a
          >
          {#if isImportedWikidotLayout}
            <WikidotLoginDialog
              open={wikidotSignInOpen}
              onclose={() => (wikidotSignInOpen = false)}
            />
          {/if}
        </div>
      {/if}
    {/snippet}

    {#snippet sideBar()}
      {@html viewData?.compiled_side_bar_html ?? ""}
    {/snippet}

    {#snippet content()}
      {@render children?.()}
    {/snippet}

    {#snippet footer()}
      {#if isImportedWikidotLayout}
        <div class="options">
          {#each wikidotFooterLinks as link, index (link.label)}
            <a href={resolve(link.href, {})}>{link.label}</a
            >{#if index < wikidotFooterLinks.length - 1}
              |
            {/if}
          {/each}
        </div>
        <div class="footer-powered-by">{WIKIDOT_POWERED_BY}</div>
      {:else}
        <div class="options">
          <a href={resolve("/", {})}>{viewData?.internationalization?.docs}</a>
          |
          <a href={resolve("/", {})}
            >{viewData?.internationalization?.["terms-conditions"]}</a
          >
          |
          <a href={resolve("/", {})}>{viewData?.internationalization?.privacy}</a>
          |
          <a href={resolve("/", {})}>{viewData?.internationalization?.security}</a>
        </div>
        <div class="footer-powered-by">
          {viewData?.internationalization?.["footer-powered-by"]}
        </div>
      {/if}
    {/snippet}
    {#snippet license()}
      {#if useWikidotLicenseHtml}
        {@html wikidotLicenseHtml}
      {:else}
        {@html viewData?.internationalization?.["footer-license-unless"] ?? ""}
      {/if}
    {/snippet}
    {#snippet bottomToolbar()}
      {#if showBottomToolbar}
        <WikidotBottomToolbar sites={promotedSites} />
      {/if}
    {/snippet}
  </Wikidot>
{:else}
  <SigmaEsque>
    {#snippet header()}
      <h1 class="header-wordmark">Wikijump</h1>
    {/snippet}

    {#snippet topBar()}
      {@html viewData?.compiled_top_bar_html ?? ""}
    {/snippet}

    {#snippet content()}
      {@render children?.()}
    {/snippet}

    {#snippet footer()}
      <div class="footer-inner">
        <ul class="footer-items">
          <li class="footer-item">
            <a href={resolve("/", {})}
              >{viewData?.internationalization?.["terms-conditions"]}</a
            >
          </li>
          <li class="footer-item">
            <a href={resolve("/", {})}>{viewData?.internationalization?.privacy}</a>
          </li>
          <li class="footer-item">
            <a href={resolve("/", {})}>{viewData?.internationalization?.docs}</a>
          </li>
          <li class="footer-item">
            <a href={resolve("/", {})}>{viewData?.internationalization?.security}</a>
          </li>
        </ul>
        <div class="footer-powered-by">
          {viewData?.internationalization?.["footer-powered-by"]}
        </div>
      </div>
    {/snippet}
  </SigmaEsque>
{/if}

<style global lang="scss">
  @use "../lib/css/abstracts/variables" as *;

  .odate[class*="time_"]:not(:empty) {
    display: inline !important;
  }

  $tablet-max-width: 767px;

  .header-wordmark {
    margin: 0;
    font-family: var(--font-display);
    font-size: 3rem;
    font-weight: 700;
    line-height: 1;
    color: #fff;
    letter-spacing: 0;
  }

  .footer-inner {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    justify-content: stretch;
    width: 100%;
    max-width: 100%;
  }

  // Wrap the links instead of pushing the powered-by line past narrow viewports.
  .footer-items {
    display: flex;
    flex: 1 1 auto;
    flex-direction: row;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    justify-content: flex-start;
    min-width: 0;
    padding: 0;
    list-style: none;

    .footer-item a {
      color: #fff;
      text-decoration: none;
    }
  }

  @media (max-width: $tablet-max-width) {
    .header-wordmark {
      font-size: 2rem;
      text-align: center;
    }
  }
</style>
