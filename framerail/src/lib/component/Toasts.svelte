<!--
  @component Toasts handler/renderer. Needs to be inserted in the `#toasts` div.
-->
<script lang="ts">
  import { format as t } from "$lib/util/locale"
  import Button from "./Button.svelte"
  import Sprite from "./Sprite.svelte"
  import { anim } from "./scripts/animation"
  import { toasts } from "./scripts/toasts"

  let listToasts = $derived(Array.from($toasts))

  const icons = {
    success: "wj-checkmark",
    danger: "wj-exclamation",
    warning: "wj-warning",
    info: "wj-info"
  }

  // what these animations do:
  // each toast container has a transition for margin-top and height
  // these start at 0, which means that they don't affect the positions
  // of the other toasts
  // intro: set height and margin-top to their "correct" values
  // outro: set height and margin-top to 0
  // this causes the toasts to gracefully reposition as the list changes

  function intro(evt: any) {
    const node = evt.currentTarget as HTMLElement
    const toast = node.children[0] as HTMLElement
    const height = toast.getBoundingClientRect().height
    node.style.height = `${height}px`
    node.style.marginTop = "1rem"
  }

  function outro(evt: any) {
    // last toast can have a weird animation, so we skip it to avoid that
    if (listToasts.length !== 0) {
      const node = evt.currentTarget as HTMLElement
      node.style.height = "0"
      node.style.marginTop = "0"
    }
  }
</script>

<ul class="toasts" aria-live="polite" aria-relevant="additions">
  {#each listToasts as toast (toast)}
    <li
      class="toast-block"
      onintrostart={intro}
      onoutrostart={outro}
      transition:anim={{
        duration: 500,
        easing: "quartOut",
        css: (_, u) => `transform: translateX(${u * 150}%);`
      }}
    >
      <div class="toast is-{toast.type} dark">
        <span class="toast-type"><Sprite i={icons[toast.type]} size="75%" /></span>
        <span class="toast-message">{toast.message}</span>
        <span class="toast-remove">
          <Button
            baseline
            i="wj-close"
            onclick={toast.remove}
            size="1.5rem"
            tip={t("close")}
          />
        </span>
      </div>
    </li>
  {/each}
</ul>

<style global lang="scss">
  @use "../css/abstracts" as *;

  .toasts {
    position: absolute;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    padding: 0;
    margin: 2rem;

    @include media("<=small") {
      align-items: center;
      width: 100%;
      margin: 1rem 0;
    }
  }

  .toast-block {
    position: relative;
    height: 0;
    margin-top: 0;
    list-style: none;
    transition:
      height 250ms 75ms,
      margin-top 250ms 75ms;
  }

  .toast {
    position: relative;
    width: fit-content;
    min-width: 20rem;
    max-width: 500px;
    padding: 0.5rem 3rem;
    border: solid 0.125rem var(--col-border);
    border-radius: 0.5rem;
    @include shadow(4);

    @include media("<=small") {
      width: 90%;
      min-width: 0;
      max-width: none;
    }
  }

  .toast-type,
  .toast-remove {
    position: absolute;
    top: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 2.25rem;
    height: 100%;
    padding: 0 0.25rem;
  }

  .toast-type {
    left: 0;
  }

  .toast-message {
    padding: 0 0.5rem;
  }

  .toast-remove {
    right: 0.25rem;
  }

  .toast.is-success {
    border-color: var(--col-success);
    .toast-type {
      background: var(--col-success);
    }
  }

  .toast.is-danger {
    border-color: var(--col-danger);
    .toast-type {
      background: var(--col-danger);
    }
  }

  .toast.is-warning {
    border-color: var(--col-warning);
    .toast-type {
      background: var(--col-warning);
    }
  }

  .toast.is-info {
    border-color: var(--col-info);
    .toast-type {
      background: var(--col-info);
    }
  }
</style>
