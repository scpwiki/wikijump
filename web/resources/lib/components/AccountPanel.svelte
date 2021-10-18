<script lang="ts">
  import { t } from "@wikijump/api"
  import { Button, Tabview, Tab } from "@wikijump/components"
  import LoginForm from "./LoginForm.svelte"
  import RegisterForm from "./RegisterForm.svelte"

  /** Gets passed to this panel if it's been mounted inside of a dialog. */
  export let closeDialog: (() => void) | undefined
</script>

<div class="account-panel" tabindex="-1">
  <Tabview noborder contained>
    <Tab>
      <span class="account-panel-tab-button" slot="button">
        {$t("account_panel.LOGIN")}
      </span>
      <div class="account-panel-form">
        <LoginForm on:login={() => closeDialog?.()} />
      </div>
    </Tab>
    <Tab>
      <span class="account-panel-tab-button" slot="button">
        {$t("account_panel.REGISTER")}
      </span>
      <div class="account-panel-form">
        <RegisterForm on:register={() => closeDialog?.()} />
      </div>
    </Tab>
  </Tabview>

  <!-- placed down here so that it's the last thing that gets focused on -->
  {#if closeDialog}
    <div class="account-panel-close-dialog">
      <Button
        i="ion:close"
        tip={$t("account_panel.CLOSE_DIALOG")}
        size="1rem"
        baseline
        on:click={closeDialog}
      />
    </div>
  {/if}
</div>

<style lang="scss">
  @import "../../css/abstracts";

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
