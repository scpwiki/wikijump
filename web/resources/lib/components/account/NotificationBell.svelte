<script lang="ts">
  import WikijumpAPI, { authed, route } from "@wikijump/api"
  import { Button } from "@wikijump/components"
  import Locale from "@wikijump/fluent"

  const t = Locale.makeComponentFormatter("notification-bell")

  let hasNotifications = false
  let count = ""

  // TODO: maybe a very slow poll?
  $: if ($authed) {
    WikijumpAPI.notificationGet()
      .then(({ notifications }) => {
        hasNotifications = notifications.length > 0
        count = formatCount(notifications.length)
      })
      .catch(err => {
        console.warn(err)
        hasNotifications = false
        count = ""
      })
  } else {
    hasNotifications = false
    count = ""
  }

  function formatCount(count: number) {
    if (count < 1000) return String(count)
    if (count < 10000) return `${Math.floor(count / 1000)}k`
    return "!"
  }
</script>

<!-- TODO: use our own bell icon in `ui.svg` -->

<span class="notification-bell" class:has-notifications={hasNotifications}>
  <Button
    i="octicon:bell-16"
    size="1em"
    baseline
    href={route("dashboard", { path: "notifications" })}
    tip={hasNotifications ? $t("#-status.unread") : $t("#-status.read")}
  />
  {#if hasNotifications}
    <span
      class="notification-bell-dot"
      class:is-2-wide={count.length === 2}
      class:is-3-wide={count.length === 3}
      aria-hidden="true"
    >
      <span class="notification-bell-count">
        {count}
      </span>
    </span>
  {/if}
</span>

<style global lang="scss">
  .notification-bell {
    position: relative;
    display: inline-block;
  }

  .notification-bell-dot {
    position: absolute;
    top: 5%;
    right: 5%;
    width: 0.8em;
    height: 0.8em;
    pointer-events: none;
    background: var(--col-danger);
    border-radius: 50%;

    &.is-2-wide,
    &.is-3-wide {
      .notification-bell-count {
        min-width: none;
        font-size: 0.625em;
      }
    }

    &.is-2-wide {
      right: 0;
      width: 1.1em;
      border-radius: 0.325em;
    }

    &.is-3-wide {
      right: 50%;
      width: 100%;
      border-radius: 0.25em;
      transform: translateX(50%);
    }
  }

  .notification-bell-count {
    position: absolute;
    top: 50%;
    left: 50%;
    min-width: 50%;
    font-family: var(--font-display);
    font-size: 0.75em;
    font-weight: bold;
    color: var(--col-white);
    text-align: center;
    white-space: nowrap;
    transform: translate(-50%, -50%);
  }
</style>
