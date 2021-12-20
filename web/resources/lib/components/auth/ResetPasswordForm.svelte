<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { format as t } from "@wikijump/fluent"
  import { TextInput, Button } from "@wikijump/components"
  import { createEventDispatcher } from "svelte"
  import { inputsValid } from "@wikijump/dom"
  import { escapeRegExp } from "@wikijump/util"
  import FormError from "./FormError.svelte"

  /**
   * If given, the component will automatically send the client to the
   * given URL. An empty string will be treated as `"/"`.
   */
  export let goto: null | true | string = null

  const dispatch = createEventDispatcher()

  let busy = false
  let error = ""

  let password = ""

  let inputPassword: HTMLInputElement
  let inputPasswordConfirm: HTMLInputElement

  async function resetPassword() {
    if (inputsValid(inputPassword, inputPasswordConfirm)) {
      busy = true
      try {
        const token = WikijumpAPI.getPathSegment(2)
        const email = WikijumpAPI.getQueryParameter("email")
        const password = inputPassword.value

        if (!token || !email || !password) {
          busy = false
          error = t("form-error.missing-field")
          return
        }

        await WikijumpAPI.post(window.location.href, { token, email, password })

        dispatch("reset")

        if (goto !== null) {
          window.location.href = goto === true ? "/" : goto || "/"
        }
      } catch {
        error = t("api-error.internal")
      }
      busy = false
    } else {
      error = t("form-error.missing-field")
    }
  }
</script>

<div class="password-reset-form">
  <form>
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
      on:enter={resetPassword}
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

  <div class="reset-password-form-submit">
    <Button on:click={resetPassword} disabled={busy} wide primary>
      {t("reset-password")}
    </Button>
  </div>
</div>

<style lang="scss">
  .reset-password-form > form {
    display: flex;
    flex-direction: column;
    row-gap: 0.25rem;
    margin-bottom: 1rem;
  }

  .reset-password-form-submit {
    margin-top: 1rem;
  }
</style>
