<script lang="ts">
  import { active } from "tinro"
  import { dashboardRoute } from "./util"
  export let path = ""
  export let aslink = false
</script>

{#if !aslink}
  <li class="dashboard-link-list-item">
    <a class="dashboard-link is-list-item" href={dashboardRoute(path)} use:active>
      <slot />
    </a>
  </li>
{:else}
  <a class="dashboard-link is-link" href={dashboardRoute(path)} use:active>
    <slot />
  </a>
{/if}

<style global lang="scss">
</style>
