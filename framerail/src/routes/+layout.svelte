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

<style global lang="scss">
  .header-wjbanner {
    height: 80%;
    color: #fff;

    svg {
      height: 100%;
      width: auto;
    }
  }

  .footer-items {
    padding: 0;
    display: flex;
    flex-direction: row;
    justify-content: flex-start;
    align-items: center;
    gap: 10px;
    list-style: none;

    .footer-item a {
      text-decoration: none;
      color: #fff;
    }
  }
</style>
