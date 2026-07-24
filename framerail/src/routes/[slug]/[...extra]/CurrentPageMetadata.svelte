<script lang="ts">
  import type { PageRevisionModelFiltered } from "$lib/server/deepwell/page"
  import type { Optional } from "$lib/types"
  import type { PageProps } from "./$types"

  let {
    data,
    revision,
    showRevision
  }: {
    data: PageProps["data"]
    revision: Optional<PageRevisionModelFiltered>
    showRevision: boolean
  } = $props()
</script>

<div class="page-tags-container">
  {data.internationalization?.tags}
  <hr />
  <ul class="page-tags">
    {#if showRevision}
      {#each revision?.tags as tag (tag)}
        <li class="tag">{tag}</li>
      {/each}
    {:else}
      {#each data.page_revision?.tags as tag (tag)}
        <li class="tag">{tag}</li>
      {/each}
    {/if}
  </ul>
</div>

<div class="page-meta-info-container">
  <div class="page-meta-info info-revision">
    {data.internationalization?.["wiki-page-revision"]}
  </div>
  <div class="page-meta-info info-last-edit">
    {data.internationalization?.["wiki-page-last-edit"]}
  </div>
</div>
