<!--
  @component Account registration form.
-->
<script lang="ts">
  import WikijumpAPI from "@wikijump/api"
  import { Form, Button, TextInput } from "@wikijump/components"
  import { format as t } from "@wikijump/fluent"
  import { escapeRegExp } from "@wikijump/util"
  import { createEventDispatcher } from "svelte"
  import FormError from "./FormError.svelte"

  /**
   * If given, the component will automatically send the client to the
   * given URL. An empty string will be treated as `"/"`.
   */
  export let goto: null | true | string = null

  const dispatch = createEventDispatcher()

  // TODO: redirect to email confirmation page
  // TODO: endpoint for verifying if password is safe
  // TODO: verifying that the username is available
  // TODO: captcha
  // TODO: hidden form field to bait spambots

  async function onsubmit(values: {
    email: string
    username: string
    password: string
    passwordConfirm: string
  }) {
    const { email, username, password } = values
    await WikijumpAPI.accountRegister({ email, username, password })

    dispatch("register")

    if (goto !== null) {
      window.location.href = goto === true ? "/" : goto || "/"
    }
  }

  function onerror(err: unknown) {
    if (err instanceof Response) {
      // prettier-ignore
      switch(err.status) {
        case 403: return t("error-api.email-taken")
        case 500: return t("error-api.internal")
        default:  return t("error-api.register-failed")
      }
    } else {
      throw err
    }
  }

  let password = ""
</script>

<div class="register-form">
  <Form {onsubmit} {onerror} let:busy let:error let:submit let:focusnext>
    <TextInput
      name="email"
      on:enter={focusnext}
      icon="ic:round-alternate-email"
      label={t("email")}
      placeholder={t("email.placeholder")}
      type="email"
      info={t("email.info")}
      required
      disabled={busy}
      autocomplete="email"
      minlength="1"
    />

    <TextInput
      name="username"
      on:enter={focusnext}
      label={t("username")}
      placeholder={t("username.placeholder")}
      info={t("username.info")}
      required
      disabled={busy}
      autocomplete="username"
      minlength="1"
    />

    <TextInput
      name="password"
      bind:value={password}
      on:enter={focusnext}
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
      name="passwordConfirm"
      on:enter={submit}
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

    <div class="register-form-submit">
      <Button submit disabled={busy} wide primary>
        {t("register")}
      </Button>
    </div>

    <FormError {error} />
  </Form>
</div>

<style global lang="scss">
  .register-form > form {
    display: flex;
    flex-direction: column;
    row-gap: 0.25rem;
    margin-bottom: 1rem;
  }

  .register-form-submit {
    margin-top: 1rem;
  }
</style>
