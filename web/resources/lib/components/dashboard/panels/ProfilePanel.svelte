<script lang="ts">
  import WikijumpAPI, { type UserProfile } from "@wikijump/api"
  import { Spinny } from "@wikijump/components"

  const profilePromise = WikijumpAPI.userClientGet({
    detail: "profile"
  }) as Promise<UserProfile>
</script>

{#await profilePromise}
  <Spinny />
{:then profile}
  <!-- profile overview -->
{/await}

<style global lang="scss">
</style>
