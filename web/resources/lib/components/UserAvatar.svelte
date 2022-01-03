<script lang="ts">
  import WikijumpAPI, { asset } from "@wikijump/api"
  import Skeleton from "./Skeleton.svelte"

  export let user: number | string = ""

  export let size = "1em"

  $: request = user
    ? typeof user === "number"
      ? WikijumpAPI.try("userGetAvatar", "id", user)
      : WikijumpAPI.try("userGetAvatar", "slug", user)
    : WikijumpAPI.try("userClientGetAvatar")
</script>

{#await request}
  <div style="width: {size}; height: {size};" class="avatar is-loading">
    <Skeleton type="block" fill />
  </div>
{:then res}
  <img
    style="width: {size}; height: {size};"
    class="avatar"
    src={res?.avatar ?? asset("BAD_AVATAR").raw}
    alt=""
  />
{/await}

<style global lang="scss">
  .avatar {
    overflow: hidden;
    // set background so that if the image doesn't load right away,
    // it'll still look like a placeholder
    background: var(--col-background-dim);
    border: solid 0.125rem var(--col-black);
    border-radius: 10%;
  }
</style>
