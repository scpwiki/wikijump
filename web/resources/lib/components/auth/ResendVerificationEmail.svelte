<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { Button } from "@wikijump/components"
  import Locale from "@wikijump/fluent"
  import FormError from "./FormError.svelte"

  const t = Locale.makeComponentFormatter("wiki-auth")

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
      resent = false
      if (err instanceof Response) {
        // you can't get to this page without being logged in and unverified
        // so probably any error is internal
        error = $t("error-api.internal")
      } else {
        throw err
      }
    }

    busy = false
  }
</script>

<Button on:click={resend} disabled={busy} wide primary>
  {$t("#-verify-email.resend-email")}
</Button>

{#if resent && !busy}
  <p class="verify-email-resent">{$t("#-verify-email.email-sent")}</p>
{/if}

<FormError {error} />

<style global lang="scss">
  .verify-email-resent {
    margin-top: 0.5rem;
    font-size: 0.825rem;
    text-align: center;
  }
</style>
