<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { Button, TextInput } from "@wikijump/components"
  import { inputsValid } from "@wikijump/dom"
  import Locale from "@wikijump/fluent"
  import { createEventDispatcher } from "svelte"
  import FormError from "./FormError.svelte"

  const t = Locale.makeComponentFormatter("wiki-auth")

  const dispatch = createEventDispatcher()

  let busy = false
  let error = ""
  let started = false

  let inputEmail: HTMLInputElement

  function statusErrorMessage(status: number) {
    // prettier-ignore
    switch(status) {
      case 403: return $t("error-api.unknown-email")
      case 409: return $t("error-api.already-logged-in")
      default:  return $t("error-api.internal")
    }
  }

  async function beginRecovery() {
    if (inputsValid(inputEmail)) {
      busy = true

      try {
        const email = inputEmail.value
        await WikijumpAPI.accountStartRecovery({ email })
        dispatch("started")
        started = true
      } catch (err) {
        if (err instanceof Response) error = statusErrorMessage(err.status)
        else throw err
      }

      busy = false
    } else {
      error = $t("form-error.missing-fields")
    }
  }
</script>

{#if !started}
  <div class="password-recovery-form">
    <form>
      <TextInput
        bind:input={inputEmail}
        on:enter={beginRecovery}
        icon="ic:round-alternate-email"
        label={$t("email")}
        placeholder={$t("email.placeholder")}
        type="email"
        required
        disabled={busy}
        autocomplete="email"
        minlength="1"
      />
    </form>

    <div class="password-recovery-form-submit">
      <Button on:click={beginRecovery} disabled={busy} wide primary>
        {$t("reset-password")}
      </Button>
    </div>
  </div>

  <FormError {error} />
{:else}
  <div class="password-recovery-email-sent">
    <p>{$t("password-recovery.email-sent")}</p>
  </div>
{/if}

<style lang="scss">
  .password-recovery-form-submit {
    margin-top: 1rem;
  }

  .password-recovery-email-sent {
    margin-top: 1rem;
    font-size: 0.825rem;
  }
</style>
