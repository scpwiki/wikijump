<!--
  @component Account registration form.
-->
<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { Button, TextInput } from "@wikijump/components"
  import { inputsValid } from "@wikijump/dom"
  import { format as t } from "@wikijump/fluent"
  import { escapeRegExp } from "@wikijump/util"
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
      case 403: return t("error-api.email-taken")
      case 500: return t("error-api.internal")
      default:  return t("error-api.register-failed")
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
      error = t("form-error.missing-fields")
    }
  }
</script>

<div class="register-form">
  <form>
    <TextInput
      bind:input={inputEmail}
      on:enter={() => inputUsername.focus()}
      icon="ic:round-alternate-email"
      label={t("email")}
      placeholder={t("email.placeholder")}
      type="email"
      info={t("email.info")}
      required
      disabled={busy}
      autocomplete="email"
      minlength="1"
    />

    <TextInput
      bind:input={inputUsername}
      on:enter={() => inputPassword.focus()}
      label={t("username")}
      placeholder={t("username.placeholder")}
      info={t("username.info")}
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
      label={t("password")}
      type="password"
      placeholder={t("password.placeholder")}
      required
      disabled={busy}
      autocomplete="new-password"
      minLength="1"
    />

    <TextInput
      bind:input={inputPasswordConfirm}
      on:enter={() => register()}
      icon="fluent:key-24-regular"
      label={t("confirm-password")}
      type="password"
      placeholder={t("password.placeholder")}
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
      {t("register")}
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
