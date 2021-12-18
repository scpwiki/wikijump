<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { TextInput, Button } from "@wikijump/components"
  import { createEventDispatcher } from "svelte"
  import { inputsValid } from "@wikijump/dom"
  import FormError from "./FormError.svelte"

  const dispatch = createEventDispatcher()

  let busy = false
  let error = ""
  let started = false

  let inputEmail: HTMLInputElement

  function statusErrorMessage(status: number) {
    // prettier-ignore
    switch(status) {
      case 403: return $t("auth.errors.UNKNOWN_EMAIL")
      case 409: return $t("auth.errors.ALREADY_LOGGED_IN")
      default:  return $t("auth.errors.INTERNAL_ERROR")
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
      error = $t("auth.errors.INVALID_INPUT")
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
        label={$t("auth.EMAIL")}
        placeholder={$t("auth.EMAIL_PLACEHOLDER")}
        type="email"
        required
        disabled={busy}
        autocomplete="email"
        minlength="1"
      />
    </form>

    <div class="password-recovery-form-submit">
      <Button on:click={beginRecovery} disabled={busy} wide primary>
        {$t("auth.password_recovery.RESET_PASSWORD")}
      </Button>
    </div>
  </div>

  <FormError {error} />
{:else}
  <div class="password-recovery-email-sent">
    <p>{$t("auth.password_recovery.EMAIL_SENT")}</p>
  </div>
{/if}

<style lang="scss">
  .password-recovery-form-submit {
    margin-top: 1rem;
  }

  .password-recovery-email-sent {
    font-size: 0.825rem;
    margin-top: 1rem;
  }
</style>
