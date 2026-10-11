<script lang="ts">
  import { goto } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { authCancelDestination } from "$lib/auth-cancel.js"
  import { untrack } from "svelte"
  import { superForm } from "sveltekit-superforms"

  import type { PageProps } from "./$types"

  let { data, form: actionData }: PageProps = $props()

  let enhancedRegistrationSucceeded = $state(false)
  let isRegistered = $derived(
    actionData?.isRegistered === true || enhancedRegistrationSucceeded
  )

  const cancelRegistration = async () => {
    await goto(authCancelDestination(), { replaceState: true })
  }

  const { form, enhance, errors } = superForm(
    untrack(() => data.registerForm),
    {
      onResult: async ({ result, cancel }) => {
        if (result.type === "success" && result.data?.isRegistered) {
          enhancedRegistrationSucceeded = true
          cancel()
          return
        }

        // Field-level validation failures are shown inline; only server
        // failures that carry a message open the error dialog.
        if (result.type === "failure" && result.data?.message) {
          errorPopupState.current = {
            state: true,
            message: result.data?.message,
            data: result.data?.data
          }
        }
      }
    }
  )
</script>

{#if isRegistered}
  <p role="status" class="register-success">
    {data.internationalization?.["register.toast"]}
    <a href="/-/login">{data.internationalization?.login}</a>
  </p>
{:else}
  <form id="register" class="register-form" method="POST" use:enhance>
    <label for="username">
      {data.internationalization?.["username"]}
    </label>
    <div class="input-container">
      <input
        id="username"
        name="username"
        class="username"
        autocomplete="username"
        placeholder={data.internationalization?.["username.placeholder"]}
        required
        type="text"
        bind:value={$form.username}
      />
      <p class="description">{data.internationalization?.["username.info"]}</p>
    </div>

    <label for="email">
      {data.internationalization?.["email"]}
    </label>
    <div class="input-container">
      <input
        id="email"
        name="email"
        class="email"
        autocomplete="email"
        placeholder={data.internationalization?.["email.placeholder"]}
        required
        type="text"
        bind:value={$form.email}
      />
      <p class="description">{data.internationalization?.["email.info"]}</p>
      {#if $errors.email}
        <!-- Although `error-api.INVALID_EMAIL` is almost the same as this one, it should
            get another name in `error-form` to avoid confusion -->
        <p class="error">UNTRANSLATED: Email format is invalid.</p>
      {/if}
    </div>

    <label for="password">
      {data.internationalization?.["password"]}
    </label>
    <div class="input-container">
      <input
        id="password"
        name="password"
        class="auth-password"
        autocomplete="new-password"
        aria-invalid={$errors.password ? "true" : undefined}
        aria-describedby={$errors.password ? "register-password-error" : undefined}
        placeholder={data.internationalization?.["password.placeholder"]}
        required
        type="password"
        bind:value={$form.password}
      />
      {#if $errors.password}
        <p id="register-password-error" class="error">
          {data.internationalization?.["error-form.password-too-short"]}
        </p>
      {/if}
    </div>

    <label for="confirm-password">
      {data.internationalization?.["confirm-password"]}
    </label>
    <div class="input-container">
      <input
        id="confirm-password"
        name="confirmPassword"
        class="confirm-password"
        autocomplete="new-password"
        aria-invalid={$errors.confirmPassword ? "true" : undefined}
        aria-describedby={$errors.confirmPassword
          ? "register-confirm-password-error"
          : undefined}
        placeholder={data.internationalization?.["password.placeholder"]}
        required
        type="password"
        bind:value={$form.confirmPassword}
      />
      {#if $errors.confirmPassword}
        <p id="register-confirm-password-error" class="error">
          {data.internationalization?.["error-form.password-mismatch"]}
        </p>
      {/if}
    </div>

    <label for="locale">UNTRANSLATED:Locale</label>
    <div class="input-container">
      <!-- TODO: Implement a multi select component -->
      <!-- I know it's ugly, but we can implement a better looking component later on -->
      <select id="locale" name="locale" multiple required bind:value={$form.locale}>
        {#each data.userInterfaceLocales as locale (locale.value)}
          <option value={locale.value}>{locale.label}</option>
        {/each}
      </select>
    </div>

    <!--
      Wikidot states the account-creation contract directly above the Sign up
      action: "By creating an account you accept our Terms of Service and
      Privacy Policy." A generic footer link is not that disclosure, so the
      sentence is part of the form, adjacent to Create Account, and the
      Create Account button names it with `aria-describedby`. Links come from
      the instance's authoritative legal destinations, never from the wiki
      being viewed.
    -->
    <p class="register-legal" id="register-legal-notice">
      {data.internationalization?.["register.legal-notice"]}<a
        class="register-legal-link"
        href={data.legalDestinations.termsOfService}
        rel="external">{data.internationalization?.["register.legal-terms"]}</a
      >{data.internationalization?.["register.legal-conjunction"]}<a
        class="register-legal-link"
        href={data.legalDestinations.privacyPolicy}
        rel="external">{data.internationalization?.["register.legal-privacy"]}</a
      >{data.internationalization?.["register.legal-suffix"]}
    </p>

    <div class="action-row auth-actions">
      <button
        class="action-button auth-button button-cancel clickable"
        onclick={cancelRegistration}
        type="button"
      >
        {data.internationalization?.cancel}
      </button>
      <button
        class="action-button auth-button button-create clickable"
        aria-describedby="register-legal-notice"
        type="submit"
      >
        {data.internationalization?.["create-account"]}
      </button>
    </div>
  </form>
{/if}

<style lang="scss">
  .register-form {
    display: grid;
    grid-template-columns: auto auto;

    gap: 1rem;
    justify-content: center;

    .input-container {
      display: flex;
      flex-direction: column;
      gap: 0.25rem;
      p {
        margin: 0;
        font-size: 0.8rem;
        &.description {
          color: #666;
        }
        &.error {
          color: red;
        }
      }
    }

    .register-legal {
      grid-column: span 2;
      margin: 0;
      font-size: 0.8rem;
      color: #666;
      text-align: center;

      .register-legal-link {
        color: inherit;
        text-decoration: underline;
      }
    }

    .action-row {
      grid-column: span 2;
      justify-self: center;
    }

    // Two fixed columns cannot fit a 375px phone viewport, so stack the fields.
    @media (max-width: 600px) {
      grid-template-columns: minmax(0, 1fr);

      .input-container {
        min-width: 0;
      }

      .action-row,
      .register-legal {
        grid-column: auto;
      }

      .register-legal {
        text-align: start;
      }
    }
  }
</style>
