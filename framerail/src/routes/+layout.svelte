<script lang="ts">
  import { page } from "$app/stores"
  import SigmaEsque from "$lib/sigma-esque/sigma-esque.svelte"
  import wjBanner from "$assets/logo-outline.min.svg?raw"
  import { useErrorPopup } from "$lib/stores"
  import ErrorPopup from "$lib/popup/error.svelte"
  let showErrorPopup = useErrorPopup()
  function closeErrorPopup() {
    showErrorPopup.set({
      state: false,
      message: null
    })
  }
</script>

{#if $showErrorPopup.state}
  <ErrorPopup exitPrompt={closeErrorPopup} />
{/if}

<SigmaEsque>
  <svelte:fragment slot="header">
    <div class="header-wjbanner">
      {@html wjBanner}
    </div>
  </svelte:fragment>

  <svelte:fragment slot="top-bar">UNTRANSLATED: Top bar</svelte:fragment>

  <svelte:fragment slot="content">
    <slot />
  </svelte:fragment>

  <svelte:fragment slot="footer">
    <div class="footer-inner">
      <ul class="footer-items">
        <li class="footer-item">
          <a href="/"
            >{$page.data?.internationalization?.terms ??
              $page.error?.internationalization?.terms}</a
          >
        </li>
        <li class="footer-item">
          <a href="/"
            >{$page.data?.internationalization?.privacy ??
              $page.error?.internationalization?.privacy}</a
          >
        </li>
        <li class="footer-item">
          <a href="/"
            >{$page.data?.internationalization?.docs ??
              $page.error?.internationalization?.docs}</a
          >
        </li>
        <li class="footer-item">
          <a href="/"
            >{$page.data?.internationalization?.security ??
              $page.error?.internationalization?.security}</a
          >
        </li>
      </ul>
      <div class="footer-powered-by">
        {$page.data?.internationalization?.["footer-powered-by"] ??
          $page.error?.internationalization?.["footer-powered-by"]}
      </div>
    </div>
  </svelte:fragment>
</SigmaEsque>

<!-- Ignoring the "unused" svg as we know we imported and embedded a raw svg -->
<!-- svelte-ignore css-unused-selector -->
<style global lang="scss">
  $tablet-max-width: 767px;

  .header-wjbanner {
    height: 80%;
    color: #fff;

    svg {
      width: auto;
      height: 100%;
    }
  }

  .footer-inner {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: center;
    justify-content: stretch;
    width: 100%;
  }

  .footer-items {
    display: flex;
    flex: 1;
    flex-direction: row;
    gap: 10px;
    align-items: center;
    justify-content: flex-start;
    padding: 0;
    list-style: none;

    .footer-item a {
      color: #fff;
      text-decoration: none;
    }
  }

  @media (max-width: $tablet-max-width) {
    .header-wjbanner {
      text-align: center;

      svg {
        height: initial;
        max-height: 6.5em;
      }
    }
  }
</style>
