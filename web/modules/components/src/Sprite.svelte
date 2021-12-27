<!--
  @component Icon that uses Wikijump's `ui.svg` spritesheet.
-->
<script lang="ts">
  import { asset } from "@wikijump/api"

  /** Sprite to use. */
  export let i: string

  /** General size of the sprite. `1em` should be close to a character in size. */
  export let size = "1em"

  /** String given to the CSS `margin` property. */
  export let margin = "0 0"

  let viewBox = ""
  // prettier-ignore
  switch (i) {
    case "wj-karma": viewBox = "0 0 64 114" ; break;
    default:         viewBox = "0 0 24 24"  ; break;
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
