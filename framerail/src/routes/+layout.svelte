<script lang="ts">
  import SigmaEsque from "$lib/sigma-esque/sigma-esque.svelte"
  import wjBanner from "$lib/assets/wikijump-banner-solid.min.svg?raw"
</script>

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
    <ul class="footer-items">
      <li class="footer-item">
        <a href="/">UT:Terms</a>
      </li>
      <li class="footer-item">
        <a href="/">UT:Privacy</a>
      </li>
      <li class="footer-item">
        <a href="/">UT:Docs</a>
      </li>
      <li class="footer-item">
        <a href="/">UT:Security</a>
      </li>
    </ul>
  </svelte:fragment>
</SigmaEsque>

<!-- Ignoring the "unused" svg as we know we imported and embedded a raw svg -->
<!-- svelte-ignore css-unused-selector -->
<style global lang="scss">
  .header-wjbanner {
    height: 80%;
    color: #fff;

    svg {
      width: auto;
      height: 100%;
    }
  }

  .footer-items {
    display: flex;
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
</style>
