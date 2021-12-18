<!--
  @component Login form.
-->
<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button, TextInput, Toggle } from "@wikijump/components"
  import { createEventDispatcher } from "svelte"
  import { inputsValid } from "@wikijump/dom"
  import FormError from "./FormError.svelte"

  /**
   * If given, the component will automatically send the client to the
   * given URL. An empty string will be treated as `"/"`.
   */
  export let back: null | true | string = null

  const dispatch = createEventDispatcher()

  let busy = false

  let inputLogin: HTMLInputElement
  let inputPassword: HTMLInputElement
  let remember = false

  let error = ""

  function statusErrorMessage(status: number) {
    // prettier-ignore
    switch(status) {
      case 409: return $t("auth.errors.ALREADY_LOGGED_IN")
      case 500: return $t("auth.errors.INTERNAL_ERROR")
      default:  return $t("auth.errors.LOGIN_FAILED")
    }
  }

  async function login() {
    if (inputsValid(inputLogin, inputPassword)) {
      busy = true
      try {
        const login = inputLogin.value
        const password = inputPassword.value
        await WikijumpAPI.authLogin({ login, password, remember })
        dispatch("login")

        if (back !== null) {
          window.location.href = back === true ? "/" : back || "/"
        }
      } catch (err) {
        // handle HTTP errors, rethrow on script errors
        if (err instanceof Response) error = statusErrorMessage(err.status)
        else throw err
      }
      busy = false
    } else {
      error = $t("auth.errors.INVALID_INPUT")
    }
  }
</script>

<div class="login-form">
  <form>
    <TextInput
      bind:input={inputLogin}
      on:enter={() => inputPassword.focus()}
      label={$t("auth.SPECIFIER")}
      placeholder={$t("auth.SPECIFIER_PLACEHOLDER")}
      required
      disabled={busy}
      autocomplete="username"
      minlength="1"
    />

    <TextInput
      bind:input={inputPassword}
      on:enter={() => login()}
      icon="fluent:key-24-regular"
      label={$t("auth.PASSWORD")}
      type="password"
      placeholder={$t("auth.PASSWORD_PLACEHOLDER")}
      required
      disabled={busy}
      autocomplete="current-password"
      minLength="1"
    />
  </form>

  <div class="login-form-options">
    <Toggle bind:toggled={remember}>{$t("auth.REMEMBER_ME")}</Toggle>
    <a class="login-form-forgot" href="/user--services/forgot-password">
      {$t("auth.FORGOT_PASSWORD")}
    </a>
  </div>

  <FormError {error} />

  <div class="login-form-submit">
    <Button on:click={login} disabled={busy} wide primary>
      {$t("auth.LOGIN")}
    </Button>
  </div>
</div>

<style lang="scss">
  @import "../../../css/abstracts";

  .login-form form {
    display: flex;
    flex-direction: column;
    row-gap: 0.25rem;
  }

  .login-form-options {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 1rem 0;
  }

  .login-form-forgot {
    font-size: 0.825rem;
    @include link-styling(var(--col-hint));
  }
</style>
