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
      case 500: return $t("auth.errors.INTERNAL_ERROR")
      default:  return $t("auth.errors.CONFIRM_FAILED")
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
      error = $t("auth.errors.INVALID_INPUT")
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
      placeholder={$t("auth.PASSWORD_PLACEHOLDER")}
      required
      disabled={busy}
      autocomplete="current-password"
      minLength="1"
    />
  </form>

  <div class="confirm-form-submit">
    <Button on:click={confirm} disabled={busy} wide primary>
      {$t("auth.CONFIRM")}
    </Button>
  </div>

  <FormError {error} />
</div>

<!-- TODO: forgot password -->
<a class="confirm-form-forgot" href="/forgot">{$t("auth.FORGOT_PASSWORD")}</a>

<style lang="scss">
  @import "../../css/abstracts";

  .confirm-form-submit {
    margin-top: 1rem;
  }

  .confirm-form-forgot {
    font-size: 0.825rem;
    display: block;
    margin-top: 1rem;
    text-align: center;
    @include link-styling(var(--col-hint));
  }
</style>
