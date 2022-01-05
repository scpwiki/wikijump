<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { Form, Button } from "@wikijump/components"
  import Locale from "@wikijump/fluent"
  import FormError from "./FormError.svelte"

  const t = Locale.makeComponentFormatter("wiki-auth")

  async function onsubmit() {
    await WikijumpAPI.accountSendVerificationEmail()
  }

  function onerror(err: unknown) {
    if (err instanceof Response) return $t("error-api.internal")
    else throw err
  }
</script>

<Form {onsubmit} {onerror} let:busy let:fired let:error>
  <Button submit disabled={busy} wide primary>
    {$t("#-verify-email.resend-email")}
  </Button>

  {#if fired && !busy}
    <p class="verify-email-resent">{$t("#-verify-email.email-sent")}</p>
  {/if}

  <FormError {error} />
</Form>

<style global lang="scss">
  .verify-email-resent {
    margin-top: 0.5rem;
    font-size: 0.825rem;
    text-align: center;
  }
</style>
