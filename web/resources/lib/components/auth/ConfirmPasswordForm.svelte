<!--
  @component Login form.
-->
<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { Button, TextInput } from "@wikijump/components"
  import { inputsValid } from "@wikijump/dom"
  import { format as t } from "@wikijump/fluent"
  import { createEventDispatcher } from "svelte"
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
      case 500: return t("error-api.internal")
      default:  return t("error-api.password-confirm-failed")
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
      error = t("error-form.missing-fields")
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
      placeholder={t("password.placeholder")}
      label={t("password")}
      required
      disabled={busy}
      autocomplete="current-password"
      minLength="1"
    />
  </form>

  <div class="confirm-form-submit">
    <Button on:click={confirm} disabled={busy} wide primary>
      {t("confirm-password")}
    </Button>
  </div>

  <FormError {error} />
</div>

<style global lang="scss">
  .confirm-form-submit {
    margin-top: 1rem;
  }
</style>
