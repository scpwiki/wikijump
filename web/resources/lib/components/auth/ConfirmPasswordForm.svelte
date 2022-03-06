<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { Form, Button, TextInput } from "@wikijump/components"
  import { format as t } from "@wikijump/fluent"
  import { createEventDispatcher } from "svelte"
  import FormError from "./FormError.svelte"

  /**
   * If given, the component will automatically send the client to the
   * given URL. An empty string will be treated as `"/"`.
   */
  export let back: null | true | string = null

  const dispatch = createEventDispatcher()

  async function onsubmit(values: { password: string }) {
    await WikijumpAPI.authConfirm({ password: values.password })

    dispatch("confirm")

    if (back !== null) {
      window.location.href = back === true ? "/" : back || "/"
    }
  }
</script>

<div class="confirm-form">
  <Form {onsubmit} let:busy let:error let:submit>
    <TextInput
      name="password"
      on:enter={submit}
      icon="fluent:key-24-regular"
      type="password"
      placeholder={t("password.placeholder")}
      label={t("password")}
      required
      disabled={busy}
      autocomplete="current-password"
      minLength="1"
    />

    <div class="confirm-form-submit">
      <Button submit disabled={busy} wide primary>
        {t("confirm-password")}
      </Button>
    </div>

    <FormError {error} />
  </Form>
</div>

<style global lang="scss">
  .confirm-form-submit {
    margin-top: 1rem;
  }
</style>
