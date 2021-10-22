<!--
  @component Login form.
-->
<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button, TextInput } from "@wikijump/components"
  import { createEventDispatcher } from "svelte"
  import { inputsValid } from "@wikijump/dom"
  import FormError from "./FormError.svelte"

  const dispatch = createEventDispatcher()

  let busy = false
  let error = ""
  let inputPassword: HTMLInputElement

  function statusErrorMessage(status: number) {
    // prettier-ignore
    switch(status) {
      case 500: return $t("account_panel.errors.INTERNAL_ERROR")
      default:  return $t("account_panel.errors.CONFIRM_FAILED")
    }
  }

  async function confirm() {
    if (inputsValid(inputPassword)) {
      busy = true
      try {
        await WikijumpAPI.authConfirm({ password: inputPassword.value })
        dispatch("confirm")
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

<div class="confirm-form">
  <form>
    <TextInput
      bind:input={inputPassword}
      on:enter={confirm}
      icon="fluent:key-24-regular"
      type="password"
      placeholder={$t("account_panel.PASSWORD_PLACEHOLDER")}
      required
      disabled={busy}
      autocomplete="current-password"
      minLength="1"
    />
  </form>

  <FormError {error} />

  <div class="confirm-form-submit">
    <Button on:click={confirm} disabled={busy} wide primary>
      {$t("account_panel.CONFIRM")}
    </Button>
  </div>
</div>

<!-- TODO: forgot password -->
<a class="confirm-form-forgot" href="/forgot">{$t("account_panel.FORGOT_PASSWORD")}</a>

<style lang="scss">
  @import "../../css/abstracts";

  .confirm-form-submit {
    margin-top: 0.5rem;
  }

  .confirm-form-forgot {
    font-size: 0.825rem;
    display: block;
    margin-top: 1rem;
    text-align: center;
    @include link-styling(var(--col-hint));
  }
</style>
