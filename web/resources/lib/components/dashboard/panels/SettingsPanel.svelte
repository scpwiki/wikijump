<script lang="ts" context="module">
  export interface Settings {
    profile: UserProfile
    account: AccountSettings
    email: string
  }
</script>

<script lang="ts">
  import API, { type AccountSettings, type UserProfile } from "@wikijump/api"
  import Locale from "@wikijump/fluent"
  import { Spinny } from "@wikijump/components"
  import { Route } from "tinro"
  import DashboardPanel from "../DashboardPanel.svelte"
  import { dashboardRoute } from "../Dashboard.svelte"
  import DashboardPanelHeader from "../DashboardPanelHeader.svelte"
  import SettingsProfileForm from "./settings/SettingsProfileForm.svelte"
  import SettingsAccountForm from "./settings/SettingsAccountForm.svelte"

  const t = Locale.makeComponentFormatter("dashboard")

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
  <Route path="/" redirect={dashboardRoute("settings/profile")} />

  <DashboardPanel path="/profile">
    <DashboardPanelHeader>{$t("#-profile")}</DashboardPanelHeader>
    <SettingsProfileForm {data} />
  </DashboardPanel>

  <DashboardPanel path="/account">
    <DashboardPanelHeader>{$t("#-account")}</DashboardPanelHeader>
    <SettingsAccountForm {data} />
  </DashboardPanel>
{/await}

<style global lang="scss">
  .dashboard-settings-profile {
    max-width: 30rem;
  }

  .dashboard-settings-profile-inputs {
    display: flex;
    flex-direction: column;
    margin-bottom: 1rem;
  }
</style>
