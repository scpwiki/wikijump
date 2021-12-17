<!--
  @component Account registration form.
-->
<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button, TextInput } from "@wikijump/components"
  import { escapeRegExp } from "@wikijump/util"
  import { inputsValid } from "@wikijump/dom"
  import { createEventDispatcher } from "svelte"
  import FormError from "./FormError.svelte"

  /**
   * If given, the component will automatically send the client to the
   * given URL. An empty string will be treated as `"/"`.
   */
  export let goto: null | true | string = null

  const dispatch = createEventDispatcher()

  // TODO: redirect to email confirmation page
  // TODO: endpoint for verifying if password is safe
  // TODO: verifying that the username is available
  // TODO: captcha
  // TODO: hidden form field to bait spambots

  let busy = false

  let inputEmail: HTMLInputElement
  let inputUsername: HTMLInputElement
  let inputPassword: HTMLInputElement
  let inputPasswordConfirm: HTMLInputElement

  let error = ""

  let password = ""

  function statusErrorMessage(status: number) {
    // prettier-ignore
    switch(status) {
      case 403: return $t("auth.errors.EMAIL_TAKEN")
      case 500: return $t("auth.errors.INTERNAL_ERROR")
      default:  return $t("auth.errors.REGISTER_FAILED")
    }
  }

  async function register() {
    if (inputsValid(inputEmail, inputUsername, inputPassword, inputPasswordConfirm)) {
      busy = true
      error = ""

      try {
        const email = inputEmail.value
        const username = inputUsername.value
        const password = inputPassword.value

        await WikijumpAPI.accountRegister({ email, username, password })

        dispatch("register")

        if (goto !== null) {
          window.location.href = goto === true ? "/" : goto || "/"
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

<div class="register-form">
  <form>
    <TextInput
      bind:input={inputEmail}
      on:enter={() => inputUsername.focus()}
      icon="ic:round-alternate-email"
      label={$t("auth.EMAIL")}
      placeholder={$t("auth.EMAIL_PLACEHOLDER")}
      type="email"
      info={$t("auth.EMAIL_INFO")}
      required
      disabled={busy}
      autocomplete="email"
      minlength="1"
    />

    <TextInput
      bind:input={inputUsername}
      on:enter={() => inputPassword.focus()}
      label={$t("auth.USERNAME")}
      placeholder={$t("auth.USERNAME_PLACEHOLDER")}
      info={$t("auth.USERNAME_INFO")}
      required
      disabled={busy}
      autocomplete="username"
      minlength="1"
    />

    <TextInput
      bind:input={inputPassword}
      bind:value={password}
      on:enter={() => inputPasswordConfirm.focus()}
      icon="fluent:key-24-regular"
      label={$t("auth.PASSWORD")}
      type="password"
      placeholder={$t("auth.PASSWORD_PLACEHOLDER")}
      required
      disabled={busy}
      autocomplete="new-password"
      minLength="1"
    />

    <TextInput
      bind:input={inputPasswordConfirm}
      on:enter={() => register()}
      icon="fluent:key-24-regular"
      label={$t("auth.CONFIRM_PASSWORD")}
      type="password"
      placeholder={$t("auth.PASSWORD_PLACEHOLDER")}
      pattern={escapeRegExp(password)}
      required
      disabled={busy}
      autocomplete="new-password"
      minLength="1"
    />
  </form>

  <FormError {error} />

  <div class="register-form-submit">
    <Button on:click={register} disabled={busy} wide primary>
      {$t("auth.REGISTER")}
    </Button>
  </div>
</div>

<style lang="scss">
  .register-form > form {
    display: flex;
    flex-direction: column;
    row-gap: 0.25rem;
    margin-bottom: 1rem;
  }

  .register-form-submit {
    margin-top: 1rem;
  }
</style>
