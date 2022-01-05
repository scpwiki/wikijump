<!--
  @component Login form.
-->
<script lang="ts">
  import WikijumpAPI, { route } from "@wikijump/api"
  import { format as t } from "@wikijump/fluent"
  import { Form, Button, TextInput, Toggle } from "@wikijump/components"
  import { createEventDispatcher } from "svelte"
  import FormError from "./FormError.svelte"

  /**
   * If given, the component will automatically send the client to the
   * given URL. An empty string will be treated as `"/"`.
   */
  export let back: null | true | string = null

  const dispatch = createEventDispatcher()

  async function onsubmit(values: {
    login: string
    password: string
    remember: boolean
  }) {
    const { login, password, remember } = values
    await WikijumpAPI.authLogin({ login, password, remember })

    dispatch("login")

    if (back !== null) {
      window.location.href = back === true ? "/" : back || "/"
    }
  }

  function onerror(err: unknown) {
    if (err instanceof Response) {
      // prettier-ignore
      switch(err.status) {
        case 409: return t("error-api.already-logged-in")
        case 500: return t("error-api.internal")
        default:  return t("error-api.login-failed")
      }
    } else {
      throw err
    }
  }
</script>

<div class="login-form">
  <Form {onsubmit} {onerror} let:busy let:error let:submit let:focusnext>
    <TextInput
      name="login"
      on:enter={focusnext}
      label={t("specifier")}
      placeholder={t("specifier.placeholder")}
      required
      disabled={busy}
      autocomplete="username"
      minlength="1"
    />

    <TextInput
      name="password"
      on:enter={submit}
      icon="fluent:key-24-regular"
      label={t("password")}
      type="password"
      placeholder={t("password.placeholder")}
      required
      disabled={busy}
      autocomplete="current-password"
      minLength="1"
    />

    <div class="login-form-options">
      <Toggle name="remember">{t("remember-me")}</Toggle>
      <a class="login-form-forgot" href={route("password.request")}>
        {t("forgot-password.question")}
      </a>
    </div>

    <div class="login-form-submit">
      <Button submit disabled={busy} wide primary>
        {t("login")}
      </Button>
    </div>

    <FormError {error} />
  </Form>
</div>

<style global lang="scss">
  .login-form form {
    display: flex;
    flex-direction: column;
    row-gap: 0.25rem;
  }

  .login-form-options {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 1rem 0;
  }

  .login-form-forgot {
    font-size: 0.825rem;
    @include link-styling(var(--col-hint));
  }
</style>
