<script lang="ts">
  import { PagePane } from "$lib/types"

  import type { PageProps } from "./$types"

  let { pagePaneState = $bindable(), data }: PageProps & { pagePaneState: PagePane } =
    $props()

  let appendedWikitext = $state("")
  const appendLabel = $derived.by(() => {
    const translated = data.internationalization?.["wiki-page-append-content"]
    return translated && translated !== "wiki-page-append-content"
      ? translated
      : "Wikitext to append"
  })
</script>

<h1 class="page-append-header">{data.wikidot_page_actions?.append ?? "Append"}</h1>

<form id="page-append" class="page-append-form" action="?/edit" method="POST">
  <input name="siteId" type="hidden" value={data.site.site_id} />
  <input name="pageId" type="hidden" value={data.page?.page_id} />
  <input name="lastRevisionId" type="hidden" value={data.page_revision?.revision_id} />
  <input name="title" type="hidden" value={data.page_revision?.title ?? ""} />
  <input name="altTitle" type="hidden" value={data.page_revision?.alt_title ?? ""} />
  <input name="tags" type="hidden" value={data.page_revision?.tags?.join(" ") ?? ""} />
  <input name="comments" type="hidden" value="" />
  <input name="wikitext" type="hidden" value={`${data.wikitext}${appendedWikitext}`} />
  <label class="page-append-label" for="page-append-input">
    {appendLabel}
  </label>
  <textarea id="page-append-input" name="append" bind:value={appendedWikitext}></textarea>
  <div class="buttons">
    <input
      class="btn btn-danger"
      onclick={() => (pagePaneState = PagePane.None)}
      type="button"
      value={data.internationalization?.cancel}
    />
    <input
      class="btn btn-primary"
      type="submit"
      value={data.internationalization?.save}
    />
  </div>
</form>

<style lang="scss">
  .page-append-form {
    padding-bottom: 2em;
  }

  #page-append-input {
    box-sizing: border-box;
    width: 100%;
    min-height: 12em;
  }

  .page-append-label {
    display: block;
    margin-bottom: 0.4em;
  }
</style>
