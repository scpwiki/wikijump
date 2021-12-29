<!--
  @component Iconify powered inline icon.
-->
<script lang="ts">
  // load iconify along with this component
  import type { IconifyIcon } from "@iconify/iconify"
  import Iconify from "@iconify/iconify"

  /** Icon to display. Uses Iconify's format. */
  export let i = ""

  /** General size of the icon. `1em` should be close to a character in size. */
  export let size = "1em"

  /** String given to the CSS `margin` property. */
  export let margin = "0 0"

  let icon: IconifyIcon | null = null
  let viewBox = "0 0 0 0"

  $: if (i) {
    if (Iconify.iconExists(i)) {
      icon = Iconify.getIcon(i)
    } else {
      Iconify.loadIcons([i], () => {
        icon = Iconify.getIcon(i)
      })
    }
  }

  $: if (icon) {
    viewBox = `${icon.left ?? 0} ${icon.top ?? 0} ${icon.width ?? 0} ${icon.height ?? 0}`
  }
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  aria-hidden="true"
  focusable="false"
  {viewBox}
  style=" width: {size}; height: {size};margin: {margin}"
  {...$$restProps}
>
  {@html icon?.body ?? ""}
</svg>

<style global lang="scss">
  svg {
    vertical-align: middle;
    transform: rotate(360deg);
  }
</style>
