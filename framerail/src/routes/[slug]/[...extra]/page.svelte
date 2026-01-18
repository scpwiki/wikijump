<script lang="ts">
  import { page } from "$app/stores"
  import { goto } from "$app/navigation"
  import { onMount } from "svelte"
  import { usePageLayoutState, usePagePaneState } from "$lib/stores"
  import { Layout, PagePane } from "$lib/types"
  import {
    EditorPane,
    FilePane,
    HistoryPane,
    LayoutPane,
    MovePane,
    ParentPane,
    VotePane,
    DeletePane
  } from "."
  let pagePaneState = usePagePaneState()
  let pageLayout = usePageLayoutState()

  let showSource = false
  let showPageOptions = false
  let showRevision = false
  let revision: Record<string, any> = {}

  function navigateEdit() {
    let options: string[] = []
    if ($page.data.options.no_render) options.push("norender")
    if ($page.data.options.no_redirect) options.push("noredirect")
    if ($page.data.options.debug) options.push("debug")
    options = options.map((opt) => `/${opt}`)
    goto(`/${$page.data.page.slug}${options.join("")}/edit`, {
      noScroll: true
    })
  }

  function setShowRevision(state: boolean) {
    showRevision = state
  }

  function toggleShowPageOptions(state?: boolean) {
    if (state !== undefined) showPageOptions = state
    else showPageOptions = !showPageOptions
  }

  function setRevision(rev: Record<string, any>) {
    revision = rev
  }

  onMount(() => {
    if ($page.data?.options.history) pagePaneState.set(PagePane.History)
  })
</script>

{#if $pageLayout === Layout.WIKIDOT}
  {#if $page.data.options?.debug}
    <h2>UNTRANSLATED:Debug Response</h2>
  {:else if showRevision}
    <div id="page-title">{revision.title}</div>
  {:else}
    <div id="page-title">{$page.data.page_revision.title}</div>
  {/if}

  <div id="page-content">
    {#if $page.data.options?.debug}
      <textarea class="debug">{JSON.stringify($page, null, 2)}</textarea>
    {:else if $page.data.options?.no_render}
      {$page.data.internationalization["wiki-page-no-render"]}
      <textarea class="page-source" readonly={true}>{$page.data.wikitext}</textarea>
    {:else if showRevision}
      {@html revision.compiled_body_html}
    {:else}
      {@html $page.data.compiled_body_html}
    {/if}
  </div>

  {#if showRevision}
    {#if revision.tags.length}
      <div class="page-tags">
        <span
          >{#each revision.tags as tag}
            <a href={`/system:page-tags/tag/${tag}`}>{tag}</a>
          {/each}</span
        >
      </div>
    {/if}
  {:else if $page.data.page_revision.tags.length}
    <div class="page-tags">
      <span
        >{#each $page.data.page_revision.tags as tag}
          <a href={`/system:page-tags/tag/${tag}`}>{tag}</a>
        {/each}</span
      >
    </div>
  {/if}

  {#if $page.data.options?.edit}
    <div id="page-options-container">
      <div id="page-info">
        {$page.data.internationalization["wiki-page-revision"]}, {$page.data
          .internationalization["wiki-page-last-edit"]}
      </div>
    </div>
    <div id="action-area">
      <EditorPane />
    </div>
  {:else}
    <div id="page-options-container">
      <div id="page-info">
        {$page.data.internationalization["wiki-page-revision"]}, {$page.data
          .internationalization["wiki-page-last-edit"]}
      </div>
      <div
        id="page-options-bottom"
        class="page-options-bottom"
        class:hidden={!!$page.data.options?.edit}
      >
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="edit-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={navigateEdit}
        >
          {$page.data.internationalization?.edit}
        </a>
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="pagerate-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => {
            showSource = false
            pagePaneState.set(PagePane.Vote)
          }}
        >
          {$page.data.internationalization?.vote}
        </a>
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="history-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => {
            showSource = false
            pagePaneState.set(PagePane.History)
          }}
        >
          {$page.data.internationalization?.history}
        </a>
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="files-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => {
            showSource = false
            pagePaneState.set(PagePane.File)
          }}
        >
          {$page.data.internationalization?.files}
        </a>

        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="more-options-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => {
            toggleShowPageOptions()
          }}
        >
          {(showPageOptions ? "- " : "+ ") + $page.data.internationalization?.options}
        </a>
      </div>
    </div>

    {#if showPageOptions}
      <div id="page-options-bottom-2" class="page-options-bottom form-actions">
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="view-source-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => (showSource = true)}
        >
          {$page.data.internationalization?.["wiki-page-view-source"]}
        </a>
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="layout-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => {
            showSource = false
            pagePaneState.set(PagePane.Layout)
          }}
        >
          {$page.data.internationalization?.layout}
        </a>
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="parent-page-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => {
            showSource = false
            pagePaneState.set(PagePane.Parent)
          }}
        >
          {$page.data.internationalization?.parents}
        </a>
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="rename-move-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => {
            showSource = false
            pagePaneState.set(PagePane.Move)
          }}
        >
          {$page.data.internationalization?.move}
        </a>
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          id="delete-button"
          class="btn btn-default"
          href="javascript:;"
          type="button"
          on:click={() => {
            showSource = false
            pagePaneState.set(PagePane.Delete)
          }}
        >
          {$page.data.internationalization?.delete}
        </a>
      </div>
    {/if}

    <div id="action-area" class:hidden={!showSource && $pagePaneState === PagePane.None}>
      {#if showSource || $pagePaneState !== PagePane.None}
        <!-- svelte-ignore a11y-invalid-attribute -->
        <a
          class="action-area-close btn btn-danger"
          href="javascript:;"
          type="button"
          on:click={() => {
            showSource = false
            pagePaneState.set(PagePane.None)
          }}
        >
          {$page.data.internationalization?.close}
        </a>
      {/if}

      {#if showSource}
        <h1 class="page-source-header">
          {$page.data.internationalization?.["wiki-page-source"]}
        </h1>
        <div class="page-source">{$page.data.wikitext ?? ""}</div>
      {:else if $pagePaneState === PagePane.Move}
        <MovePane />
      {:else if $pagePaneState === PagePane.Layout}
        <LayoutPane />
      {:else if $pagePaneState === PagePane.Parent}
        <ParentPane />
      {:else if $pagePaneState === PagePane.Vote}
        <VotePane />
      {:else if $pagePaneState === PagePane.File}
        <FilePane />
      {:else if $pagePaneState === PagePane.History}
        <HistoryPane {setRevision} {setShowRevision} />
      {:else if $pagePaneState === PagePane.Delete}
        <DeletePane />
      {/if}
    </div>
  {/if}
{:else}
  {#if $page.data.options?.debug}
    <h2>UNTRANSLATED:Debug Response</h2>
  {:else if showRevision}
    <h2>{revision.title}</h2>
  {:else}
    <h2>{$page.data.page_revision.title}</h2>
  {/if}

  <hr />

  <div class="page-content">
    {#if $page.data.options?.debug}
      <textarea class="debug">{JSON.stringify($page, null, 2)}</textarea>
    {:else if $page.data.options?.no_render}
      {$page.data.internationalization["wiki-page-no-render"]}
      <textarea class="page-source" readonly={true}>{$page.data.wikitext}</textarea>
    {:else if showRevision}
      {@html revision.compiled_body_html}
    {:else}
      {@html $page.data.compiled_body_html}
    {/if}
  </div>

  <div class="page-tags-container">
    {$page.data.internationalization?.tags}
    <hr />
    <ul class="page-tags">
      {#if showRevision}
        {#each revision.tags as tag}
          <li class="tag">{tag}</li>
        {/each}
      {:else}
        {#each $page.data.page_revision.tags as tag}
          <li class="tag">{tag}</li>
        {/each}
      {/if}
    </ul>
  </div>

  <div class="page-meta-info-container">
    <div class="page-meta-info info-revision">
      {$page.data.internationalization["wiki-page-revision"]}
    </div>
    <div class="page-meta-info info-last-edit">
      {$page.data.internationalization["wiki-page-last-edit"]}
    </div>
  </div>

  {#if $page.data.options?.edit}
    <EditorPane />
  {:else}
    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-move clickable"
        type="button"
        on:click={() => {
          pagePaneState.set(PagePane.Move)
        }}
      >
        {$page.data.internationalization?.move}
      </button>
      <button
        class="action-button editor-button button-layout clickable"
        type="button"
        on:click={() => {
          pagePaneState.set(PagePane.Layout)
        }}
      >
        {$page.data.internationalization?.layout}
      </button>
      <button
        class="action-button editor-button button-parents clickable"
        type="button"
        on:click={() => {
          pagePaneState.set(PagePane.Parent)
        }}
      >
        {$page.data.internationalization?.parents}
      </button>
      <button
        class="action-button editor-button button-delete clickable"
        type="button"
        on:click={() => {
          pagePaneState.set(PagePane.Delete)
        }}
      >
        {$page.data.internationalization?.delete}
      </button>
      <button
        class="action-button editor-button button-edit clickable"
        type="button"
        on:click={navigateEdit}
      >
        {$page.data.internationalization?.edit}
      </button>
    </div>
    <div class="action-row other-actions">
      <button
        class="action-button button-source clickable"
        type="button"
        on:click={() => (showSource = true)}
      >
        {$page.data.internationalization?.["wiki-page-view-source"]}
      </button>
      <button
        class="action-button button-history clickable"
        type="button"
        on:click={() => {
          pagePaneState.set(PagePane.History)
        }}
      >
        {$page.data.internationalization?.history}
      </button>
      <button
        class="action-button button-vote clickable"
        type="button"
        on:click={() => {
          pagePaneState.set(PagePane.Vote)
        }}
      >
        {$page.data.internationalization?.vote}
      </button>
      <button
        class="action-button button-files clickable"
        type="button"
        on:click={() => {
          pagePaneState.set(PagePane.File)
        }}
      >
        {$page.data.internationalization?.files}
      </button>
    </div>
  {/if}

  {#if showSource}
    <h2 class="page-source-header">
      {$page.data.internationalization?.["wiki-page-source"]}
    </h2>
    <textarea class="page-source" readonly={true}>{$page.data.wikitext ?? ""}</textarea>
  {/if}

  {#if $pagePaneState === PagePane.Move}
    <MovePane />
  {:else if $pagePaneState === PagePane.Layout}
    <LayoutPane />
  {:else if $pagePaneState === PagePane.Parent}
    <ParentPane />
  {:else if $pagePaneState === PagePane.Vote}
    <VotePane />
  {:else if $pagePaneState === PagePane.File}
    <FilePane />
  {:else if $pagePaneState === PagePane.History}
    <HistoryPane {setRevision} {setShowRevision} />
  {:else if $pagePaneState === PagePane.Delete}
    <DeletePane />
  {/if}
{/if}

<style global lang="scss">
  .debug {
    width: 100%;
    height: 60vh;
  }

  .page-content,
  .page-tags-container,
  .page-meta-info-container,
  .editor-actions,
  .other-actions {
    padding: 0 0 2em;
  }

  .page-tags {
    display: flex;
    flex-direction: row;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    justify-content: flex-start;
    padding: 0;
    margin: 0;
    list-style: none;
  }

  .page-meta-info-container {
    text-align: right;
  }

  textarea.page-source {
    width: 100%;
    height: 60vh;
  }

  div.page-source {
    width: calc(100% - 4em - 2px);
    padding: 1em 2em;
    height: fit-content;
    white-space: pre-wrap;
  }

  .action-row {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: stretch;
    justify-content: flex-end;
    width: 100%;
  }
</style>
