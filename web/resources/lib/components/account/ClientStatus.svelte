<script lang="ts">
  import WikijumpAPI, { authed, identity } from "@wikijump/api"
  import Locale from "@wikijump/fluent"
  import { focusGroup } from "@wikijump/dom"
  import { toast, Sprite, Button, Card, DetailsMenu } from "@wikijump/components"
  import UserInfo from "../UserInfo.svelte"
  import NotificationBell from "./NotificationBell.svelte"
  import { AuthModal } from "../auth/auth-modal"

  const t = Locale.loadWithObservableFormatter("client-status")

  async function logout() {
    if (!$authed) return
    await WikijumpAPI.authLogout()
    toast("success", $t("auth.toasts.LOGGED_OUT"))
  }
</script>

<!-- TODO: persist auth state across page -->

{#if !$authed}
  <div class="account-control dark">
    <Button baseline compact on:click={() => AuthModal.toggle(true)}>
      {$t("login")}
    </Button>

    <div class="account-control-sep" />

    <Button baseline compact on:click={() => AuthModal.toggle(true)}>
      {$t("create-account")}
    </Button>
  </div>
{:else if $identity}
  <div class="account-control dark is-authed">
    <NotificationBell />

    <div class="account-control-sep" />

    <DetailsMenu placement="bottom-end" hoverable let:open>
      <Button slot="button" tabindex="-1" active={open} baseline compact>
        <UserInfo nolink />
        <Sprite i="wj-downarrow" size="0.55rem" margin="0 0 0 0.15rem" />
      </Button>

      <Card>
        <div class="account-control-menu" use:focusGroup={"vertical"}>
          <!-- TODO: proper links -->
          <Button href="/account" tabindex="-1" baseline compact>
            {$t("client-status-menu.account")}
          </Button>

          <Button href="/user:info" tabindex="-1" baseline compact>
            {$t("client-status-menu.profile")}
          </Button>

          <Button href="account/messages" tabindex="-1" baseline compact>
            {$t("client-status-menu.messages")}
          </Button>

          <hr />

          <Button href="/docs" tabindex="-1" baseline compact>
            {$t("client-status-menu.help")}
          </Button>

          <Button href="/account/settings" tabindex="-1" baseline compact>
            {$t("client-status-menu.settings")}
          </Button>

          <hr />

          <Button on:click={logout} tabindex="-1" baseline compact>
            {$t("logout")}
          </Button>
        </div>
      </Card>
    </DetailsMenu>
  </div>
{/if}

<style lang="scss">
  @import "../../../css/abstracts";

  @keyframes account-control-reveal {
    0% {
      opacity: 0;
    }
    100% {
      opacity: 1;
    }
  }

  .account-control {
    display: flex;
    align-items: center;
    justify-content: space-evenly;
    background: var(--col-background);
    border: 0.075rem solid var(--col-border);
    padding: 0.325rem 0.625rem;
    border-radius: 0.325rem;
    font-size: 0.925rem;
    @include shadow(4);
    animation: account-control-reveal 100ms backwards ease-out;
  }

  .account-control-sep {
    width: 0.075rem;
    height: 0.75rem;
    background: var(--col-border);
    margin: 0 0.5rem;
  }

  .account-control-menu {
    display: flex;
    flex-direction: column;
    min-width: 7rem;
    font-size: 0.875rem;

    > hr {
      margin: 0.5rem 0;
    }
  }
</style>
