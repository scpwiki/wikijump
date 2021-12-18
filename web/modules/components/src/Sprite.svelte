<!--
  @component Icon that uses Wikijump's `ui.svg` spritesheet.
-->
<script lang="ts">
  import { asset } from "@wikijump/api"

  /** Sprite to use. */
  export let i:
    | "wj-karma"
    | "wj-clipboard-copy"
    | "wj-clipboard-success"
    | "wj-downarrow"
    | "wj-hamburger"

  /** General size of the sprite. `1em` should be close to a character in size. */
  export let size = "1em"

  /** String given to the CSS `margin` property. */
  export let margin = "0 0"

  let viewBox = ""
  // prettier-ignore
  switch (i) {
    case "wj-karma":             viewBox = "0 0 64 114" ; break;
    case "wj-clipboard-copy":    viewBox = "0 0 24 24"  ; break;
    case "wj-clipboard-success": viewBox = "0 0 24 24"  ; break;
    case "wj-downarrow":         viewBox = "0 0 26 18"  ; break;
    case "wj-hamburger":         viewBox = "0 0 20 20"  ; break;
  }
</script>

<svg
  class="wj-ui-sprite sprite-{i}"
  width={size}
  style="margin: {margin};"
  {viewBox}
  {...$$restProps}
  aria-hidden="true"
>
  <use href="{asset('UI')}#{i}" />
</svg>
