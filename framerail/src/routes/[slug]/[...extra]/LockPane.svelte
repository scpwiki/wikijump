<script lang="ts">
  import { deserialize } from "$app/forms"
  import { errorPopupState, pageLayoutState } from "$lib/stores.svelte"
  import { Layout, PageLockType, PagePane } from "$lib/types"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageLockModel } from "$lib/server/deepwell/page"
  import type { PageProps } from "./$types"

  let { pagePaneState = $bindable(), data }: PageProps & { pagePaneState: PagePane } =
    $props()

  const { form, enhance } = superForm(
    untrack(() => data.forms.pageLockForm),
    {
      dataType: "json",
      onSubmit: async ({ jsonData }) => {
        jsonData({
          ...$form,
          expiresAt: $form.expiresAt ? new Date($form.expiresAt).toISOString() : undefined
        })
      },
      onResult: async ({ result }) => {
        if (result.type === "success") {
          await fetchLockHistory()
        } else if (result.type === "failure" && result.data) {
          errorPopupState.current = {
            state: true,
            message: result.data.message,
            data: result.data.data
          }
        }
      }
    }
  )

  $form.lockType = PageLockType.PermissionOnly
  $form.reason = ""
  $form.overrideExisting = false

  let lockHistory = $state<PageLockModel[]>([])

  async function removeLock() {
    const res = await fetch("?/lockRemove", {
      method: "POST",
      body: JSON.stringify({})
    }).then((res) => res.text())

    const result = deserialize<
      Record<string, never>,
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else {
      await fetchLockHistory()
    }
  }

  /** Helper function to get lock status from data and subsequent lock */
  function getLockStatus(
    lock: PageLockModel,
    index: number
  ): keyof typeof data.internationalization {
    if (lock.deleted_at) {
      // Heuristic check to determine if a lock was overridden:
      // If next lock was created within a short time of last lock being deleted
      if (index > 0) {
        const newerLock = lockHistory[index - 1]
        const deletedAt = new Date(lock.deleted_at).getTime()
        const newerCreatedAt = new Date(newerLock.created_at).getTime()
        if (Math.abs(newerCreatedAt - deletedAt) < 5000) {
          return "wiki-page-lock.history-overridden"
        }
      }
      return "wiki-page-lock.history-removed"
    }
    if (lock.expires_at && new Date(lock.expires_at) < new Date()) {
      return "wiki-page-lock.history-expired"
    }
    return "wiki-page-lock.history-active"
  }

  async function fetchLockHistory() {
    const res = await fetch("?/lockHistory", {
      method: "POST",
      body: JSON.stringify({})
    }).then((res) => res.text())

    const result = deserialize<
      { res: PageLockModel[] },
      { message: string; code: string; data: Record<string, unknown> }
    >(res)

    if (result.type === "failure" && result.data?.message) {
      errorPopupState.current = {
        state: true,
        message: result.data.message,
        data: result.data
      }
    } else if (result.type === "success" && result.data?.res) {
      lockHistory = result.data.res
    }
  }

  $effect(() => {
    fetchLockHistory()
  })
</script>

{#if pageLayoutState.current === Layout.WIKIDOT}
  <h1 class="page-lock-header">{data.internationalization?.["wiki-page-lock"]}</h1>
{:else}
  <h2 class="page-lock-header">{data.internationalization?.["wiki-page-lock"]}</h2>
{/if}

<form id="page-lock" class="page-lock" action="?/lockCreate" method="POST" use:enhance>
  <div class="page-lock-type-group">
    <label class="page-lock-field-label" for="page-lock-type-permission">
      <input
        id="page-lock-type-permission"
        name="lockType"
        type="radio"
        value={PageLockType.PermissionOnly}
        bind:group={$form.lockType}
      />
      {data.internationalization?.["wiki-page-lock.permission-only-text"]}
    </label>
    <label class="page-lock-field-label" for="page-lock-type-author">
      <input
        id="page-lock-type-author"
        name="lockType"
        type="radio"
        value={PageLockType.AuthorOrPermissionOnly}
        bind:group={$form.lockType}
      />
      {data.internationalization?.["wiki-page-lock.author-or-permission-only-text"]}
    </label>
  </div>

  <textarea
    name="reason"
    class="page-lock-reason"
    placeholder={data.internationalization?.["wiki-page-lock.reason"]}
    bind:value={$form.reason}
  ></textarea>

  <label class="page-lock-field-label" for="page-lock-expires-at">
    {data.internationalization?.["wiki-page-lock.expires-at"]}
  </label>
  <input
    id="page-lock-expires-at"
    name="expiresAt"
    type="datetime-local"
    bind:value={$form.expiresAt}
  />

  <label class="page-lock-field-label">
    <input
      name="overrideExisting"
      type="checkbox"
      bind:checked={$form.overrideExisting}
    />
    {data.internationalization?.["wiki-page-lock.override"]}
  </label>

  {#if pageLayoutState.current === Layout.WIKIDOT}
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
        value={data.internationalization?.["wiki-page-lock"]}
      />
    </div>
  {:else}
    <div class="action-row page-lock-actions">
      <button
        class="action-button page-lock-button button-cancel clickable"
        onclick={() => (pagePaneState = PagePane.None)}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button class="action-button page-lock-button button-lock clickable" type="submit">
        {data.internationalization?.["wiki-page-lock"]}
      </button>
    </div>
  {/if}
</form>

<h3 class="page-lock-history-header">
  {data.internationalization?.["wiki-page-lock.history"]}
</h3>

{#if lockHistory.length === 0}
  <p class="page-lock-history-none">
    {data.internationalization?.["wiki-page-lock.history-none"]}
  </p>
{:else}
  <div class="lock-history-list">
    <div class="lock-history-row lock-history-header-row">
      <div class="lock-history-attr">
        {data.internationalization?.["wiki-page-lock.history-type"]}
      </div>
      <div class="lock-history-attr">
        {data.internationalization?.["wiki-page-lock.history-user"]}
      </div>
      <div class="lock-history-attr">
        {data.internationalization?.["wiki-page-lock.history-status"]}
      </div>
      <div class="lock-history-attr">
        {data.internationalization?.["wiki-page-lock.history-created"]}
      </div>
      <div class="lock-history-attr">
        {data.internationalization?.["wiki-page-lock.history-expires"]}
      </div>
      <div class="lock-history-attr">
        {data.internationalization?.["wiki-page-lock.history-reason"]}
      </div>
      <div class="lock-history-attr"></div>
    </div>
    {#each lockHistory as lock, index (lock.page_lock_id)}
      <div class="lock-history-row">
        <div class="lock-history-attr">
          {data.internationalization?.[
            `wiki-page-lock.${lock.lock_type}` as keyof typeof data.internationalization
          ]}
        </div>
        <div class="lock-history-attr">{lock.user_id}</div>
        <div class="lock-history-attr">
          {data.internationalization?.[getLockStatus(lock, index)]}
        </div>
        <div class="lock-history-attr">
          {new Date(lock.created_at).toLocaleString()}
        </div>
        <div class="lock-history-attr">
          {lock.expires_at ? new Date(lock.expires_at).toLocaleString() : "—"}
        </div>
        <div class="lock-history-attr">{lock.reason || "—"}</div>
        <div class="lock-history-attr">
          {#if getLockStatus(lock, index) === "wiki-page-lock.history-active"}
            {#if pageLayoutState.current === Layout.WIKIDOT}
              <button
                class="btn-remove-lock-text clickable"
                onclick={removeLock}
                type="button"
              >
                {data.internationalization?.["wiki-page-lock.remove"]}
              </button>
            {:else}
              <button
                class="btn-remove-lock clickable"
                onclick={removeLock}
                type="button"
              >
                {data.internationalization?.["wiki-page-lock.remove"]}
              </button>
            {/if}
          {/if}
        </div>
      </div>
    {/each}
  </div>
{/if}

<style lang="scss">
  .page-lock {
    display: flex;
    flex-direction: column;
    gap: 1em;
    align-items: stretch;
    width: 100%;
    padding: 0 0 2em;
  }

  .page-lock-type-group {
    display: flex;
    flex-direction: column;
    gap: 0.5em;
    margin: 0;
  }

  textarea.page-lock-reason {
    min-height: 5em;
    resize: vertical;
    border-radius: 0.25em;
  }

  .page-lock-field-label {
    cursor: pointer;
  }

  input[type="datetime-local"] {
    padding: 0.5em 1em;
    color: var(--text);
    background-color: var(--background);
    border: 1px solid var(--border);
    border-radius: 0.5em;
  }

  .lock-history-list {
    display: table;
    width: 100%;

    .lock-history-header-row,
    .lock-history-row {
      display: table-row;

      .lock-history-attr {
        display: table-cell;
        padding: 0.3em 0.5em;
      }
    }

    .lock-history-header-row .lock-history-attr {
      font-weight: 600;
    }
  }

  .btn-remove-lock-text {
    padding: 0;
    color: var(--danger, #c0392b);
    background: none;
    border: none;
    cursor: pointer;
    text-decoration: none;
    font-size: inherit;

    &:hover {
      text-decoration: underline;
    }
  }

  .btn-remove-lock {
    padding: 0.2em 0.6em;
    color: var(--danger, #c0392b);
    background: none;
    border: 1px solid var(--danger, #c0392b);
    border-radius: 0.25em;
    cursor: pointer;

    &:hover {
      color: var(--background);
      background-color: var(--danger, #c0392b);
    }
  }
</style>
