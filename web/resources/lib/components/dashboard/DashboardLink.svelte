<script lang="ts">
  import { Button, Sprite } from "@wikijump/components"
  import { router } from "tinro"
  import { dashboardRoute } from "./Dashboard.svelte"
  import { unfold } from "@wikijump/components/lib"

  export let path = ""
  export let subpaths: { path: string; title: string }[] = []

  $: active = $router.path.startsWith(dashboardRoute(path))

  $: topPath = dashboardRoute(path)

  // automatically use first subpath
  $: if (path && subpaths[0]?.path) {
    topPath = `${dashboardRoute(path)}/${subpaths[0].path}`
  }
</script>

<li class="dashboard-link" class:has-subpaths={subpaths.length}>
  <Button href={topPath} {active} baseline wide compact primary>
    <slot />
    {#if subpaths.length}
      <Sprite i="wj-downarrow" size="0.75rem" />
    {/if}
  </Button>
  {#if active && subpaths.length}
    <div
      class="dashboard-subpaths-container"
      transition:unfold={{
        duration: 300,
        easing: "circInOut"
      }}
    >
      <hr class="dashboard-link-hr" />
      <ul class="dashboard-subpaths">
        {#each subpaths as { path: subpath, title } (subpath)}
          <li class="dashboard-subpath">
            <Button
              href={`${dashboardRoute(path)}/${subpath}`}
              active={$router.path.endsWith(subpath)}
              baseline
              wide
              compact
            >
              {title}
            </Button>
          </li>
        {/each}
      </ul>
    </div>
  {/if}
</li>

<style global lang="scss">
  .dashboard-link {
    font-size: 1.25rem;

    .button {
      display: flex;
      justify-content: space-between;
      padding: 0.25rem 0.5rem;
    }
  }

  .dashboard-subpaths-container {
    transform-origin: top;
    margin-top: 0.5rem;
    overflow: hidden;
    will-change: max-height;
  }

  .dashboard-link-hr {
    margin-bottom: 0.25rem;
  }

  .dashboard-subpaths {
    display: flex;
    flex-direction: column;
    margin-left: 0.25rem;
    font-size: 1rem;
    list-style: none;
  }
</style>
