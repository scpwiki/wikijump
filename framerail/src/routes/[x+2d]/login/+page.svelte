<script lang="ts">
  import { invalidateAll } from "$app/navigation"
  import { errorPopupState } from "$lib/stores.svelte"
  import { superForm } from "sveltekit-superforms"
  import { untrack } from "svelte"
  import { toast } from "$lib/component/scripts/toasts"
  import { ToastType } from "$lib/types"

  import type { PageProps } from "./$types"

  let { data }: PageProps = $props()

  let isLoggedIn = $derived<boolean>(data.isLoggedIn)

  const { form, enhance } = superForm(
    untrack(() => data.loginForm),
    {
      onResult: async ({ result }) => {
        if (result.type === "success" && result.data) {
          toast(ToastType.Success, data.internationalization!["login.toast"]!)
          isLoggedIn = true
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
