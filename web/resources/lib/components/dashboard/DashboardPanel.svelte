<script lang="ts">
  import { format as t } from "@wikijump/fluent"
  import { Route, router } from "tinro"
  import { fade } from "svelte/transition"
  import * as Easings from "svelte/easing"
  import { dashboardRoute } from "./util"

  export let path = ""
  export let title = ""
  export let fallback = false

  $: if (title && $router.path.startsWith(dashboardRoute(path))) {
    document.title = t("base-title", { title })
  }
</script>

<Route {path} {fallback}>
  <div class="dashboard-panel" in:fade={{ duration: 150, easing: Easings.quartInOut }}>
    <slot />
  </div>
</Route>

<style global lang="scss">
</style>
