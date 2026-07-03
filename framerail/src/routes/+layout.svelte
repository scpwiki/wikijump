<script lang="ts">
  import SigmaEsque from "$lib/sigma-esque/sigma-esque.svelte"
  import Wikidot from "$lib/sigma-esque/wikidot.svelte"
  import ErrorPopup from "$lib/popup/error.svelte"

  import { page } from "$app/state"
  import { setContext } from "svelte"
  import { pageLayoutState, errorPopupState } from "$lib/stores.svelte"
  import { Layout } from "$lib/types"
  import {
    WIKIDOT_FOOTER_LINKS,
    WIKIDOT_POWERED_BY,
    buildWikidotLicenseHtml,
    isImportedWikidotView
  } from "$lib/wikidot-footer"
  import {
    PAGE_LAYOUT_CONTEXT_KEY,
    type PageLayoutContext
  } from "$lib/page-layout-context"
  import { resolveShellLayout } from "$lib/wikidot-shell"
  import { resolve } from "$app/paths"

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
  const isImportedWikidotLayout = $derived(isImportedWikidotView(page.data ?? page.error))
  const wikidotLicenseHtml = $derived(
    buildWikidotLicenseHtml({
      licenseName: page.data?.license_name ?? page.error?.license_name,
      licenseUrl: page.data?.license_url ?? page.error?.license_url
    })
  )
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
</svelte:head>

{#if currentLayout === Layout.WIKIDOT}
  <style global>
    /* Use Sigma 10 as default Wikidot theme for now */
    @import url("https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--theme/base/css/style.css");
    @import url("https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/css/pagerate/PageRateWidgetModule.css");
    @import url("https://cdn.scpwiki.com/theme/en/sigma/css/sigma.min.css");
  </style>
  <Wikidot>
    {#snippet header()}
      <h1>
        <a class="active" href={resolve("/", {})}><span>{page.data.site?.name}</span></a>
      </h1>
      <h2>
        <span>{page.data.site?.tagline}</span>
      </h2>
    {/snippet}

    {#snippet topBar()}
      {@html page.data?.compiled_top_bar_html ?? page.error?.compiled_top_bar_html ?? ""}
    {/snippet}

    {#snippet loginStatus()}
      {#if !(page.data?.user_session ?? page.error?.user_session)}
        <div id="login-status">
          <a class="login-status-create-account btn" href={resolve("/-/register", {})}
            >Create account</a
          >
          <span>or</span>
          <a class="login-status-sign-in btn btn-primary" href={resolve("/-/login", {})}
            >Sign in</a
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
          {#each WIKIDOT_FOOTER_LINKS as link, index (link.label)}
            <a href={resolve(link.href, {})}>{link.label}</a
            >{#if index < WIKIDOT_FOOTER_LINKS.length - 1}{" | "}{/if}
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
