<script lang="ts">
  let {
    title,
    siteName,
    fontPreloadHrefs,
    styleFrameStylesheets,
    compiledBodyStylesHead,
    wikidot
  }: {
    title: string | null | undefined
    siteName: string
    fontPreloadHrefs: string[]
    styleFrameStylesheets: { href: string; priority: string }[]
    compiledBodyStylesHead: string
    wikidot: boolean
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
  {#if wikidot}
    {#each styleFrameStylesheets as stylesheet, index (`${stylesheet.priority}:${stylesheet.href}:${index}`)}
      <link
        data-wikidot-style-preloaded
        data-wikidot-style-priority={stylesheet.priority}
        href={stylesheet.href}
        rel="stylesheet"
      />
    {/each}
  {/if}
  {@html compiledBodyStylesHead}
</svelte:head>
