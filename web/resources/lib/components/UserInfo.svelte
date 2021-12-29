<script lang="ts">
  import WikijumpAPI, {
    identity as currentIdentity,
    route,
    type UserIdentity
  } from "@wikijump/api"
  import { Sprite } from "@wikijump/components"

  export let user: null | string | number = null

  export let noavatar = false

  export let nokarma = false

  export let nousername = false

  export let nolink = false

  let identity: UserIdentity | null = null

  $: {
    identity = null
    // default is current user
    if (user === null) identity = $currentIdentity
    // fetch user from API
    else {
      const type = typeof user === "number" ? "id" : "name"
      WikijumpAPI.userGet(type, user, { avatars: !noavatar }).then(data => {
        identity = data
      })
    }
  }

  function avatarUrl() {
    if (!identity || noavatar) return ""
    return `data:image/png;base64,${identity.tinyavatar}`
  }
</script>

{#if identity}
  <span class="wj-user-info">
    {#if !nolink}
      <a
        class="wj-user-info-link"
        href={route("user.profile", { user: identity.username })}
      >
        {#if !noavatar}
          {#if !nokarma}
            <span class="wj-karma" data-karma={identity.karma}>
              <Sprite i="wj-karma" />
            </span>
          {/if}

          <img class="wj-user-info-avatar" src={avatarUrl()} />
        {/if}

        {#if !nousername}
          <span class="wj-user-info-name">
            {identity.username}
          </span>
        {/if}
      </a>
    {:else}
      <span class="wj-user-info-link">
        {#if !noavatar}
          {#if !nokarma}
            <span class="wj-karma" data-karma={identity.karma}>
              <Sprite i="wj-karma" />
            </span>
          {/if}

          <img class="wj-user-info-avatar" src={avatarUrl()} />
        {/if}

        {#if !nousername}
          <span class="wj-user-info-name">
            {identity.username}
          </span>
        {/if}
      </span>
    {/if}
  </span>
{/if}
