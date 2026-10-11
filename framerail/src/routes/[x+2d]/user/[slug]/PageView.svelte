<script lang="ts">
  import { normalizePublicWebsiteUrl } from "$lib/public-website-url.js"
  import type { userEditSchema } from "$lib/server/load/user"
  import type { InferOutput } from "valibot"
  import type { PageData } from "./$types"

  let {
    data,
    userData
  }: { data: PageData; userData: InferOutput<typeof userEditSchema> } = $props()

  // The stored avatar follows the server-confirmed user record, so a removal
  // that reloads the page data stops rendering it without a full navigation.
  let storedAvatar = $derived(
    data.user?.avatar_s3_hash
      ? `https://${data.site_file_domain}/-/avatar/${data.user.user_id}`
      : null
  )
  let previewAvatar = $state<string | null>(null)
  let avatar = $derived(previewAvatar ?? storedAvatar)
  let websiteHref = $derived(normalizePublicWebsiteUrl(userData?.website))

  $effect(() => {
    let url: string | undefined
    // If the user has edited their avatar, preview the new avatar
    if (userData?.avatar) {
      url = URL.createObjectURL(userData.avatar)
      previewAvatar = url
    } else {
      previewAvatar = null
    }

    return () => {
      if (url) {
        URL.revokeObjectURL(url)
      }
    }
  })
</script>

<svelte:head>
  <title>{data.user?.name ?? data.user?.slug} | {data?.site?.name}</title>
</svelte:head>

<div class="user-info" data-id={data.user?.user_id}>
  {#if userData?.name}
    <h1 class="user-attribute name">
      {userData.name}
    </h1>
  {/if}

  {#if userData?.realName}
    <div class="user-attribute real-name">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.real-name"]}</span
      >
      <span class="user-attribute-value">{userData.realName}</span>
    </div>
  {/if}

  {#if userData?.gender}
    <div class="user-attribute gender">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.gender"]}</span
      >
      <span class="user-attribute-value">{userData.gender}</span>
    </div>
  {/if}

  {#if avatar}
    <div class="user-attribute avatar">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.avatar"]}</span
      >
      <img
        class="user-attribute-value"
        alt={data.internationalization?.avatar}
        src={avatar}
      />
    </div>
  {/if}

  {#if userData?.birthday}
    <div class="user-attribute birthday">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.birthday"]}</span
      >
      <span class="user-attribute-value">{userData.birthday}</span>
    </div>
  {/if}

  {#if userData?.location}
    <div class="user-attribute location">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.location"]}</span
      >
      <span class="user-attribute-value">{userData.location}</span>
    </div>
  {/if}

  {#if userData?.website}
    <div class="user-attribute website">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.website"]}</span
      >
      {#if websiteHref}
        <a class="user-attribute-value website-link" href={websiteHref}
          >{userData.website}</a
        >
      {:else}
        <span class="user-attribute-value">{userData.website}</span>
      {/if}
    </div>
  {/if}

  {#if userData?.userPage}
    <div class="user-attribute user-page">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.user-page"]}</span
      >
      <span class="user-attribute-value">{userData.userPage}</span>
    </div>
  {/if}

  {#if userData?.biography}
    <div class="user-attribute biography">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.biography"]}</span
      >
      <span class="user-attribute-value">{userData.biography}</span>
    </div>
  {/if}

  {#if userData?.locales}
    <div class="user-attribute locales">
      <span class="user-attribute-label"
        >{data.internationalization?.["user-profile-info.locales"]}</span
      >
      <span class="user-attribute-value">{userData.locales.split(" ").join(", ")}</span>
    </div>
  {/if}
</div>
