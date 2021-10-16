<script lang="ts">
  import { cubicOut } from "svelte/easing"
  import { toasts } from "./lib/toasts"
  import Button from "./Button.svelte"
  import Icon from "./Icon.svelte"
  // import { t } from "@wikijump/api"

  $: listToasts = Array.from($toasts)

  const icons = {
    success: "fluent:checkmark-12-filled",
    danger: "ion:alert",
    warning: "ph:warning-bold",
    info: "ion:information"
  }

  function listTransition(_elem: Element, anim: { from: DOMRect; to: DOMRect }, _: any) {
    const d = anim.from.top - anim.to.top
    return {
      delay: d > 0 ? 0 : 300,
      duration: 250,
      easing: cubicOut,
      css: (_t: number, u: number) => `top: ${u * d}px`
    }
  }
</script>

<ul class="toasts" aria-live="polite" aria-relevant="additions">
  {#each listToasts as toast (toast)}
    <li class="toast is-{toast.type} dark" animate:listTransition>
      <span class="toast-type"><Icon i={icons[toast.type]} size="100%" /></span>
      {toast.message}
      <span class="toast-remove">
        <Button
          i="ion:close"
          size="1.5rem"
          tip="Close Notification"
          baseline
          on:click={toast.remove}
        />
      </span>
    </li>
  {/each}
</ul>

<style lang="scss">
  @import "../../../resources/css/abstracts";

  .toasts {
    position: absolute;
    right: 0;
    bottom: 0;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    padding: 0;
    margin: 2rem;

    // @include match("<=small") {
    //   align-items: center;
    //   width: 100%;
    //   margin: 1rem 0;
    // }
  }

  .toast {
    position: relative;
    width: fit-content;
    min-width: 20rem;
    max-width: 500px;
    padding: 0.5rem 3rem;
    margin: 0.5rem 0;
    list-style: none;
    border: solid 0.125rem var(--col-border);
    border-radius: 0.5rem;
    @include shadow(4);

    // @include match("<=small") {
    //   width: 90%;
    //   min-width: 0;
    //   max-width: none;
    // }
  }

  .toast-type,
  .toast-remove {
    position: absolute;
    top: 0;
    display: flex;
    align-items: center;
    justify-items: center;
    width: 2.25rem;
    height: 100%;
    padding: 0 0.25rem;
  }

  .toast-type {
    left: 0;
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
