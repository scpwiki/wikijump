<script lang="ts">
  import SigmaEsque from "$lib/sigma-esque/sigma-esque.svelte"
  import Wikidot from "$lib/sigma-esque/wikidot.svelte"
  import wjBanner from "$assets/logo-outline.min.svg?raw"
  import ui from "$assets/ui.svg?raw"
  import ErrorPopup from "$lib/popup/error.svelte"
  import Toasts from "$lib/component/Toasts.svelte"

  import { page } from "$app/state"
  import { pageLayoutState, errorPopupState } from "$lib/stores.svelte"
  import { Layout } from "$lib/types"
  import { resolve } from "$app/paths"

  let { children } = $props()

  function closeErrorPopup() {
    errorPopupState.current = {
      state: false,
      message: null,
      data: null
    }
  }

  function setLayout() {
    if (page.route.id?.startsWith("/[x+2d]/")) {
      // this is a special page, use Wikijump layout
      pageLayoutState.current = Layout.WIKIJUMP
    } else {
      pageLayoutState.current =
        page.data?.page?.layout ?? page.data?.site?.layout ?? Layout.WIKIJUMP
    }
  }
  $effect(() => {
    setLayout()
  })
</script>

<div class="svg-defs hidden">
  {@html ui}
</div>

{#if errorPopupState.current.state}
  <ErrorPopup exitPrompt={closeErrorPopup} />
{/if}

<div id="toasts">
  <Toasts />
</div>

<svelte:head>
  <title>{page.data.site?.name}</title>
</svelte:head>

{#if pageLayoutState.current === Layout.WIKIDOT}
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

    {#snippet sideBar()}
      {@html page.data?.compiled_side_bar_html ??
        page.error?.compiled_side_bar_html ??
        ""}
    {/snippet}

    {#snippet content()}
      {@render children?.()}
    {/snippet}

    {#snippet footer()}
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
    {/snippet}
    {#snippet license()}
      {@html page.data?.internationalization?.["footer-license-unless"] ??
        page.error?.internationalization?.["footer-license-unless"]}
    {/snippet}
  </Wikidot>
{:else}
  <SigmaEsque>
    {#snippet header()}
      <div class="header-wjbanner">
        {@html wjBanner}
      </div>
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

<!-- Ignoring the "unused" svg as we know we imported and embedded a raw svg -->
<!-- svelte-ignore css_unused_selector -->
<style global lang="scss">
  @use "../lib/css/abstracts/variables" as *;

  $tablet-max-width: 767px;

  .header-wjbanner {
    height: 80%;
    color: #fff;

    svg {
      width: auto;
      height: 100%;
    }
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

  #toasts,
  #modals {
    position: fixed;
    bottom: 0;
    left: 0;
    z-index: $z-dialog;
    width: 100%;
  }

  @media (max-width: $tablet-max-width) {
    .header-wjbanner {
      text-align: center;

      svg {
        height: initial;
        max-height: 6.5em;
      }
    }
  }
</style>
