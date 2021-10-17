<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button, Card, TextInput, Toggle, toast } from "@wikijump/components"

  /** Gets passed to this panel if it's been mounted inside of a dialog. */
  export let closeDialog: (() => void) | undefined

  let busy = false

  let inputLogin: HTMLInputElement
  let inputPassword: HTMLInputElement
  let remember = false

  async function login() {
    if (!inputLogin || !inputPassword) return
    if (inputLogin.validity.valid && inputPassword.validity.valid) {
      busy = true
      try {
        const login = inputLogin.value
        const password = inputPassword.value
        await WikijumpAPI.authLogin({ login, password, remember })
        toast("success", $t("login.toasts.LOGGED_IN"))
      } catch {
        toast("danger", $t("login.toasts.FAILED"))
      }
      busy = false
    }
  }
</script>

<div class="login-panel">
  <Card title={$t("login.LOGIN")} width="25rem">
    <form>
      <TextInput
        bind:input={inputLogin}
        on:enter={() => inputPassword.focus()}
        label={$t("login.SPECIFIER")}
        placeholder={$t("login.SPECIFIER_PLACEHOLDER")}
        required
        disabled={busy}
        autocomplete="username"
        minlength="1"
      />

      <TextInput
        bind:input={inputPassword}
        on:enter={() => login()}
        label={$t("login.PASSWORD")}
        type="password"
        placeholder={$t("login.PASSWORD_PLACEHOLDER")}
        required
        disabled={busy}
        autocomplete="current-password"
        minLength="1"
      />
    </form>

    <div class="login-panel-options">
      <Toggle bind:toggled={remember}>{$t("login.REMEMBER")}</Toggle>
      <!-- TODO: forgot password -->
      <a class="login-panel-forgot" href="/forgot">{$t("login.FORGOT")}</a>
    </div>

    <div class="login-panel-submit">
      <Button on:click={login} disabled={busy} wide primary>
        {$t("login.LOGIN")}
      </Button>
    </div>

    <!-- placed down here so that it's the last thing that gets focused on -->
    {#if closeDialog}
      <div class="login-panel-close-dialog">
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
  @import "../../css/abstracts";

  .login-panel-close-dialog {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
  }

  .login-panel-options {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .login-panel-forgot {
    font-size: 0.825rem;
    color: var(--col-hint);

    @include hover {
      text-decoration: underline;
    }
  }
</style>
