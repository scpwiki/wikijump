<script lang="ts">
  let {
    title,
    siteName,
    fontPreloadHrefs,
    compiledBodyStylesHead
  }: {
    title: string | null | undefined
    siteName: string
    fontPreloadHrefs: string[]
    compiledBodyStylesHead: string
  } = $props()
</script>

<svelte:head>
  <title>{title} | {siteName}</title>
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
