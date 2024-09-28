<script lang="ts">
  import { page } from "$app/stores"
  import { invalidateAll } from "$app/navigation"
  import { useErrorPopup } from "$lib/stores"
  let showErrorPopup = useErrorPopup()

  let isLoggedIn = $page.data.isLoggedIn

  async function tryLogin() {
    let form = document.getElementById("login")
    let fdata = new FormData(form)
    let res = await fetch(`/-/login`, {
      method: "POST",
      body: fdata
    }).then((res) => res.json())

    if (res.session_token) {
      isLoggedIn = true
      invalidateAll()
    } else {
      showErrorPopup.set({
        state: true,
        message: res.message,
        data: res.data
      })
    }
  }
</script>

{#if isLoggedIn}
  {$page.data.internationalization?.["login.toast"]}
{:else}
  <form id="login" class="login-form" method="POST" on:submit|preventDefault={tryLogin}>
    <input
      name="name-or-email"
      class="auth-name-or-email"
      placeholder={$page.data.internationalization?.specifier}
      type="text"
    />
    <input
      name="password"
      class="auth-password"
      placeholder={$page.data.internationalization?.password}
      type="password"
    />
    <div class="action-row auth-actions">
      <button
        class="action-button auth-button button-cancel clickable"
        type="button"
        on:click|stopPropagation={() => {}}
      >
        {$page.data.internationalization?.cancel}
      </button>
      <button
        class="action-button auth-button button-login clickable"
        type="submit"
        on:click|stopPropagation
      >
        {$page.data.internationalization?.login}
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
  }
</style>
