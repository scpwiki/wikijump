<script lang="ts">
  import WikijumpAPI, { type UserProfile } from "@wikijump/api"
  import { Spinny } from "@wikijump/components"
  import UserAvatar from "../../UserAvatar.svelte"

  const profilePromise = WikijumpAPI.userClientGet({
    detail: "profile"
  }) as Promise<UserProfile>
</script>

{#await profilePromise}
  <Spinny />
{:then profile}
  <div class="dashboard-profile-header">
    <UserAvatar size="6rem" />
    <h1>{profile.username}</h1>
  </div>
{/await}

<style global lang="scss">
  .dashboard-profile-header {
    display: flex;
    column-gap: 1rem;
    align-items: flex-end;
    padding-bottom: 1rem;
    margin-bottom: 2rem;
    border-bottom: solid 0.125rem var(--col-border);
  }
</style>
