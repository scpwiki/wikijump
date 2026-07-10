<script lang="ts">
  import SigmaEsque from "$lib/sigma-esque/sigma-esque.svelte"
  import Wikidot from "$lib/sigma-esque/wikidot.svelte"
  import ErrorPopup from "$lib/popup/error.svelte"

  import { page } from "$app/state"
  import { setContext } from "svelte"
  import { pageLayoutState, errorPopupState } from "$lib/stores.svelte"
  import { Layout } from "$lib/types"
  import {
    WIKIDOT_POWERED_BY,
    buildWikidotFooterLinks,
    buildWikidotLicenseHtml,
    buildWikidotLoginLabels,
    isImportedWikidotView
  } from "$lib/wikidot-footer"
  import {
    PAGE_LAYOUT_CONTEXT_KEY,
    type PageLayoutContext
  } from "$lib/page-layout-context"
  import { resolveShellLayout } from "$lib/wikidot-shell"
  import { resolve } from "$app/paths"
  import {
    resolveWikidotSessionUserName,
    resolveWikidotSiteTagline,
    resolveWikidotSiteTitle,
    shouldUseSandboxWikidotChrome
  } from "$lib/wikidot-chrome"

  let { children } = $props()

  function closeErrorPopup() {
    errorPopupState.current = {
      state: false,
      message: null,
      data: null
    }
  }

  function resolveCurrentLayout() {
    if (page.route.id?.startsWith("/[x+2d]/")) {
      // this is a special page, use Wikijump layout
      return Layout.WIKIJUMP
    }

    return resolveShellLayout(page.error ?? page.data)
  }

  const currentLayout = $derived.by(resolveCurrentLayout)
  const wikidotLocale = $derived(page.data?.site?.locale ?? page.error?.site?.locale)
  const wikidotFooterLinks = $derived(buildWikidotFooterLinks(wikidotLocale))
  const wikidotLoginLabels = $derived(buildWikidotLoginLabels(wikidotLocale))
  const wikidotLicenseHtml = $derived(
    buildWikidotLicenseHtml({
      licenseName: page.data?.license_name ?? page.error?.license_name,
      licenseUrl: page.data?.license_url ?? page.error?.license_url,
      locale: wikidotLocale,
      sourceSite:
        page.data?.wikidot_snapshot?.source_site ??
        page.error?.wikidot_snapshot?.source_site
    })
  )
  const viewData = $derived(page.error ?? page.data)
  const isImportedWikidotLayout = $derived(isImportedWikidotView(viewData))
  const useSandboxWikidotChrome = $derived(shouldUseSandboxWikidotChrome(viewData))
  const wikidotSiteTitle = $derived(resolveWikidotSiteTitle(viewData))
  const wikidotSiteTagline = $derived(resolveWikidotSiteTagline(viewData))
  const wikidotSessionUserName = $derived(resolveWikidotSessionUserName(viewData))
  const pageLayoutContext = $state<PageLayoutContext>({
    current: resolveCurrentLayout()
  })

  setContext(PAGE_LAYOUT_CONTEXT_KEY, pageLayoutContext)

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
  <title>{page.data.site?.name}</title>
  {#if currentLayout === Layout.WIKIDOT}
    <link
      href="https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--theme/base/css/style.css"
      rel="stylesheet"
    />
    <link
      href="https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/css/pagerate/PageRateWidgetModule.css"
      rel="stylesheet"
    />
    <link
      href="https://cdn.scpwiki.com/theme/en/sigma/css/sigma.min.css"
      rel="stylesheet"
    />
  {/if}
</svelte:head>

{#if currentLayout === Layout.WIKIDOT}
  {#if useSandboxWikidotChrome}
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
      {@html page.data?.compiled_top_bar_html ?? page.error?.compiled_top_bar_html ?? ""}
    {/snippet}

    {#snippet loginStatus()}
      {#if isImportedWikidotLayout && !useSandboxWikidotChrome && wikidotSessionUserName}
        <div id="login-status">
          <span class="printuser">{wikidotSessionUserName}</span>
          <span> | </span>
          <span>My account</span>
          <span> ▼</span>
        </div>
      {:else if !useSandboxWikidotChrome && !(page.data?.user_session ?? page.error?.user_session)}
        <div id="login-status">
          <a class="login-status-create-account btn" href={resolve("/-/register", {})}
            >{wikidotLoginLabels.createAccount}</a
          >
          <span>{wikidotLoginLabels.or}</span>
          <a class="login-status-sign-in btn btn-primary" href={resolve("/-/login", {})}
            >{wikidotLoginLabels.signIn}</a
          >
        </div>
      {/if}
    {/snippet}

    {#snippet sideBar()}
      {@html page.data?.compiled_side_bar_html ??
        page.error?.compiled_side_bar_html ??
        ""}
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
          <a href={resolve("/", {})}
            >{page.data?.internationalization?.docs ??
              page.error?.internationalization?.docs}</a
          >
          |
          <a href={resolve("/", {})}
            >{page.data?.internationalization?.["terms-conditions"] ??
              page.error?.internationalization?.["terms-conditions"]}</a
          >
          |
          <a href={resolve("/", {})}
            >{page.data?.internationalization?.privacy ??
              page.error?.internationalization?.privacy}</a
          >
          |
          <a href={resolve("/", {})}
            >{page.data?.internationalization?.security ??
              page.error?.internationalization?.security}</a
          >
        </div>
        <div class="footer-powered-by">
          {page.data?.internationalization?.["footer-powered-by"] ??
            page.error?.internationalization?.["footer-powered-by"]}
        </div>
      {/if}
    {/snippet}
    {#snippet license()}
      {#if isImportedWikidotLayout}
        {@html wikidotLicenseHtml}
      {:else}
        {@html page.data?.internationalization?.["footer-license-unless"] ??
          page.error?.internationalization?.["footer-license-unless"]}
      {/if}
    {/snippet}
  </Wikidot>
{:else}
  <SigmaEsque>
    {#snippet header()}
      <h1 class="header-wordmark">Wikijump</h1>
    {/snippet}

    {#snippet topBar()}
      {@html page.data?.compiled_top_bar_html ?? page.error?.compiled_top_bar_html ?? ""}
    {/snippet}

    {#snippet content()}
      {@render children?.()}
    {/snippet}

    {#snippet footer()}
      <div class="footer-inner">
        <ul class="footer-items">
          <li class="footer-item">
            <a href={resolve("/", {})}
              >{page.data?.internationalization?.["terms-conditions"] ??
                page.error?.internationalization?.["terms-conditions"]}</a
            >
          </li>
          <li class="footer-item">
            <a href={resolve("/", {})}
              >{page.data?.internationalization?.privacy ??
                page.error?.internationalization?.privacy}</a
            >
          </li>
          <li class="footer-item">
            <a href={resolve("/", {})}
              >{page.data?.internationalization?.docs ??
                page.error?.internationalization?.docs}</a
            >
          </li>
          <li class="footer-item">
            <a href={resolve("/", {})}
              >{page.data?.internationalization?.security ??
                page.error?.internationalization?.security}</a
            >
          </li>
        </ul>
        <div class="footer-powered-by">
          {page.data?.internationalization?.["footer-powered-by"] ??
            page.error?.internationalization?.["footer-powered-by"]}
        </div>
      </div>
    {/snippet}
  </SigmaEsque>
{/if}

<style global lang="scss">
  @use "../lib/css/abstracts/variables" as *;

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
    gap: 10px;
    align-items: center;
    justify-content: stretch;
    width: 100%;
  }

  .footer-items {
    display: flex;
    flex: 1;
    flex-direction: row;
    gap: 10px;
    align-items: center;
    justify-content: flex-start;
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
