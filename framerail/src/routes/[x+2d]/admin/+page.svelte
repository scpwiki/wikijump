<script lang="ts">
  import ForumSettings from "./ForumSettings.svelte"
  import LicenseSettings from "./LicenseSettings.svelte"
  import NavigationSettings from "./NavigationSettings.svelte"
  import RatingSettings from "./RatingSettings.svelte"
  import SiteIconSettings from "./SiteIconSettings.svelte"
  import SiteSettings from "./SiteSettings.svelte"
  import TemplateSettings from "./TemplateSettings.svelte"

  import type { PageProps } from "./$types"

  let { data }: PageProps = $props()
</script>

<SiteSettings {data} />
<NavigationSettings {data} />
<TemplateSettings {data} />
<SiteIconSettings {data} />
<ForumSettings {data} />
<RatingSettings {data} />
<LicenseSettings {data} />

<style global lang="scss">
  .site-info {
    padding: 0 0 2em;
  }

  .editor {
    display: flex;
    flex-direction: column;
    gap: 15px;
    align-items: stretch;
    justify-content: stretch;
    width: 100%;
  }

  .action-row {
    display: flex;
    flex-direction: row;
    gap: 10px;
    align-items: stretch;
    justify-content: flex-end;
    width: 100%;
  }

  .admin-section {
    margin-top: 2rem;
  }

  .checkbox-label {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .navigation-fields {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .navigation-fields.inherited {
    display: none;
  }

  .license-fields {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .license-fields.inherited {
    display: none;
  }

  .rating-fields {
    display: flex;
    flex-direction: column;
    gap: 15px;
  }

  .rating-fields.inherited,
  #page-rating-settings .default-category {
    display: none;
  }

  #sm-license-noind.default-category {
    display: none;
  }
</style>
