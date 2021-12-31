<script lang="ts">
  import API, { type AccountSettings, type UserProfile } from "@wikijump/api"
  import { Spinny } from "@wikijump/components"
  import { Route } from "tinro"
  import DashboardPanel from "../DashboardPanel.svelte"
  import { dashboardRoute } from "../util"

  interface Settings {
    profile: UserProfile
    account: AccountSettings
    email: string
  }

  async function getData(): Promise<Settings> {
    const profile = (await API.userClientGet({ detail: "profile" })) as UserProfile
    const account = await API.accountGetSettings()
    const { email } = await API.accountGetEmail()
    return { profile, account, email }
  }
</script>

{#await getData()}
  <Spinny />
{:then data}
  <Route>
    <Route fallback redirect={dashboardRoute("settings/account")} />

    <DashboardPanel path="/profile" />

    <DashboardPanel path="/account">
      <pre><code>{JSON.stringify(data, null, 2)}</code></pre>
    </DashboardPanel>

    <DashboardPanel path="/about" />
  </Route>
{/await}

<style global lang="scss">
</style>
