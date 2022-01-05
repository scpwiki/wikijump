<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { Form, Button, TextInput } from "@wikijump/components"
  import Locale from "@wikijump/fluent"
  import { createEventDispatcher } from "svelte"
  import FormError from "./FormError.svelte"

  const t = Locale.makeComponentFormatter("wiki-auth")

  const dispatch = createEventDispatcher()

  let started = false

  async function onsubmit(values: { email: string }) {
    await WikijumpAPI.accountStartRecovery({ email: values.email })
    dispatch("started")
    started = true
  }

  function onerror(err: unknown) {
    if (err instanceof Response) {
      // prettier-ignore
      switch(err.status) {
        case 403: return $t("error-api.unknown-email")
        case 409: return $t("error-api.already-logged-in")
        default:  return $t("error-api.internal")
      }
    } else {
      throw err
    }
  }
</script>

{#if !started}
  <div class="password-recovery-form">
    <Form {onsubmit} {onerror} let:busy let:error let:submit>
      <TextInput
        name="email"
        on:enter={submit}
        icon="ic:round-alternate-email"
        label={$t("email")}
        placeholder={$t("email.placeholder")}
        type="email"
        required
        disabled={busy}
        autocomplete="email"
        minlength="1"
      />

      <div class="password-recovery-form-submit">
        <Button submit disabled={busy} wide primary>
          {$t("reset-password")}
        </Button>
      </div>

      <FormError {error} />
    </Form>
  </div>
{:else}
  <div class="password-recovery-email-sent">
    <p>{$t("password-recovery.email-sent")}</p>
  </div>
{/if}

<style global lang="scss">
  .password-recovery-form-submit {
    margin-top: 1rem;
  }

  .password-recovery-email-sent {
    margin-top: 1rem;
    font-size: 0.825rem;
  }
</style>
