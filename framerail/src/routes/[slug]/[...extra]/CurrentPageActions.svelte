<script lang="ts">
  import { PagePane } from "$lib/types"

  import type { PageProps } from "./$types"

  let {
    data,
    activatePagePane,
    navigateEdit,
    showSource = $bindable()
  }: {
    data: PageProps["data"]
    activatePagePane: (pane: PagePane) => void
    navigateEdit: () => Promise<void>
    showSource: boolean
  } = $props()
</script>

<div class="action-row editor-actions">
  <button
    class="action-button editor-button button-move clickable"
    onclick={() => activatePagePane(PagePane.Move)}
    type="button"
  >
    {data.internationalization?.move}
  </button>
  <button
    class="action-button editor-button button-layout clickable"
    onclick={() => activatePagePane(PagePane.Layout)}
    type="button"
  >
    {data.internationalization?.layout}
  </button>
  <button
    class="action-button editor-button button-parents clickable"
    onclick={() => activatePagePane(PagePane.Parent)}
    type="button"
  >
    {data.internationalization?.parents}
  </button>
  <button
    class="action-button editor-button button-delete clickable"
    onclick={() => activatePagePane(PagePane.Delete)}
    type="button"
  >
    {data.internationalization?.delete}
  </button>
  <button
    class="action-button editor-button button-edit clickable"
    onclick={navigateEdit}
    type="button"
  >
    {data.internationalization?.edit}
  </button>
</div>
<div class="action-row other-actions">
  <button
    class="action-button button-source clickable"
    onclick={() => (showSource = true)}
    type="button"
  >
    {data.internationalization?.["wiki-page-view-source"]}
  </button>
  <button
    class="action-button button-history clickable"
    onclick={() => activatePagePane(PagePane.History)}
    type="button"
  >
    {data.internationalization?.history}
  </button>
  <button
    class="action-button button-vote clickable"
    onclick={() => activatePagePane(PagePane.Vote)}
    type="button"
  >
    {data.internationalization?.vote}
  </button>
  <button
    class="action-button button-files clickable"
    onclick={() => activatePagePane(PagePane.File)}
    type="button"
  >
    {data.internationalization?.files}
  </button>
</div>
