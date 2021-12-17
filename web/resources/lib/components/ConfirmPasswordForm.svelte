<!--
  @component Login form.
-->
<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button, TextInput } from "@wikijump/components"
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

<div class="confirm-form">
  <form>
    <TextInput
      bind:input={inputPassword}
      on:enter={confirm}
      icon="fluent:key-24-regular"
      type="password"
      placeholder={$t("auth.PASSWORD_PLACEHOLDER")}
      label={$t("auth.PASSWORD")}
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

<style lang="scss">
  .confirm-form-submit {
    margin-top: 1rem;
  }
</style>
