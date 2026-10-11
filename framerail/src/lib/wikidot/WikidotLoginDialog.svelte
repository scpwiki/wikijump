<!--
  @component In-place Sign in surface for imported Wikidot-layout pages.
  Submits the native /-/login form action without navigating the page. Cancel,
  Escape, and successful sign-in close the surface and leave the page in place.
-->
<script lang="ts">
  import { enhance } from "$app/forms"
  import { invalidateAll } from "$app/navigation"
  import { resolve } from "$app/paths"
  import {
    WIKIDOT_LOGIN_DIALOG_LABELS,
    interpretLoginActionResult
  } from "$lib/wikidot/wikidot-login-dialog.js"

  let {
    open = false,
    onclose
  }: {
    open?: boolean
    onclose: () => void
  } = $props()

  const labels = WIKIDOT_LOGIN_DIALOG_LABELS

  let dialog = $state<HTMLDialogElement>()
  let formElement = $state<HTMLFormElement>()
  let pending = $state(false)
  let message = $state<string | null>(null)
  let mfaRequired = $state(false)

  $effect(() => {
    if (!dialog) return
    if (open && !dialog.open) {
      dialog.showModal()
      formElement?.querySelector<HTMLInputElement>("input[name=nameOrEmail]")?.focus()
    } else if (!open && dialog.open) {
      dialog.close()
    }
  })

  /** Never let a password value survive the surface, whatever the outcome. */
  function clearPassword() {
    const password = formElement?.elements.namedItem("password")
    if (password instanceof HTMLInputElement) password.value = ""
  }

  function cancel() {
    clearPassword()
    message = null
    mfaRequired = false
    onclose()
  }
</script>

<dialog
  bind:this={dialog}
  class="wikidot-login-dialog"
  aria-labelledby="wikidot-login-dialog-title"
  onclose={() => {
    clearPassword()
    if (open) onclose()
  }}
>
  <form
    bind:this={formElement}
    class="login-form"
    method="POST"
    action={resolve("/-/login", {})}
    autocomplete="off"
    use:enhance={() => {
      pending = true
      message = null
      mfaRequired = false
      return async ({ result }) => {
        pending = false
        clearPassword()
        const outcome = interpretLoginActionResult(result)
        if (outcome.kind === "success") {
          dialog?.close()
          await invalidateAll()
          return
        }
        if (outcome.kind === "mfa") {
          mfaRequired = true
          return
        }
        message =
          outcome.kind === "failure" && outcome.message
            ? outcome.message
            : outcome.kind === "failure"
              ? labels.failed
              : labels.unavailable
      }
    }}
  >
    <h2 id="wikidot-login-dialog-title">{labels.title}</h2>
    <label for="wikidot-login-name-or-email">{labels.specifier}</label>
    <input
      id="wikidot-login-name-or-email"
      name="nameOrEmail"
      class="auth-name-or-email"
      autocomplete="username"
      type="text"
      required
    />
    <label for="wikidot-login-password">{labels.password}</label>
    <input
      id="wikidot-login-password"
      name="password"
      class="auth-password"
      autocomplete="current-password"
      type="password"
      required
    />
    {#if message}
      <p class="wikidot-login-dialog-message" role="alert">{message}</p>
    {/if}
    {#if mfaRequired}
      <p class="wikidot-login-dialog-message" role="status">
        {labels.mfaRequired}
        <a href={resolve("/-/login", {})}>{labels.signInPage}</a>
      </p>
    {/if}
    <div class="action-row">
      <button class="button-cancel" type="button" onclick={cancel}>{labels.cancel}</button
      >
      <button class="button-login" type="submit" disabled={pending}
        >{labels.submit}</button
      >
    </div>
  </form>
</dialog>

<style>
  .wikidot-login-dialog {
    max-width: min(24em, calc(100vw - 2em));
    padding: 1.25em;
    border: 1px solid #999;
    border-radius: 0.5em;
    background: #fff;
    color: #222;
  }

  .wikidot-login-dialog::backdrop {
    background: rgb(0 0 0 / 0.4);
  }

  .wikidot-login-dialog h2 {
    margin: 0 0 0.75em;
    font-size: 1.25em;
  }

  .login-form {
    display: flex;
    flex-direction: column;
    gap: 0.5em;
  }

  .login-form input {
    padding: 0.4em;
    font: inherit;
  }

  .action-row {
    display: flex;
    justify-content: flex-end;
    gap: 0.5em;
    margin-top: 0.5em;
  }

  .wikidot-login-dialog-message {
    margin: 0.25em 0;
    color: #a00;
  }
</style>
