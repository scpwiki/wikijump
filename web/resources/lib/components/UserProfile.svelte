<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { format as t } from "@wikijump/fluent"
  import { Spinny } from "@wikijump/components"
  import Error from "./Error.svelte"
  import { fade } from "svelte/transition"
  import UserAvatar from "./UserAvatar.svelte"

  export let user: number | string = ""

  $: request = user
    ? WikijumpAPI.getUser(user, "profile")
    : WikijumpAPI.getClient("profile")
</script>

{#await request}
  <Spinny />
{:then profile}
  {#if profile}
    <div class="user-profile" in:fade={{ duration: 150 }}>
      <div class="user-profile-header">
        <UserAvatar {user} size="4rem" />
        {#if profile.realname}
          <h1>
            <span>{profile.username}</span>
            <small class="user-profile-realname">{profile.realname}</small>
          </h1>
        {:else}
          <h1>{profile.username}</h1>
        {/if}
      </div>
      {#if profile.about}
        <div class="user-profile-about">
          <wj-body>
            {profile.about}
          </wj-body>
        </div>
      {/if}
    </div>
  {:else}
    <Error>
      <p>{t("error-404.user")}</p>
    </Error>
  {/if}
{/await}

<style global lang="scss">
  .user-profile-header {
    display: flex;
    column-gap: 1rem;
    align-items: flex-end;
    padding-bottom: 1rem;
    margin-bottom: 1rem;
    border-bottom: solid 0.125rem var(--col-border);

    > h1 {
      display: flex;
      flex-direction: column;
      margin-bottom: 0.25rem;
      line-height: 1;

      > small {
        font-size: 0.825rem;
        color: var(--col-text-subtle);
      }
    }
  }
</style>
