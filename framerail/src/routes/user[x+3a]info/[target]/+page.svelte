<script lang="ts">
  import { resolve } from "$app/paths"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { showUserProfileActionAuthenticationGuard } from "$lib/user-profile-action-auth-guard.js"

  import type { PageProps } from "./$types"

  let { data }: PageProps = $props()
</script>

{#if data.view === "user_missing" && "error" in data}
  <div class="error-block">{data.error}</div>
{:else if "user" in data && "privateMessageControl" in data}
  <div class="col-md-9" data-user-id={data.user.userId}>
    {#if !data.user_session}
      <a
        href={resolve(data.privateMessageControl.authenticationPath, {})}
        data-action="write-private-message"
        data-target-user-id={data.user.userId}
      >
        {data.privateMessageControl.label}
      </a>

      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        class="btn btn-default btn-xs"
        href="javascript:;"
        role="button"
        data-action="add-to-contacts"
        data-target-user-id={data.user.userId}
        onclick={(event) => {
          event.preventDefault()
          showUserProfileActionAuthenticationGuard(errorPopupState)
        }}
      >
        <i class="icon-book" aria-hidden="true"></i>
        {data.contactControl.label}
      </a>

      <!-- svelte-ignore a11y_invalid_attribute -->
      <a
        class="btn btn-danger btn-xs pull-right"
        href="javascript:;"
        role="button"
        data-action="flag-user-as-abusive"
        data-target-user-id={data.user.userId}
        onclick={(event) => {
          event.preventDefault()
          showUserProfileActionAuthenticationGuard(errorPopupState)
        }}
      >
        {data.abuseReportControl.label}
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
