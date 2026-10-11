<script lang="ts">
  import { page } from "$app/state"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { showUserContactAuthenticationGuard } from "$lib/user-contact-auth-guard.js"

  import type { PageProps } from "./$types"

  let { data }: PageProps = $props()
</script>

{#if data.view === "user_missing" && "error" in data}
  <div class="error-block">{data.error}</div>
{:else if "user" in data && "privateMessageControl" in data}
  <div class="col-md-9" data-user-id={data.user.userId}>
    <span data-redacted-control="private-message">
      {data.privateMessageControl.label}
    </span>

    {#if !page.data.user_session}
      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        class="btn btn-default btn-xs"
        href="javascript:;"
        role="button"
        data-action="add-to-contacts"
        data-target-user-id={data.user.userId}
        onclick={(event) => {
          event.preventDefault()
          showUserContactAuthenticationGuard(errorPopupState)
        }}
      >
        <i class="icon-book" aria-hidden="true"></i>
        Add to contacts
      </a>
    {/if}

    <h1 class="profile-title">
      {#if data.user.avatar}
        <img alt="" src={data.user.avatar} />
      {/if}
      {data.user.name}
    </h1>

    <div id="user-info-area">
      <div class="profile-box">
        <dl class="dl-horizontal">
          <dt>Wikidot user since:</dt>
          <dd><span class="odate">{data.user.createdAt}</span></dd>

          {#if data.user.accountType}
            <dt>Account type:</dt>
            <dd>
              {data.user.accountType === "regular" ? "free" : data.user.accountType}
            </dd>
          {/if}

          {#if data.user.karmaLevel}
            <dt>Karma level:</dt>
            <dd>{data.user.karmaLevel}</dd>
          {/if}
        </dl>
      </div>
    </div>
  </div>
{/if}
