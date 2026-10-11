<script lang="ts">
  import { page } from "$app/state"
  import { buildGeneratedPageStylesHead } from "$lib/generated-page-styles"
  import {
    buildWikidotPrintOptionsHtml,
    buildWikidotPrintSourceInfoHtml,
    wikidotPrintView
  } from "$lib/wikidot/wikidot-print-view"

  let { data } = $props()

  const pageTitle = $derived(data.page_revision?.title ?? "")
  const sourcePath = $derived(`/${page.params.path ?? ""}`)
  const sourceUrl = $derived(`${page.url.origin}${sourcePath}`)
</script>

<svelte:head>
  <title>{data.site.name}: {pageTitle}</title>
  {@html buildGeneratedPageStylesHead(data.compiled_body_styles ?? [])}
</svelte:head>

<div id="container" use:wikidotPrintView>
  {@html buildWikidotPrintOptionsHtml()}
  <hr />
  {@html buildWikidotPrintSourceInfoHtml({
    siteName: data.site.name,
    siteUrl: page.url.origin,
    pageTitle,
    pageUrl: sourceUrl
  })}
  <h1>{pageTitle}</h1>
  <div id="print-content">
    {@html data.compiled_body_html}
  </div>
</div>

<style>
  :global(body.print-body) {
    margin: 0;
    color: #000;
    background: #fff;
  }

  #container {
    padding: 0.5rem;
    font-family: monospace;
  }

  :global(#print-options table) {
    border-collapse: collapse;
  }

  :global(#print-options td) {
    padding: 0 0.5rem 0 0;
    vertical-align: top;
  }

  :global(#print-options a) {
    color: #00e;
  }

  :global(#print-source-info) {
    margin-bottom: 0.75rem;
  }

  :global(#print-source-info[hidden]) {
    display: none !important;
  }

  @media print {
    /* The on-screen option controls and the separator generated for them
       must not reach printed or PDF output. Source attribution stays. */
    :global(body.print-body #container #print-options),
    :global(body.print-body #container > hr) {
      display: none;
    }
  }
</style>
