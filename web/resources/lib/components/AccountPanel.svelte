<script lang="ts">
  import { t } from "@wikijump/api"
  import { Button, Card } from "@wikijump/components"
  import LoginForm from "./LoginForm.svelte"

  /** Gets passed to this panel if it's been mounted inside of a dialog. */
  export let closeDialog: (() => void) | undefined
</script>

<div class="account-panel">
  <Card title={$t("login.LOGIN")} width="25rem">
    <LoginForm />

    <!-- placed down here so that it's the last thing that gets focused on -->
    {#if closeDialog}
      <div class="account-panel-close-dialog">
        <Button
          i="ion:close"
          tip={$t("login.CLOSE")}
          size="1.5rem"
          baseline
          on:click={closeDialog}
        />
      </div>
    {/if}
  </Card>
</div>

<style lang="scss">
  .account-panel-close-dialog {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
  }
</style>
