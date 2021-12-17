<script lang="ts">
  import WikijumpAPI, { authed } from "@wikijump/api"
  import { Sprite } from "@wikijump/components"
  import type { UserIdentity } from "@wikijump/api"

  export let user: null | string | number = null

  export let noavatar = false

  export let nokarma = false

  export let nousername = false

  let identity: UserIdentity | null = null

  $: {
    identity = null
    if (user === null && $authed) {
      WikijumpAPI.userClientGet({ avatars: !noavatar }).then(data => {
        identity = data
      })
    } else if (user) {
      const type = typeof user === "number" ? "id" : "name"
      WikijumpAPI.userGet(type, user, { avatars: !noavatar }).then(data => {
        identity = data
      })
    }
  }

  function avatarUrl() {
    if (!identity || !noavatar) return ""
    return `data:image/png;base64,${identity.tinyavatar}`
  }
</script>

{#if identity}
  <span class="wj-user-info">
    <a class="wj-user-info-link" href="">
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
  </span>
{/if}
