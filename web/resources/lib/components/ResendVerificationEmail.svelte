<script lang="ts">
  import WikijumpAPI, { t } from "@wikijump/api"
  import { Button } from "@wikijump/components"
  import FormError from "./FormError.svelte"

  let busy = false
  let resent = false
  let error = ""

  async function resend() {
    busy = true
    error = ""

    try {
      await WikijumpAPI.accountSendVerificationEmail()
      resent = true
    } catch (err) {
      if (err instanceof Response) {
        // you can't get to this page without being logged in and unverified
        // so probably any error is internal
        error = $t("account_panel.errors.INTERNAL_ERROR")
      } else {
        throw err
      }
    }

    busy = false
  }
</script>

<Button on:click={resend} disabled={busy} wide primary>
  {$t("account_panel.verify_email.RESEND_EMAIL")}
</Button>

{#if resent && !busy}
  <p class="verify-email-resent">{$t("account_panel.verify_email.EMAIL_SENT")}</p>
{/if}

<FormError {error} />

<style lang="scss">
  .verify-email-resent {
    text-align: center;
    margin-top: 0.5rem;
    font-size: 0.825rem;
  }
</style>
