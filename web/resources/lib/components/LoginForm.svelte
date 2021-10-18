<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button, TextInput, Toggle, toast } from "@wikijump/components"
  import { createEventDispatcher } from "svelte"
  import { inputsValid } from "../util"

  const dispatch = createEventDispatcher()

  let busy = false

  let inputLogin: HTMLInputElement
  let inputPassword: HTMLInputElement
  let remember = false

  async function login() {
    if (inputsValid(inputLogin, inputPassword)) {
      busy = true
      try {
        const login = inputLogin.value
        const password = inputPassword.value
        await WikijumpAPI.authLogin({ login, password, remember })
        toast("success", $t("account_panel.toasts.LOGGED_IN"))
        dispatch("login")
      } catch {
        toast("danger", $t("account_panel.toasts.LOGIN_FAILED"))
      }
      busy = false
    } else {
      toast("danger", $t("account_panel.toasts.INVALID_INPUT"))
    }
  }
</script>

<div class="login-form">
  <form>
    <TextInput
      bind:input={inputLogin}
      on:enter={() => inputPassword.focus()}
      label={$t("account_panel.SPECIFIER")}
      placeholder={$t("account_panel.SPECIFIER_PLACEHOLDER")}
      required
      disabled={busy}
      autocomplete="username"
      minlength="1"
    />

    <TextInput
      bind:input={inputPassword}
      on:enter={() => login()}
      label={$t("account_panel.PASSWORD")}
      type="password"
      placeholder={$t("account_panel.PASSWORD_PLACEHOLDER")}
      required
      disabled={busy}
      autocomplete="current-password"
      minLength="1"
    />
  </form>

  <div class="login-form-options">
    <Toggle bind:toggled={remember}>{$t("account_panel.REMEMBER_ME")}</Toggle>
    <!-- TODO: forgot password -->
    <a class="login-form-forgot" href="/forgot">{$t("account_panel.FORGOT_PASSWORD")}</a>
  </div>

  <div class="login-form-submit">
    <Button on:click={login} disabled={busy} wide primary>
      {$t("account_panel.LOGIN")}
    </Button>
  </div>
</div>

<style lang="scss">
  @import "../../css/abstracts";

  .login-form-options {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0.5rem 0;
  }

  .login-form-forgot {
    font-size: 0.825rem;
    color: var(--col-hint);

    @include hover {
      text-decoration: underline;
    }
  }
</style>
