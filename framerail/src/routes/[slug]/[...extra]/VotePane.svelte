<script lang="ts">
  import { deserialize } from "$app/forms"
  import { errorPopupState, pageLayoutState } from "$lib/stores.svelte"
  import { toast } from "$lib/component/scripts/toasts"
  import { Layout, ToastType } from "$lib/types"
  import { SvelteMap } from "svelte/reactivity"

  import type { PageProps } from "./$types"
  import type { Optional, PageVoteModel } from "$lib/types"
  import type { PageScore } from "$lib/server/deepwell/page"

  let { data }: PageProps = $props()

  let showVoteList = $state<boolean>(false)
  let voteMap = new SvelteMap<number, PageVoteModel>()
  let voteRating = $state<number>()

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
        siteId: data.site.site_id,
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
      toast(ToastType.Success, data.internationalization!["wiki-page-vote.toast-set"]!)
    }
  }

  async function cancelVote() {
    const res = await fetch(`?/voteCancel`, {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
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
      toast(ToastType.Success, data.internationalization!["wiki-page-vote.toast-remove"]!)
    }
  }

  async function fetchVoteRating() {
    const res = await fetch(`?/score`, {
      method: "POST",
      body: JSON.stringify({
        siteId: data.site.site_id,
        pageId: data.page?.page_id
      })
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

  $effect(() => {
    fetchVoteRating()
  })
</script>

{#if pageLayoutState.current === Layout.WIKIDOT}
  <h1 class="page-vote-header">
    {data.internationalization["wiki-page-vote"]}
  </h1>
  <div class="page-rate-widget-area">
    <div class="page-rate-widget-box">
      <span class="rate-points">
        {data.internationalization["wiki-page-vote.score"]}&nbsp;
        <span class="number prw54353">{voteRating}</span>
      </span><span class="rateup btn btn-default">
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a href="javascript:;" onclick={() => castVote(1)} type="button">+</a>
      </span><span class="ratedown btn btn-default">
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a href="javascript:;" onclick={() => castVote(-1)} type="button">-</a>
      </span><span class="cancel btn btn-default">
        <!-- svelte-ignore a11y_invalid_attribute -->
        <a href="javascript:;" onclick={cancelVote} type="button">x</a>
      </span>
    </div>
  </div>
  <p>
    <!-- svelte-ignore a11y_invalid_attribute -->
    <a
      href="javascript:;"
      onclick={() =>
        getVoteList().then(() => {
          showVoteList = true
        })}
    >
      {data.internationalization?.["wiki-page-vote.list"]}
    </a>
  </p>
  {#if showVoteList}
    <ul class="vote-list">
      {#each [...voteMap].sort((a, b) => b[0] - a[0]) as [userId, vote] (vote.page_vote_id)}
        <li class="vote-item" data-id={vote.page_vote_id} data-user-id={userId}>
          UT: User {vote.user_id}: {vote.value}
        </li>
      {/each}
    </ul>
  {/if}
{:else}
  <h2 class="page-vote-header">
    {data.internationalization["wiki-page-vote"]}
  </h2>
  <div class="vote-panel">
    <div class="action-row vote-action">
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
        <button
          class="vote-subbutton clickable"
          onclick={() => castVote(1)}
          type="button"
        >
          +1
        </button>
        <button
          class="vote-subbutton clickable"
          onclick={() => castVote(0)}
          type="button"
        >
          0
        </button>
        <button
          class="vote-subbutton clickable"
          onclick={() => castVote(-1)}
          type="button"
        >
          -1
        </button>
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
            UNTRANSLATED: User {vote.user_id}: {vote.value}
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
