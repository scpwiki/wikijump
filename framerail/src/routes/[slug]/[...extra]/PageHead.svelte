<script lang="ts">
  let {
    fontPreloadHrefs,
    compiledBodyStylesHead,
    metaTags
  }: {
    fontPreloadHrefs: string[]
    compiledBodyStylesHead: string
    metaTags: { name: string; content: string; all_pages: boolean }[]
  } = $props()
</script>

<svelte:head>
  {#each metaTags as metaTag (`${metaTag.all_pages}:${metaTag.name}`)}
    <meta name={metaTag.name} content={metaTag.content} />
  {/each}
  {#each fontPreloadHrefs as fontHref (fontHref)}
    <link
      as="font"
      crossorigin="anonymous"
      href={fontHref}
      rel="preload"
      type="font/woff2"
    />
  {/each}
  {@html compiledBodyStylesHead}
</svelte:head>
