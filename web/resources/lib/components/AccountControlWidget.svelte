<script lang="ts">
  import WikijumpAPI, { t, authed } from "@wikijump/api"
  import { focusGroup } from "@wikijump/dom"
  import { toast, Button, Card, DetailsMenu } from "@wikijump/components"
  import UserInfo from "./UserInfo.svelte"
  import { AccountModal } from "../account-panel"

  async function logout() {
    if (!$authed) return
    await WikijumpAPI.authLogout()
    toast("success", $t("auth.toasts.LOGGED_OUT"))
  }
</script>

<!-- TODO: notifications -->
<!-- TODO: tiny dropdown arrow -->
<!-- TODO: persist auth state across page -->

<div class="account-control dark" class:is-authed={$authed}>
  {#if !$authed}
    <Button baseline compact on:click={() => AccountModal.toggle(true)}>
      {$t("auth.LOGIN")}
    </Button>

    <div class="account-control-sep" />

    <Button baseline compact on:click={() => AccountModal.toggle(true)}>
      {$t("auth.CREATE_ACCOUNT")}
    </Button>
  {:else}
    <DetailsMenu placement="bottom" hoverable let:open>
      <Button slot="button" tabindex="-1" active={open} baseline compact>
        <UserInfo nolink />
      </Button>

      <Card>
        <div class="account-control-menu" use:focusGroup={"vertical"}>
          <!-- TODO: proper links -->
          <Button href="/account" tabindex="-1" baseline compact>
            {$t("components.account_control_widget.ACCOUNT")}
          </Button>

          <Button href="/user:info" tabindex="-1" baseline compact>
            {$t("components.account_control_widget.PROFILE")}
          </Button>

          <Button href="account/messages" tabindex="-1" baseline compact>
            {$t("components.account_control_widget.MESSAGES")}
          </Button>

          <hr />

          <Button href="/docs" tabindex="-1" baseline compact>
            {$t("components.account_control_widget.HELP")}
          </Button>

          <Button href="/account/settings" tabindex="-1" baseline compact>
            {$t("components.account_control_widget.SETTINGS")}
          </Button>

          <hr />

          <Button on:click={logout} tabindex="-1" baseline compact>
            {$t("auth.LOGOUT")}
          </Button>
        </div>
      </Card>
    </DetailsMenu>
  {/if}
</div>

<style lang="scss">
  @import "../../css/abstracts";

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
    padding: 0.325rem 0.675rem;
    border-radius: 0.325rem;
    font-size: 0.875rem;
    @include shadow(4);
    // slight delay on animations to allow for the auth state to be set
    animation: account-control-reveal 100ms 250ms backwards ease-out;
  }

  .account-control-sep {
    width: 0.075rem;
    height: 0.75rem;
    background: var(--col-border);
    margin: 0 0.5em;
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
