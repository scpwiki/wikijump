<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button, TextInput, toast } from "@wikijump/components"
  import { escapeRegExp } from "@wikijump/util"
  import { createEventDispatcher } from "svelte"
  import { inputsValid } from "../util"
  import FormError from "./FormError.svelte"

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
      case 403: return $t("account_panel.errors.EMAIL_TAKEN")
      case 500: return $t("account_panel.errors.INTERNAL_ERROR")
      default:  return $t("account_panel.errors.REGISTER_FAILED")
    }
  }

  async function register() {
    if (inputsValid(inputEmail, inputUsername, inputPassword, inputPasswordConfirm)) {
      busy = true
      try {
        const email = inputEmail.value
        const username = inputUsername.value
        const password = inputPassword.value

        // does nothing right now
        await WikijumpAPI.accountRegister({ email, username, password })

        toast("success", $t("account_panel.toasts.REGISTERED"))
        dispatch("register")
      } catch (err) {
        // handle HTTP errors, rethrow on script errors
        if (err instanceof Response) error = statusErrorMessage(err.status)
        else throw err
      }
      busy = false
    } else {
      error = $t("account_panel.errors.INVALID_INPUT")
    }
  }
</script>

<div class="register-form">
  <form>
    <TextInput
      bind:input={inputEmail}
      on:enter={() => inputUsername.focus()}
      label={$t("account_panel.EMAIL")}
      placeholder={$t("account_panel.EMAIL_PLACEHOLDER")}
      type="email"
      info={$t("account_panel.EMAIL_INFO")}
      required
      disabled={busy}
      autocomplete="email"
      minlength="1"
    />

    <TextInput
      bind:input={inputUsername}
      on:enter={() => inputPassword.focus()}
      label={$t("account_panel.USERNAME")}
      placeholder={$t("account_panel.USERNAME_PLACEHOLDER")}
      info={$t("account_panel.USERNAME_INFO")}
      required
      disabled={busy}
      autocomplete="username"
      minlength="1"
    />

    <TextInput
      bind:input={inputPassword}
      bind:value={password}
      on:enter={() => inputPasswordConfirm.focus()}
      label={$t("account_panel.PASSWORD")}
      type="password"
      placeholder={$t("account_panel.PASSWORD_PLACEHOLDER")}
      required
      disabled={busy}
      autocomplete="new-password"
      minLength="1"
    />

    <TextInput
      bind:input={inputPasswordConfirm}
      on:enter={() => register()}
      label={$t("account_panel.CONFIRM_PASSWORD")}
      type="password"
      placeholder={$t("account_panel.PASSWORD_PLACEHOLDER")}
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
      {$t("account_panel.REGISTER")}
    </Button>
  </div>
</div>

<style lang="scss">
  @import "../../css/abstracts";

  .register-form > form {
    margin-bottom: 1rem;
  }

  .register-form-submit {
    margin-top: 1rem;
  }
</style>
