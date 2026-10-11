<script lang="ts">
  import { deserialize } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"

  import { Layout } from "$lib/types"
  import {
    wikidotHistoryActionTitles,
    wikidotHistoryHeaders,
    wikidotRevisionDate,
    wikidotRevisionFlags,
    wikidotRevisionSourceHtml
  } from "$lib/wikidot-history-contract"
  import { onDestroy, tick } from "svelte"
  import { SvelteMap } from "svelte/reactivity"

  import type { PageProps } from "./$types"
  import type {
    PageRevisionModelFiltered,
    PageHistoryEntry,
    CreatePageRevisionOutput,
    PageRevisionDiffOutput
  } from "$lib/server/deepwell/page"
  import type { Optional } from "$lib/types"
  import RevisionAuthor from "./RevisionAuthor.svelte"
  import WikidotRevisionAuthor from "./WikidotRevisionAuthor.svelte"

  interface Props extends PageProps {
    setShowRevision: (val: boolean) => void
    setRevision: (rev: Optional<PageRevisionModelFiltered>) => void
  }

  let { setShowRevision, setRevision, data }: Props = $props()
  const historyPaneId = $props.id()

  const pageLayoutContext = getPageLayoutContext()

  let revisionMap = new SvelteMap<number, PageHistoryEntry>()
  let revision = $state<Optional<PageRevisionModelFiltered>>(undefined)
  let showRevisionSource = $state<boolean>(false)
  let fromRevisionNumber = $state<Optional<number>>(undefined)
  let toRevisionNumber = $state<Optional<number>>(undefined)
  let revisionDiff = $state<Optional<PageRevisionDiffOutput>>(undefined)
  let revisionDiffLoading = $state(false)
  let revisionDiffCompareButton = $state<HTMLButtonElement | undefined>(undefined)
  let revisionDiffRequestId = 0
  let active = true
  let latestRevisionNumber = $derived(Math.max(...revisionMap.keys()))

  onDestroy(() => {
    active = false
    revisionDiffRequestId += 1
  })

  const SVELTEKIT_ACTION_HEADERS = {
    accept: "application/json",
    "content-type": "text/plain;charset=UTF-8",
    "x-sveltekit-action": "true"
  }

  async function fetchHistory() {
    const res = await fetch("?/history", {
      method: "POST",
      headers: SVELTEKIT_ACTION_HEADERS,
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        includeFileRevisions: pageLayoutContext.current === Layout.WIKIDOT
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: PageHistoryEntry[] },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (!active) return

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      revisionMap.clear()
      result.data.res.forEach((rawRevision) => {
        const rev: PageHistoryEntry = {
          ...rawRevision,
          history_kind: rawRevision.history_kind ?? "page",
          history_row_id: rawRevision.history_row_id ?? String(rawRevision.revision_id),
          timeline_number: rawRevision.timeline_number ?? rawRevision.revision_number,
          page_revision_number:
            rawRevision.page_revision_number === undefined
              ? rawRevision.history_kind === "file"
                ? null
                : rawRevision.revision_number
              : rawRevision.page_revision_number
        }
        revisionMap.set(rev.timeline_number, rev)
      })
      const revisionNumbers = result.data.res
        .map((rev) => rev.timeline_number)
        .sort((a, b) => a - b)
      if (fromRevisionNumber === undefined) {
        fromRevisionNumber = revisionNumbers.at(-2)
      }
      if (toRevisionNumber === undefined) {
        toRevisionNumber = revisionNumbers.at(-1)
      }
    }
  }

  async function fetchRevisionDiff() {
    if (fromRevisionNumber === undefined || toRevisionNumber === undefined) return

    const fromRevision = revisionMap.get(fromRevisionNumber)?.page_revision_number
    const toRevision = revisionMap.get(toRevisionNumber)?.page_revision_number
    if (
      fromRevision === undefined ||
      fromRevision === null ||
      toRevision === undefined ||
      toRevision === null
    )
      return

    const restoreCompareFocus = document.activeElement === revisionDiffCompareButton
    const requestedFromRevisionNumber = fromRevisionNumber
    const requestedToRevisionNumber = toRevisionNumber
    const requestId = ++revisionDiffRequestId
    revisionDiffLoading = true
    try {
      const res = await fetch("?/revisionDiff", {
        method: "POST",
        headers: SVELTEKIT_ACTION_HEADERS,
        body: JSON.stringify({
          siteId: data.site.site_id,
          pageId: data.page?.page_id,
          fromRevisionNumber: fromRevision,
          toRevisionNumber: toRevision
        })
      }).then((response) => response.text())

      const result = deserialize<
        { res: Optional<PageRevisionDiffOutput> },
        { message: string; code: string; data: Record<string, unknown> }
      >(res)

      if (
        !active ||
        requestId !== revisionDiffRequestId ||
        requestedFromRevisionNumber !== fromRevisionNumber ||
        requestedToRevisionNumber !== toRevisionNumber
      ) {
        return
      }

      if (result.type === "failure" && result.data?.message) {
        errorPopupState.current = {
          state: true,
          message: result.data.message,
          data: result.data
        }
      } else if (result.type === "success") {
        revisionDiff = result.data?.res
      }
    } finally {
      if (requestId === revisionDiffRequestId) {
        revisionDiffLoading = false
        if (restoreCompareFocus) {
          await tick()
          revisionDiffCompareButton?.focus()
        }
      }
    }
  }

  function clearRevisionDiff() {
    revisionDiffRequestId += 1
    revisionDiff = undefined
    revisionDiffLoading = false
  }

  function swapRevisionDiff() {
    const previousFrom = fromRevisionNumber
    fromRevisionNumber = toRevisionNumber
    toRevisionNumber = previousFrom
    clearRevisionDiff()
  }

  function revisionTypeLabel(entry: PageHistoryEntry) {
    const key =
      entry.history_kind === "file"
        ? `wiki-page-file-revision-type.${entry.revision_type}`
        : `wiki-page-revision-type.${entry.revision_type}`
    return data.internationalization?.[
      key as keyof NonNullable<typeof data.internationalization>
    ]
  }

  function revisionUnavailableLabel() {
    const key = "wiki-page-revision-view-unavailable"
    const translated = data.internationalization?.[key]
    return translated && translated !== key
      ? translated
      : "Historical page snapshot unavailable"
  }

  async function getRevision(
    timelineNumber: number,
    compiledHtml: boolean,
    wikitext: boolean
  ): Promise<boolean> {
    const entry = revisionMap.get(timelineNumber)
    const revisionNumber = entry?.page_revision_number
    if (revisionNumber === undefined || revisionNumber === null) return false

    const res = await fetch("?/revision", {
      method: "POST",
      headers: SVELTEKIT_ACTION_HEADERS,
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        revisionNumber,
        compiledHtml,
        wikitext
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: Optional<PageRevisionModelFiltered> },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (!active) return false

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      setRevision(result.data.res)
      revision = result.data.res
      return true
    }
    return false
  }

  async function rollbackRevision(timelineNumber: number, comments?: string) {
    const entry = revisionMap.get(timelineNumber)
    if (!entry || entry.history_kind !== "page") return
    if (!canEditPage) return
    // A rollback writes a new page revision. Ask before sending it; cancelling
    // sends no request.
    const label = data.internationalization?.["wiki-page-revision-rollback"] ?? "Revert"
    // eslint-disable-next-line no-alert
    if (!globalThis.confirm(`${label} ${entry.revision_number}?`)) return

    const res = await fetch("?/rollback", {
      method: "POST",
      headers: SVELTEKIT_ACTION_HEADERS,
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        revisionNumber: entry.revision_number,
        lastRevisionId: data.page_revision?.revision_id,
        comments
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: Optional<CreatePageRevisionOutput> },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (!active) return

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      invalidateAll()
    }
  }

  // Revert is offered only to viewers who may edit the page. The server still
  // revalidates every rollback; this keeps a submitting control away from
  // viewers who can never complete it.
  let canEditPage = $state(false)

  async function fetchEditPermission() {
    const res = await fetch("?/editPermission", {
      method: "POST",
      headers: SVELTEKIT_ACTION_HEADERS,
      body: JSON.stringify({})
    }).then((res) => res.text())

    const result = deserialize<
      { res: { can_edit: boolean } },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (!active) return
    canEditPage = result.type === "success" && result.data?.res?.can_edit === true
  }

  $effect(() => {
    fetchHistory()
  })

  $effect(() => {
    void fetchEditPermission()
  })
</script>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  <h1 class="page-revision-header">
    {data.internationalization?.["wiki-page-revision-history"]}
  </h1>
  <div id="revision-list" class="revision-list">
    <table class="page-history">
      <tbody>
        <tr>
          {#each wikidotHistoryHeaders(data.site.locale) as heading, index (index)}
            <td
              role="columnheader"
              aria-label={heading
                ? undefined
                : `${data.internationalization?.["wiki-page-revision-diff.from"]} / ${data.internationalization?.["wiki-page-revision-diff.to"]}`}
              >{heading || " "}</td
            >
          {/each}
        </tr>
        <!-- Here we sort the list in descending order. -->
        {#each [...revisionMap].sort((a, b) => b[0] - a[0]) as [, revisionItem] (revisionItem.timeline_number)}
          {@const date = wikidotRevisionDate(revisionItem.created_at)}
          <tr id={`revision-row-${revisionItem.history_row_id}`}>
            <td>{revisionItem.timeline_number + 1}.</td>
            <td style="width: 5em">
              <input
                id={`${historyPaneId}-from-${revisionItem.timeline_number}`}
                type="radio"
                name="from"
                value={revisionItem.revision_id}
                checked={revisionItem.timeline_number === fromRevisionNumber}
                aria-label={`${data.internationalization?.["wiki-page-revision-diff.from"]}: ${revisionItem.timeline_number + 1}`}
                onchange={() => {
                  fromRevisionNumber = revisionItem.timeline_number
                  clearRevisionDiff()
                }}
              />
              <input
                id={`${historyPaneId}-to-${revisionItem.timeline_number}`}
                type="radio"
                name="to"
                value={revisionItem.revision_id}
                checked={revisionItem.timeline_number === toRevisionNumber}
                aria-label={`${data.internationalization?.["wiki-page-revision-diff.to"]}: ${revisionItem.timeline_number + 1}`}
                onchange={() => {
                  toRevisionNumber = revisionItem.timeline_number
                  clearRevisionDiff()
                }}
              />
            </td>
            <td>
              {#each wikidotRevisionFlags(revisionItem, data.site.locale) as flag (flag.code)}
                <span class="spantip" title={flag.title}>{flag.code}</span>
              {/each}
            </td>
            <td style="width: 5em" class="optionstd">
              <!-- svelte-ignore a11y_invalid_attribute -->
              {#if revisionItem.history_kind === "file" && (revisionItem.page_revision_number === null || revisionItem.page_revision_number === undefined)}
                <span
                  class="revision-view-unavailable"
                  aria-label={revisionUnavailableLabel()}
                  title={revisionUnavailableLabel()}
                >
                  {revisionUnavailableLabel()}
                </span>
              {:else}
                <a
                  href="javascript:;"
                  title={wikidotHistoryActionTitles(data.site.locale).view}
                  onclick={async (event) => {
                    event.stopPropagation()
                    if (!(await getRevision(revisionItem.timeline_number, true, false))) {
                      return
                    }
                    if (!active) return
                    setShowRevision(true)
                    showRevisionSource = false
                  }}
                >
                  V
                </a>
              {/if}
              <!-- svelte-ignore a11y_invalid_attribute -->
              <a
                href="javascript:;"
                title={wikidotHistoryActionTitles(data.site.locale).source}
                onclick={(event) => {
                  event.stopPropagation()
                  getRevision(revisionItem.timeline_number, false, true).then(() => {
                    if (!active) return
                    setShowRevision(false)
                    showRevisionSource = true
                  })
                }}
              >
                S
              </a>
              {#if canEditPage && revisionItem.history_kind === "page" && revisionItem.timeline_number < latestRevisionNumber}
                <!-- svelte-ignore a11y_invalid_attribute -->
                <a
                  href="javascript:;"
                  title={wikidotHistoryActionTitles(data.site.locale).rollback}
                  onclick={(event) => {
                    event.stopPropagation()
                    rollbackRevision(revisionItem.timeline_number)
                  }}
                >
                  R
                </a>
              {/if}
            </td>
            <td style="width: 15em">
              <WikidotRevisionAuthor
                author={revisionItem.author}
                userId={revisionItem.user_id}
              />
            </td>
            <td style="padding: 0 0.5em; width: 7em;">
              {#if date}<span class={date.className}>{date.text}</span>{/if}
            </td>
            <td style="font-size: 90%">
              {revisionItem.comments}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>

  {#if showRevisionSource}
    <div id="history-subarea">
      {@html wikidotRevisionSourceHtml(revision?.wikitext ?? "")}
    </div>
  {/if}
{:else}
  <h2 class="page-revision-header">
    {data.internationalization?.["wiki-page-revision-history"]}
  </h2>
  <div class="revision-list">
    <div class="revision-header">
      <div class="revision-attribute action"></div>
      <div class="revision-attribute revision-number">
        {data.internationalization?.["wiki-page-revision-number"]}
      </div>
      <div class="revision-attribute revision-type">
        {data.internationalization?.["wiki-page-revision-type"]}
      </div>
      <div class="revision-attribute created-at">
        {data.internationalization?.["wiki-page-revision-created-at"]}
      </div>
      <div class="revision-attribute user">
        {data.internationalization?.["wiki-page-revision-user"]}
      </div>
      <div class="revision-attribute comments">
        {data.internationalization?.["wiki-page-revision-comments"]}
      </div>
    </div>
    <!-- Here we sort the list in descending order. -->
    {#each [...revisionMap].sort((a, b) => b[0] - a[0]) as [, revisionItem] (revisionItem.timeline_number)}
      <div class="revision-row" data-id={revisionItem.history_row_id}>
        <div class="revision-attribute action">
          {#if ["create", "regular"].includes(revisionItem.revision_type)}
            {#if revisionItem.history_kind === "page"}
              <button
                class="action-button view-revision clickable"
                onclick={(event) => {
                  event.stopPropagation()
                  getRevision(revisionItem.timeline_number, true, false).then(() => {
                    if (!active) return
                    setShowRevision(true)
                    showRevisionSource = false
                  })
                }}
                type="button"
              >
                {data.internationalization?.view}
              </button>
              <button
                class="action-button view-revision-source clickable"
                onclick={(event) => {
                  event.stopPropagation()
                  getRevision(revisionItem.timeline_number, false, true).then(() => {
                    if (!active) return
                    setShowRevision(false)
                    showRevisionSource = true
                  })
                }}
                type="button"
              >
                {data.internationalization?.["wiki-page-view-source"]}
              </button>
              {#if canEditPage}
                <button
                  class="action-button revision-rollback clickable"
                  onclick={(event) => {
                    event.stopPropagation()
                    rollbackRevision(revisionItem.timeline_number)
                  }}
                  type="button"
                >
                  {data.internationalization?.["wiki-page-revision-rollback"]}
                </button>
              {/if}
            {/if}
          {/if}
        </div>
        <div class="revision-attribute revision-number">
          {revisionItem.revision_number}
        </div>
        <div class="revision-attribute revision-type">
          {revisionTypeLabel(revisionItem)}
        </div>
        <div class="revision-attribute created-at">
          {new Date(revisionItem.created_at).toLocaleString()}
        </div>
        <div class="revision-attribute user">
          <RevisionAuthor author={revisionItem.author} />
        </div>
        <div class="revision-attribute comments">
          {revisionItem.comments}
        </div>
      </div>
    {/each}
  </div>

  {#if showRevisionSource}
    <textarea class="revision-source" readonly={true}>{revision?.wikitext ?? ""}</textarea
    >
  {/if}
{/if}

{#if revisionMap.size >= 2}
  <section class="revision-diff-panel" aria-labelledby="revision-diff-heading">
    <h2 id="revision-diff-heading">
      {data.internationalization?.["wiki-page-revision-diff"]}
    </h2>
    <div class="revision-diff-controls">
      <label for="revision-diff-from">
        {data.internationalization?.["wiki-page-revision-diff.from"]}
      </label>
      <select
        id="revision-diff-from"
        onchange={clearRevisionDiff}
        bind:value={fromRevisionNumber}
      >
        {#each [...revisionMap.keys()].sort((a, b) => a - b) as revisionNumber (revisionNumber)}
          <option value={revisionNumber}>{revisionNumber}</option>
        {/each}
      </select>
      <label for="revision-diff-to">
        {data.internationalization?.["wiki-page-revision-diff.to"]}
      </label>
      <select
        id="revision-diff-to"
        onchange={clearRevisionDiff}
        bind:value={toRevisionNumber}
      >
        {#each [...revisionMap.keys()].sort((a, b) => a - b) as revisionNumber (revisionNumber)}
          <option value={revisionNumber}>{revisionNumber}</option>
        {/each}
      </select>
      <button class="action-button clickable" onclick={swapRevisionDiff} type="button">
        {data.internationalization?.["wiki-page-revision-diff.swap"]}
      </button>
      <button
        bind:this={revisionDiffCompareButton}
        class="action-button clickable"
        disabled={revisionDiffLoading}
        onclick={fetchRevisionDiff}
        type="button"
      >
        {revisionDiffLoading
          ? data.internationalization?.["wiki-page-revision-diff.loading"]
          : data.internationalization?.["wiki-page-revision-diff.compare"]}
      </button>
    </div>
    {#if revisionDiff}
      {#if revisionDiff.lines.length === 0}
        <p>{data.internationalization?.["wiki-page-revision-diff.no-changes"]}</p>
      {:else}
        <pre
          class="revision-diff"
          aria-live="polite">{#each revisionDiff.lines as line, lineIndex (lineIndex)}<span
              class="revision-diff-line"
              class:added={line.kind === "added"}
              class:removed={line.kind === "removed"}
              class:unchanged={line.kind === "unchanged"}
              >{line.kind === "added"
                ? "+"
                : line.kind === "removed"
                  ? "-"
                  : " "}{line.text}&#10;</span
            >{/each}</pre>
      {/if}
    {/if}
  </section>
{/if}

<style lang="scss">
  textarea.revision-source {
    width: 100%;
    height: 60vh;
  }

  .revision-list {
    max-width: 100%;
  }

  .revision-diff-panel {
    margin-top: 1rem;
  }

  .revision-diff-controls {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
  }

  .revision-diff {
    padding: 0.75rem;
    margin-top: 0.75rem;
    width: 100%;
    max-width: 100%;
    min-width: 0;
    overflow: auto;
    box-sizing: border-box;
    white-space: pre-wrap !important;
    overflow-wrap: anywhere !important;
    word-break: normal;

    .revision-diff-line {
      display: block;
    }

    .added {
      background: rgb(220 255 220);
    }

    .removed {
      background: rgb(255 225 225);
    }
  }

  // Theme selectors commonly target `pre` descendants with `white-space: pre`.
  // Keep the UI's individual revision lines bounded and readable even when
  // that upstream declaration would otherwise make the pre horizontally wider
  // than a phone viewport.
  :global(#action-area pre.revision-diff > .revision-diff-line) {
    white-space: pre-wrap !important;
    overflow-wrap: anywhere !important;
    max-width: 100%;
    box-sizing: border-box;
  }

  /* Theme CSS can set the page-wide light text color with enough reach to
     wash out these pale semantic diff rows. Keep added/removed source legible
     while retaining the conventional green/red distinction in every theme. */
  :global(#action-area .revision-diff .revision-diff-line.added) {
    color: #173421 !important;
    background-color: rgb(220 255 220) !important;
  }

  :global(#action-area .revision-diff .revision-diff-line.removed) {
    color: #4a2020 !important;
    background-color: rgb(255 225 225) !important;
  }
</style>
