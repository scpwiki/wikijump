<script lang="ts">
  import { router } from "tinro"
</script>

{#key $router.path}
  <slot />
{/key}
