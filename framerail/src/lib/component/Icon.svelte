<!--
  @component Iconify powered inline icon.
-->
<script lang="ts">
  // load iconify along with this component
  import { iconLoaded, getIcon, loadIcons, type IconifyIcon } from "iconify-icon"
  let {
    i,
    size = "1em",
    margin = "0 0",
    ...others
  }: {
    /** Icon to display. Uses Iconify's format. */
    i: string
    /**
     * General size of the icon. `1em` should be close to a character in
     * size.
     */
    size?: string
    /** String given to the CSS `margin` property. */
    margin?: string
    [key: string]: any
  } = $props()

  let icon = $state<Required<IconifyIcon> | null | undefined>(null)
  let viewBox = $derived(
    icon
      ? `${icon.left ?? 0} ${icon.top ?? 0} ${icon.width ?? 0} ${icon.height ?? 0}`
      : "0 0 0 0"
  )

  $effect(() => {
    if (i) {
      if (iconLoaded(i)) {
        icon = getIcon(i)
      } else {
        loadIcons([i], () => {
          icon = getIcon(i)
        })
      }
    }
  })
</script>

<svg
  style:width={size}
  style:height={size}
  style:margin
  aria-hidden="true"
  focusable="false"
  {viewBox}
  xmlns="http://www.w3.org/2000/svg"
  {...others}
>
  {@html icon?.body ?? ""}
</svg>

<style global lang="scss">
  svg {
    vertical-align: middle;
    transform: rotate(360deg);
  }
</style>
