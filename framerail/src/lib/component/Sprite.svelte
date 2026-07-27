<!--
  @component Icon that uses Wikijump's `ui.svg` spritesheet.
-->
<script lang="ts">
  let {
    i,
    size = "1em",
    margin = "0 0",
    ...others
  }: {
    /** Sprite to use. */
    i: string
    /**
     * General size of the sprite. `1em` should be close to a character in
     * size.
     */
    size?: string
    /** String given to the CSS `margin` property. */
    margin?: string
    [key: string]: any
  } = $props()

  let viewBox = $state("")

  $effect(() => {
    // prettier-ignore
    switch (i) {
      case "wj-karma": viewBox = "0 0 64 114" ; break;
      default:         viewBox = "0 0 24 24"  ; break;
    }
  })
</script>

<svg
  style:margin
  class="wj-ui-sprite sprite-{i}"
  {viewBox}
  width={size}
  {...others}
  aria-hidden="true"
>
  <use href="#{i}" />
</svg>
