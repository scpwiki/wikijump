import { client } from "$lib/server/deepwell"
import { Layout } from "$lib/types"

import type {
  Nullable,
  Optional,
  PageCategoryModel,
  PageRatingPermission,
  PageRatingType,
  PageRatingVisibility,
  SiteModel
} from "$lib/types"

type SiteUpdateRequestContext = {
  sessionToken: string
  siteId: number
}

interface CategoryNavigationUpdateInput {
  siteId: number
  categoryId: number
  userId: number
  userIpAddr: string
  topBarPage: Nullable<string>
  sideBarPage: Nullable<string>
}

export async function categoryNavigationUpdate(
  {
    siteId,
    categoryId,
    userId,
    userIpAddr,
    topBarPage,
    sideBarPage
  }: CategoryNavigationUpdateInput,
  requestContext: SiteUpdateRequestContext
): Promise<PageCategoryModel> {
  return client.request(
    "category_update",
    {
      site: siteId,
      category: categoryId,
      user_id: userId,
      top_bar_page: topBarPage,
      side_bar_page: sideBarPage,
      ip_address: userIpAddr
    },
    requestContext
  )
}

interface CategoryLicenseUpdateInput {
  siteId: number
  categoryId: number
  userId: number
  userIpAddr: string
  license: Nullable<string>
  licenseOther: Nullable<string>
}

export async function categoryLicenseUpdate(
  {
    siteId,
    categoryId,
    userId,
    userIpAddr,
    license,
    licenseOther
  }: CategoryLicenseUpdateInput,
  requestContext: SiteUpdateRequestContext
): Promise<PageCategoryModel> {
  return client.request(
    "category_update",
    {
      site: siteId,
      category: categoryId,
      user_id: userId,
      license,
      license_other: licenseOther,
      ip_address: userIpAddr
    },
    requestContext
  )
}

interface CategoryTemplateUpdateInput {
  siteId: number
  categoryId: number
  userId: number
  userIpAddr: string
  templatePageId: Nullable<number>
}

export async function categoryTemplateUpdate(
  { siteId, categoryId, userId, userIpAddr, templatePageId }: CategoryTemplateUpdateInput,
  requestContext: SiteUpdateRequestContext
): Promise<PageCategoryModel> {
  return client.request(
    "category_update",
    {
      site: siteId,
      category: categoryId,
      user_id: userId,
      template_page_id: templatePageId,
      ip_address: userIpAddr
    },
    requestContext
  )
}

export async function categoryRatingUpdate(
  siteId: number,
  categoryId: number,
  userId: number,
  userIpAddr: string,
  enabled: Nullable<boolean>,
  permission: Nullable<PageRatingPermission>,
  visibility: Nullable<PageRatingVisibility>,
  ratingType: Nullable<PageRatingType>,
  requestContext: SiteUpdateRequestContext
): Promise<PageCategoryModel> {
  return client.request(
    "category_update",
    {
      site: siteId,
      category: categoryId,
      user_id: userId,
      rating_enabled: enabled,
      rating_permission: permission,
      rating_visibility: visibility,
      rating_type: ratingType,
      ip_address: userIpAddr
    },
    requestContext
  )
}

export async function categoryDiscussionUpdate(
  siteId: number,
  categoryId: number,
  userId: number,
  userIpAddr: string,
  enabled: Nullable<boolean>,
  requestContext: SiteUpdateRequestContext
): Promise<PageCategoryModel> {
  return client.request(
    "category_update",
    {
      site: siteId,
      category: categoryId,
      user_id: userId,
      per_page_discussion: enabled,
      ip_address: userIpAddr
    },
    requestContext
  )
}

export async function siteForumNestingUpdate(
  siteId: number,
  userId: number,
  userIpAddr: string,
  maxNestLevel: number,
  requestContext: SiteUpdateRequestContext
): Promise<SiteModel> {
  return client.request(
    "site_update",
    {
      site: siteId,
      user_id: userId,
      forum_max_nest_level: maxNestLevel,
      ip_address: userIpAddr
    },
    requestContext
  )
}

export async function siteIconsUpdate(
  siteId: number,
  userId: number,
  userIpAddr: string,
  icons: {
    faviconSource: Nullable<string>
    iosIconSource: Nullable<string>
    windowsTileSource: Nullable<string>
  },
  requestContext: SiteUpdateRequestContext
): Promise<SiteModel> {
  return client.request(
    "site_update",
    {
      site: siteId,
      user_id: userId,
      favicon_source: icons.faviconSource,
      ios_icon_source: icons.iosIconSource,
      windows_tile_source: icons.windowsTileSource,
      ip_address: userIpAddr
    },
    requestContext
  )
}

interface SiteUpdateInput {
  siteId: number
  userId: number
  userIpAddr: string
  name: Optional<string>
  slug: Optional<string>
  tagline: Optional<string>
  description: Optional<string>
  defaultPage: Optional<string>
  locale: Optional<string>
  layout: Optional<Nullable<Layout>>
}

export async function siteUpdate(
  {
    siteId,
    userId,
    userIpAddr,
    name,
    slug,
    tagline,
    description,
    defaultPage,
    locale,
    layout
  }: SiteUpdateInput,
  requestContext: SiteUpdateRequestContext
): Promise<SiteModel> {
  return client.request(
    "site_update",
    {
      site: siteId,
      user_id: userId,
      name,
      slug,
      tagline,
      description,
      default_page: defaultPage,
      locale,
      layout:
        layout !== undefined
          ? (Layout[layout?.toUpperCase() as keyof typeof Layout] ?? null)
          : undefined,
      ip_address: userIpAddr
    },
    requestContext
  )
}
