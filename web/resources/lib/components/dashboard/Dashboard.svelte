<script lang="ts">
  import Locale from "@wikijump/fluent"
  import { Route, router } from "tinro"
  import RouteAnnouncer from "../RouteAnnouncer.svelte"
  import TransitionRoute from "../TransitionRoute.svelte"
  import DashboardLink from "./DashboardLink.svelte"
  import DashboardPanel from "./DashboardPanel.svelte"
  import { dashboardRoute } from "./util"

  const t = Locale.makeComponentFormatter("dashboard")

  // bit of a hack, unfortunately, but this is to prevent Tinro
  // from hijacking non-dashboard routes

  const selector = `a:not([data-tinro-ignore]):not([href^="${dashboardRoute()}"])`

  const observer = new MutationObserver(() => {
    const links = Array.from(document.body.querySelectorAll<HTMLAnchorElement>(selector))
    for (const link of links) {
      link.setAttribute("data-tinro-ignore", "")
    }
  })

  observer.observe(document.body, { childList: true, subtree: true })
</script>

<div class="dashboard">
  <RouteAnnouncer />

  <div class="dashboard-tabs">
    <ul class="dashboard-links">
      <DashboardLink path="profile">{$t("profile")}</DashboardLink>
      <DashboardLink path="settings">{$t("settings")}</DashboardLink>
    </ul>
  </div>

  <div class="dashboard-panels">
    <TransitionRoute>
      <Route path={dashboardRoute("*")} firstmatch>
        <Route fallback redirect={dashboardRoute("profile")} />

        <DashboardPanel path="/profile" title={$t("profile")}>
          <h1>Profile</h1>
          <p>
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pellentesque euismod,
            urna eu tincidunt consectetur, nisi nunc ultricies nisi, eget consectetur nunc
            nisi vitae nunc.
          </p>
        </DashboardPanel>

        <DashboardPanel path="/settings" title={$t("settings")}>
          <h1>Settings</h1>
          <p>
            Lorem ipsum dolor sit amet, consectetur adipiscing elit. Pellentesque euismod,
            urna eu tincidunt consectetur, nisi nunc ultricies nisi, eget consectetur nunc
            nisi vitae nunc.
          </p>
        </DashboardPanel>
      </Route>
    </TransitionRoute>
  </div>
</div>

<style global lang="scss">
</style>
