<!--
  @component Tabbed account login, registration panel.
-->
<script lang="ts">
  import { Button, Tab, Tabview } from "@wikijump/components"
  import { toast } from "@wikijump/components/lib"
  import { format as t } from "@wikijump/fluent"
  import LoginForm from "./LoginForm.svelte"
  import RegisterForm from "./RegisterForm.svelte"

  /** Gets passed to this panel if it's been mounted inside of a dialog. */
  export let closeDialog: (() => void) | undefined

  function login() {
    closeDialog?.()
    toast("success", t("login.toast"))
  }

  function register() {
    closeDialog?.()
    toast("success", t("register.toast"))
  }
</script>

<div class="account-panel" tabindex="-1">
  <Tabview noborder contained>
    <Tab>
      <span class="account-panel-tab-button" slot="button">
        {t("login")}
      </span>
      <div class="account-panel-form">
        <LoginForm on:login={login} />
      </div>
    </Tab>
    <Tab>
      <span class="account-panel-tab-button" slot="button">
        {t("register")}
      </span>
      <div class="account-panel-form">
        <RegisterForm on:register={register} />
      </div>
    </Tab>
  </Tabview>

  <!-- placed down here so that it's the last thing that gets focused on -->
  {#if closeDialog}
    <div class="account-panel-close-dialog">
      <Button i="wj-close" tip={t("close")} size="1rem" baseline on:click={closeDialog} />
    </div>
  {/if}
</div>

<style global lang="scss">
  .account-panel {
    width: 30rem;
    max-width: 90vw;
    font-size: 1rem;
    background: var(--col-background);
    border: solid 0.075rem var(--col-border);
    border-radius: 0.5rem;
    contain: content;
    @include shadow(4);
  }

  .account-panel-form {
    padding: 2rem;
    padding-top: 0;
    padding-bottom: 1rem;
  }

  .account-panel-close-dialog {
    position: absolute;
    top: 0;
    right: 0.25rem;
  }
</style>
