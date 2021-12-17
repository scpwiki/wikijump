<script lang="ts">
  import WikijumpAPI, { authed } from "@wikijump/api"
  import { Sprite } from "@wikijump/components"
  import type { UserIdentity } from "@wikijump/api"

  export let user: null | string | number = null

  export let avatar = true

  let identity: UserIdentity | null = null

  $: {
    identity = null
    if (user === null && $authed) {
      WikijumpAPI.userClientGet({ avatars: avatar }).then(data => {
        identity = data
      })
    } else if (user) {
      const type = typeof user === "number" ? "id" : "name"
      WikijumpAPI.userGet(type, user, { avatars: avatar }).then(data => {
        identity = data
      })
    }
  }

  function avatarUrl() {
    if (!identity || !avatar) return ""
    return `data:image/png;base64,${identity.tinyavatar}`
  }
</script>

{#if identity}
  <span class="wj-user-info">
    <a href="">
      {#if avatar}
        <span class="wj-karma" data-karma={identity.karma}>
          <Sprite i="wj-karma" />
        </span>

        <img class="wj-user-info-avatar" src={avatarUrl()} />
      {/if}

      <span class="wj-user-info-name">
        {identity.username}
      </span>
    </a>
  </span>
{/if}
