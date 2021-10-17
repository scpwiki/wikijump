<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button, TextInput, Toggle, toast } from "@wikijump/components"

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

<div class="login-form">
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

  <div class="login-form-options">
    <Toggle bind:toggled={remember}>{$t("login.REMEMBER")}</Toggle>
    <!-- TODO: forgot password -->
    <a class="login-form-forgot" href="/forgot">{$t("login.FORGOT")}</a>
  </div>

  <div class="login-form-submit">
    <Button on:click={login} disabled={busy} wide primary>
      {$t("login.LOGIN")}
    </Button>
  </div>
</div>

<style lang="scss">
  @import "../../css/abstracts";

  .login-form-options {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .login-form-forgot {
    font-size: 0.825rem;
    color: var(--col-hint);

    @include hover {
      text-decoration: underline;
    }
  }
</style>
