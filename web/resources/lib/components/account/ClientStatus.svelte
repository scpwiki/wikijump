<script lang="ts">
  import WikijumpAPI, { authed, identity, route } from "@wikijump/api"
  import { Button, Card, DetailsMenu, Icon, Sprite } from "@wikijump/components"
  import { matchBreakpoint, toast } from "@wikijump/components/lib"
  import { focusGroup } from "@wikijump/dom"
  import { format as t } from "@wikijump/fluent"
  import { AuthModal } from "../auth/auth-modal"
  import UserInfo from "../UserInfo.svelte"
  import NotificationBell from "./NotificationBell.svelte"

  /** If true, the status will be rendered with a background. */
  export let background = true

  async function logout() {
    if (!$authed) return
    await WikijumpAPI.authLogout()
    toast("success", t("logout.toast"))
  }
</script>

<!-- TODO: persist auth state across page -->

{#if !$authed}
  <div class="client-status" class:has-background={background}>
    <Button baseline compact on:click={() => AuthModal.toggle(true)}>
      <Icon i="ic:round-login" size="1.25rem" />
      {t("login")}
    </Button>

    <div class="client-status-sep" />

    <Button baseline compact on:click={() => AuthModal.toggle(true)}>
      <Icon i="ic:round-person-add" size="1.25rem" />
      {t("create-account")}
    </Button>
  </div>
{:else if $identity}
  <div class="client-status is-authed" class:has-background={background}>
    <NotificationBell />

    <div class="client-status-sep" />

    <DetailsMenu placement="bottom-end" hoverable let:open>
      <Button slot="button" tabindex="-1" active={open} baseline compact>
        <UserInfo nolink nousername={$matchBreakpoint("<=small")} />
        <Sprite i="wj-downarrow" size="0.55rem" margin="0 0 0 0.15rem" />
      </Button>

      <Card>
        <div class="client-status-menu" use:focusGroup={"vertical"}>
          <!-- TODO: proper links -->
          <Button href={route("account")} tabindex="-1" baseline compact>
            {t("account")}
          </Button>

          <Button href={route("account.profile")} tabindex="-1" baseline compact>
            {t("profile")}
          </Button>

          <Button href={route("account.messages")} tabindex="-1" baseline compact>
            {t("messages")}
          </Button>

          <hr />

          <Button href={route("docs")} tabindex="-1" baseline compact>
            {t("help")}
          </Button>

          <Button href={route("account.settings")} tabindex="-1" baseline compact>
            {t("settings")}
          </Button>

          <hr />

          <Button on:click={logout} tabindex="-1" baseline compact>
            {t("logout")}
          </Button>
        </div>
      </Card>
    </DetailsMenu>
  </div>
{/if}

<style lang="scss">
  @import "../../../css/abstracts";

  @keyframes client-status-reveal {
    0% {
      opacity: 0;
    }
    100% {
      opacity: 1;
    }
  }

  .client-status {
    display: flex;
    align-items: center;
    justify-content: space-evenly;
    font-size: 0.925rem;
    animation: client-status-reveal 100ms backwards ease-out;
    white-space: nowrap;

    &.has-background {
      padding: 0.325rem 0.625rem;
      background: var(--col-background);
      border: 0.075rem solid var(--col-border);
      border-radius: 0.325rem;
      @include shadow(4);
    }
  }

  .client-status-sep {
    width: 0.075rem;
    height: 0.75rem;
    margin: 0 0.5rem;
    background: var(--col-border);

    @include media("<=small") {
      margin: 0 0.25rem;
    }
  }

  .client-status-menu {
    display: flex;
    flex-direction: column;
    min-width: 7rem;
    font-size: 0.875rem;

    > hr {
      margin: 0.5rem 0;
    }
  }
</style>
