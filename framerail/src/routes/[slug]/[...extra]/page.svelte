<script lang="ts">
  import { page } from "$app/state"
  import { goto } from "$app/navigation"
  import { pageLayoutState, errorPopupState } from "$lib/stores.svelte"
  import { Layout, PagePane } from "$lib/types"
  import {
    DeletePane,
    EditorPane,
    FilePane,
    HistoryPane,
    LayoutPane,
    LockPane,
    MovePane,
    ParentPane,
    VotePane
  } from "."
  import { resolve } from "$app/paths"

  import type { PageProps } from "./$types"
  import type { Optional } from "$lib/types"
  import type { PageRevisionModelFiltered } from "$lib/server/deepwell/page"
  import { deserialize } from "$app/forms"

  let props: PageProps = $props()
  let { data } = $derived(props)

  let showSource = $state<boolean>(false)
  let showPageOptions = $state<boolean>(false)
  let showRevision = $state<boolean>(false)
  let revision = $state<Optional<PageRevisionModelFiltered>>(undefined)
  let pagePaneState = $state<PagePane>(PagePane.None)

  async function navigateEdit() {
    // Check edit permission first
    const res = await fetch("?/editPermission", {
      method: "POST",
      body: ""
    }).then((res) => res.text())

    const result = deserialize<
      { res: { can_edit: boolean } },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      if (!result.data.res.can_edit) {
        errorPopupState.current = {
          state: true,
          message: "UNTRANSLATED:You don't have permission to edit this page",
          data: null
        }
      } else {
        // Permission granted, navigate to edit page
        const options: string[] = Object.entries({
          norender: data.options.no_render,
          noredirect: data.options.no_redirect,
          debug: data.options.debug
        })
          .filter(([, enabled]) => enabled)
          .map(([key]) => `/${key}`)

        goto(resolve(`/${data.page!.slug}${options.join("")}/edit`, {}), {
          noScroll: true
        })
      }
    }
  }

  function setShowRevision(state: boolean) {
    showRevision = state
  }

  function toggleShowPageOptions(state?: boolean) {
    if (state !== undefined) showPageOptions = state
    else showPageOptions = !showPageOptions
  }

  function setRevision(rev: Optional<PageRevisionModelFiltered>) {
    revision = rev
  }

  $effect(() => {
    if (data.options?.history) {
      pagePaneState = PagePane.History
    }
  })
</script>

<svelte:head>
  <title>{data.page_revision?.title} | {data.site.name}</title>
</svelte:head>

{#if pageLayoutState.current === Layout.WIKIDOT}
  {#if data.options?.debug}
    <h2>UNTRANSLATED:Debug Response</h2>
  {:else if showRevision}
    <div id="page-title">{revision?.title}</div>
  {:else}
    <div id="page-title">{data.page_revision?.title}</div>
  {/if}

  <div id="page-content">
    {#if data.options?.debug}
      <textarea class="debug">{JSON.stringify(page, null, 2)}</textarea>
    {:else if data.options?.no_render}
      {data.internationalization?.["wiki-page-no-render"]}
      <textarea class="page-source" readonly={true}>{data.wikitext}</textarea>
    {:else if showRevision}
      {@html revision?.compiled_body_html}
    {:else}
      {@html data.compiled_body_html}
    {/if}
  </div>

  {#if showRevision}
    {#if revision?.tags?.length}
      <div class="page-tags">
        <span
          >{#each revision.tags as tag (tag)}
            <a href={resolve(`/system:page-tags/tag/${tag}`, {})}>{tag}</a>
          {/each}</span
        >
      </div>
    {/if}
  {:else if data.page_revision?.tags?.length}
    <div class="page-tags">
      <span
        >{#each data.page_revision?.tags as tag (tag)}
          <a href={resolve(`/system:page-tags/tag/${tag}`, {})}>{tag}</a>
        {/each}</span
      >
    </div>
  {/if}

  {#if data.options?.edit}
    <div id="page-options-container">
      <div id="page-info">
        {data.internationalization?.["wiki-page-revision"]}, {data.internationalization?.[
          "wiki-page-last-edit"
        ]}
      </div>
    </div>
    <div id="action-area">
      <EditorPane {...props} />
    </div>
  {:else}
    <div id="page-options-container">
      <div id="page-info">
        {data.internationalization?.["wiki-page-revision"]}, {data.internationalization?.[
          "wiki-page-last-edit"
        ]}
      </div>
      <div
        id="page-options-bottom"
        class="page-options-bottom"
        class:hidden={!!data.options?.edit}
      >
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="edit-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={navigateEdit}
          type="button"
        >
          {data.internationalization?.edit}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="pagerate-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.Vote
          }}
          type="button"
        >
          {data.internationalization?.vote}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="history-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.History
          }}
          type="button"
        >
          {data.internationalization?.history}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="files-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.File
          }}
          type="button"
        >
          {data.internationalization?.files}
        </a>

        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="more-options-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => toggleShowPageOptions()}
          type="button"
        >
          {(showPageOptions ? "- " : "+ ") + data.internationalization?.options}
        </a>
      </div>
    </div>

    {#if showPageOptions}
      <div id="page-options-bottom-2" class="page-options-bottom form-actions">
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="view-source-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => (showSource = true)}
          type="button"
        >
          {data.internationalization?.["wiki-page-view-source"]}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="layout-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.Layout
          }}
          type="button"
        >
          {data.internationalization?.layout}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="parent-page-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.Parent
          }}
          type="button"
        >
          {data.internationalization?.parents}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="lock-page-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.Lock
          }}
          type="button"
        >
          {data.internationalization?.["wiki-page-lock"]}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="rename-move-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.Move
          }}
          type="button"
        >
          {data.internationalization?.move}
        </a>
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          id="delete-button"
          class="btn btn-default"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.Delete
          }}
          type="button"
        >
          {data.internationalization?.delete}
        </a>
      </div>
    {/if}

    <div id="action-area" class:hidden={!showSource && pagePaneState === PagePane.None}>
      {#if showSource || pagePaneState !== PagePane.None}
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a
          class="action-area-close btn btn-danger"
          href="javascript:;"
          onclick={() => {
            showSource = false
            pagePaneState = PagePane.None
          }}
          type="button"
        >
          {data.internationalization?.close}
        </a>
      {/if}

      {#if showSource}
        <h1 class="page-source-header">
          {data.internationalization?.["wiki-page-source"]}
        </h1>
        <div class="page-source">{data.wikitext ?? ""}</div>
      {:else if pagePaneState === PagePane.Move}
        <MovePane bind:pagePaneState {...props} />
      {:else if pagePaneState === PagePane.Layout}
        <LayoutPane bind:pagePaneState {...props} />
      {:else if pagePaneState === PagePane.Parent}
        <ParentPane bind:pagePaneState {...props} />
      {:else if pagePaneState === PagePane.Lock}
        <LockPane bind:pagePaneState {...props} />
      {:else if pagePaneState === PagePane.Vote}
        <VotePane {...props} />
      {:else if pagePaneState === PagePane.File}
        <FilePane {...props} />
      {:else if pagePaneState === PagePane.History}
        <HistoryPane {setRevision} {setShowRevision} {...props} />
      {:else if pagePaneState === PagePane.Delete}
        <DeletePane bind:pagePaneState {...props} />
      {/if}
    </div>
  {/if}
{:else}
  {#if data.options?.debug}
    <h2>UNTRANSLATED:Debug Response</h2>
  {:else if showRevision}
    <h2 class="page-title">{revision?.title}</h2>
  {:else}
    <h2 class="page-title">{data.page_revision?.title}</h2>
  {/if}

  <hr />

  <div class="page-content">
    {#if data.options?.debug}
      <textarea class="debug">{JSON.stringify(page, null, 2)}</textarea>
    {:else if data.options?.no_render}
      {data.internationalization?.["wiki-page-no-render"]}
      <textarea class="page-source" readonly={true}>{data.wikitext}</textarea>
    {:else if showRevision}
      {@html revision?.compiled_body_html}
    {:else}
      {@html data.compiled_body_html}
    {/if}
  </div>

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

  {#if data.options?.edit}
    <EditorPane {...props} />
  {:else}
    <div class="action-row editor-actions">
      <button
        class="action-button editor-button button-move clickable"
        onclick={() => (pagePaneState = PagePane.Move)}
        type="button"
      >
        {data.internationalization?.move}
      </button>
      <button
        class="action-button editor-button button-layout clickable"
        onclick={() => (pagePaneState = PagePane.Layout)}
        type="button"
      >
        {data.internationalization?.layout}
      </button>
      <button
        class="action-button editor-button button-parents clickable"
        onclick={() => (pagePaneState = PagePane.Parent)}
        type="button"
      >
        {data.internationalization?.parents}
      </button>
      <button
        class="action-button editor-button button-lock clickable"
        onclick={() => (pagePaneState = PagePane.Lock)}
        type="button"
      >
        {data.internationalization?.["wiki-page-lock"]}
      </button>
      <button
        class="action-button editor-button button-delete clickable"
        onclick={() => (pagePaneState = PagePane.Delete)}
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
        onclick={() => (pagePaneState = PagePane.History)}
        type="button"
      >
        {data.internationalization?.history}
      </button>
      <button
        class="action-button button-vote clickable"
        onclick={() => (pagePaneState = PagePane.Vote)}
        type="button"
      >
        {data.internationalization?.vote}
      </button>
      <button
        class="action-button button-files clickable"
        onclick={() => (pagePaneState = PagePane.File)}
        type="button"
      >
        {data.internationalization?.files}
      </button>
    </div>
  {/if}

  {#if showSource}
    <h2 class="page-source-header">
      {data.internationalization?.["wiki-page-source"]}
    </h2>
    <textarea class="page-source" readonly={true}>{data.wikitext ?? ""}</textarea>
  {/if}

  {#if pagePaneState === PagePane.Move}
    <MovePane bind:pagePaneState {...props} />
  {:else if pagePaneState === PagePane.Layout}
    <LayoutPane bind:pagePaneState {...props} />
  {:else if pagePaneState === PagePane.Parent}
    <ParentPane bind:pagePaneState {...props} />
  {:else if pagePaneState === PagePane.Lock}
    <LockPane bind:pagePaneState {...props} />
  {:else if pagePaneState === PagePane.Vote}
    <VotePane {...props} />
  {:else if pagePaneState === PagePane.File}
    <FilePane {...props} />
  {:else if pagePaneState === PagePane.History}
    <HistoryPane {setRevision} {setShowRevision} {...props} />
  {:else if pagePaneState === PagePane.Delete}
    <DeletePane bind:pagePaneState {...props} />
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
    height: fit-content;
    padding: 1em 2em;
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
