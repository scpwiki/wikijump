<script lang="ts">
  import { format as t } from "@wikijump/fluent"
  import { router } from "tinro"
  $: current = $router.path === "/" ? "Home" : $router.path.slice(1)
</script>

<div class="route-announcer" aria-live="assertive" aria-atomic="true">
  {#key current}
    {t("navigated-to", { path: current })}
  {/key}
</div>

<style global lang="scss">
  .route-announcer {
    @include hide-visually;
  }
</style>
