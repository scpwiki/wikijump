<script lang="ts">
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/layout/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"

  import type { PageProps } from "./$types"

  let { data }: PageProps = $props()

  let isLoggedIn = $derived<boolean>(data.isLoggedIn)
  let mfaSessionToken = $state<string | undefined>()
  let totpOrCode = $state("")

  const { form, enhance } = superForm(
    untrack(() => data.loginForm),
    {
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          if (result.data.needsMfa && result.data.session_token) {
            mfaSessionToken = result.data.session_token
            isLoggedIn = false
            return
          }

          mfaSessionToken = undefined
          isLoggedIn = result.data.isLoggedIn
          await invalidateAll()
          return
        }

        if (result.type === "failure" && result.data) {
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

{#if isLoggedIn}
  {data.internationalization?.["login.toast"]}
{:else if mfaSessionToken}
  <form id="login-mfa" class="login-form" method="POST" use:enhance>
    <input name="mfaSessionToken" type="hidden" value={mfaSessionToken} />
    <input
      name="totpOrCode"
      class="auth-mfa-code"
      autocomplete="one-time-code"
      placeholder="MFA code"
      type="text"
      bind:value={totpOrCode}
    />
    <div class="action-row auth-actions">
      <button class="action-button auth-button button-login clickable" type="submit">
        {data.internationalization?.login}
      </button>
    </div>
  </form>
{:else}
  <form id="login" class="login-form" method="POST" use:enhance>
    <input
      name="nameOrEmail"
      class="auth-name-or-email"
      placeholder={data.internationalization?.specifier}
      type="text"
      bind:value={$form.nameOrEmail}
    />
    <input
      name="password"
      class="auth-password"
      placeholder={data.internationalization?.password}
      type="password"
      bind:value={$form.password}
    />
    <div class="action-row auth-actions">
      <button class="action-button auth-button button-cancel clickable" type="button">
        {data.internationalization?.cancel}
      </button>
      <button class="action-button auth-button button-login clickable" type="submit">
        {data.internationalization?.login}
      </button>
    </div>
  </form>
{/if}

<style lang="scss">
  .login-form {
    display: flex;
    flex-direction: column;
    gap: 1em;
    align-items: center;
    justify-content: center;

    .action-row {
      justify-content: center;
    }
  }
</style>
