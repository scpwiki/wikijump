<script lang="ts">
  import WikijumpAPI, { authed } from "@wikijump/api"
  import Locale from "@wikijump/fluent"
  import { Button } from "@wikijump/components"

  const t = Locale.loadWithObservableFormatter("notification-bell")

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
    href="/account/notifications"
    tip={hasNotifications
      ? $t("notification-bell-status.unread")
      : $t("notification-bell-status.read")}
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

<style lang="scss">
  .notification-bell {
    position: relative;
    display: inline-block;
  }

  .notification-bell-dot {
    position: absolute;
    pointer-events: none;
    top: 5%;
    right: 5%;
    width: 0.8em;
    height: 0.8em;
    background: var(--col-danger);
    border-radius: 50%;

    &.is-2-wide,
    &.is-3-wide {
      .notification-bell-count {
        font-size: 0.625em;
        min-width: none;
      }
    }

    &.is-2-wide {
      width: 1.1em;
      right: 0;
      border-radius: 0.325em;
    }

    &.is-3-wide {
      width: 100%;
      right: 50%;
      transform: translateX(50%);
      border-radius: 0.25em;
    }
  }

  .notification-bell-count {
    position: absolute;
    top: 50%;
    left: 50%;
    min-width: 50%;
    text-align: center;
    transform: translate(-50%, -50%);
    color: var(--col-white);
    font-size: 0.75em;
    white-space: nowrap;
    font-family: var(--font-display);
    font-weight: bold;
  }
</style>
