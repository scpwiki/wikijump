<script lang="ts">
  import { deserialize } from "$app/forms"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { getPageLayoutContext } from "$lib/layout/page-layout-context"
  import { Layout } from "$lib/types"
  import { SvelteMap } from "svelte/reactivity"

  import type { PageProps } from "./$types"
  import type { Optional, PageVoteModel } from "$lib/types"
  import type { PageScore } from "$lib/server/deepwell/page"

  let { data }: PageProps = $props()

  const pageLayoutContext = getPageLayoutContext()
  const pageRating = $derived(data.page_rating)

  let showVoteList = $state<boolean>(false)
  let voteMap = new SvelteMap<number, PageVoteModel>()
  let voteRating = $state<number>()
  let whoRatedBody = $state<string>("")

  async function getWikidotWhoRated() {
    const pageId = data.page?.page_id
    if (pageId === undefined) return
    const response = await fetch("/ajax-module-connector.php", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: new URLSearchParams({
        moduleName: "pagerate/WhoRatedPageModule",
        pageId: String(pageId)
      })
    })
    const result = (await response.json()) as {
      status: string
      body?: string
      message?: string
    }
    if (response.ok && result.status === "ok" && typeof result.body === "string") {
      whoRatedBody = result.body
      showVoteList = true
      return
    }
    errorPopupState.current = {
      state: true,
      message: result.message ?? "Unable to list page ratings",
      data: {}
    }
  }

  async function getVoteList() {
    const res = await fetch(`?/voteGet`, {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id,
        slug: data.page?.slug
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: PageVoteModel[] },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data.data
      }
    } else if (result.type === "success" && result.data?.res) {
      voteMap.clear()
      result.data.res.forEach((vote) => {
        voteMap.set(vote.user_id, vote)
      })
    }
  }

  async function castVote(value = 0) {
    const res = await fetch(`?/voteCast`, {
      method: "POST",
      body: JSON.stringify({
        pageId: data.page?.page_id,
        value
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: Optional<PageVoteModel> },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data.data
      }
    } else if (result.type === "success") {
      await fetchVoteRating()
    }
  }

  async function cancelVote() {
    const res = await fetch(`?/voteCancel`, {
      method: "POST",
      body: JSON.stringify({
        pageId: data.page?.page_id
      })
    }).then((res) => res.text())

    const result = deserialize<
      { res: PageVoteModel },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data.data
      }
    } else if (result.type === "success") {
      await fetchVoteRating()
    }
  }

  async function fetchVoteRating() {
    const res = await fetch(`?/score`, {
      method: "POST",
      body: JSON.stringify({})
    }).then((res) => res.text())

    const result = deserialize<
      { res: PageScore },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data.data
      }
    } else if (result.type === "success" && result.data?.res) {
      voteRating = result.data.res.score
    }
  }

  function starAsset(index: number) {
    const threshold = index + 1
    const rating = voteRating ?? 0
    const asset =
      rating >= threshold
        ? "star-on.png"
        : rating >= threshold - 0.5
          ? "star-half.png"
          : "star-off.png"
    return `/common--images/jquery-raty/${asset}`
  }

  $effect(() => {
    fetchVoteRating()
  })
</script>

{#if pageLayoutContext.current === Layout.WIKIDOT}
  <h1 class="page-vote-header">
    {data.internationalization["wiki-page-vote"]}
  </h1>
  <p>{data.internationalization["wiki-page-vote.description"]}</p>
  <div class="page-rate-widget-area">
    {#if pageRating.rating_type === "stars"}
      <div class="page-rate-widget">
        <div
          style:cursor="pointer"
          style:width="100px"
          class="page-rate-widget-start"
          data-rating={voteRating ?? 0}
        >
          {#each ["Poor", "Fair", "Good", "Very Good", "Excellent"] as title, index (title)}
            <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <img
              alt={`${index + 1}`}
              onclick={() => castVote(index + 1)}
              src={starAsset(index)}
              {title}
            />{#if index < 4}&nbsp;{/if}
          {/each}
          <input name="score" type="hidden" />
        </div>
      </div>
    {:else}
      <div class="page-rate-widget-box">
        <span class="rate-points">
          {data.internationalization["wiki-page-vote.score"]}&nbsp;
          <span class="number prw54353">{voteRating}</span>
        </span><span class="rateup btn btn-default">
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a
            href="javascript:;"
            onclick={() => castVote(1)}
            aria-label={data.internationalization["wiki-page-vote.like"]}
            title={data.internationalization["wiki-page-vote.like"]}
            type="button">+</a
          >
        </span>{#if pageRating.rating_type === "plus_minus"}<span
            class="ratedown btn btn-default"
          >
            <!-- svelte-ignore a11y_invalid_attribute -->
            <a
              href="javascript:;"
              onclick={() => castVote(-1)}
              aria-label={data.internationalization["wiki-page-vote.dislike"]}
              title={data.internationalization["wiki-page-vote.dislike"]}
              type="button">–</a
            >
          </span>{/if}<span class="cancel btn btn-default">
          <!-- svelte-ignore a11y_invalid_attribute -->
          <a
            href="javascript:;"
            onclick={cancelVote}
            aria-label={data.internationalization["wiki-page-vote.cancel"]}
            title={data.internationalization["wiki-page-vote.cancel"]}
            type="button">x</a
          >
        </span>
      </div>
    {/if}
  </div>
  {#if pageRating.visibility === "visible"}
    <p>
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a href="javascript:;" onclick={getWikidotWhoRated}>
        {data.internationalization["wiki-page-vote.who-rated"]}
      </a>
    </p>
    <div id="who-rated-page-area">
      {#if showVoteList}
        {@html whoRatedBody}
      {/if}
    </div>
  {/if}
{:else}
  <h2 class="page-vote-header">
    {data.internationalization["wiki-page-vote"]}
  </h2>
  <div class="vote-panel">
    <div class="action-row vote-action">
      {#if pageRating.visibility === "visible"}
        <button
          class="action-button view-vote-list clickable"
          onclick={() =>
            getVoteList().then(() => {
              showVoteList = true
            })}
          type="button"
        >
          {data.internationalization["wiki-page-vote.list"]}
        </button>
      {/if}
      <div class="action-button vote-rating">
        <span class="vote-desc">
          {data.internationalization["wiki-page-vote.score"]}
        </span>
        <span class="vote-rating-number">{voteRating}</span>
      </div>
      <div class="action-button cast-vote">
        <span class="vote-desc">
          {data.internationalization["wiki-page-vote.set"]}
        </span>
        {#if pageRating.rating_type === "stars"}
          {#each [1, 2, 3, 4, 5] as value (value)}
            <button
              class="vote-subbutton clickable"
              onclick={() => castVote(value)}
              type="button"
            >
              {value}
            </button>
          {/each}
        {:else}
          <button
            class="vote-subbutton clickable"
            onclick={() => castVote(1)}
            type="button"
          >
            +1
          </button>
          {#if pageRating.rating_type === "plus_minus"}
            <button
              class="vote-subbutton clickable"
              onclick={() => castVote(-1)}
              type="button"
            >
              -1
            </button>
          {/if}
        {/if}
      </div>
      <button
        class="action-button remove-vote clickable"
        onclick={cancelVote}
        type="button"
      >
        {data.internationalization["wiki-page-vote.remove"]}
      </button>
    </div>
    {#if showVoteList}
      <ul class="vote-list">
        {#each [...voteMap].sort((a, b) => b[0] - a[0]) as [userId, vote] (vote.page_vote_id)}
          <li class="vote-item" data-id={vote.page_vote_id} data-user-id={userId}>
            {data.internationalization["wiki-page-vote.user"]}
            {vote.user_id}: {vote.value}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
{/if}

<style global lang="scss">
  .page-rate-widget-area {
    text-align: center;
  }
</style>
