import { client } from "$lib/server/deepwell"

import type {
  Nullable,
  Optional,
  PageAttribution,
  PageCategoryModel,
  PageModel,
  PageOptions,
  PageRevisionModel,
  SessionModel,
  SiteModel,
  SiteSettings,
  ThemeSetting,
  UserModel
} from "$lib/types"

export interface Viewer {
  site: SiteModel
  site_settings: SiteSettings
  promoted_sites: PromotedSiteView[]
  site_file_domain: string
  license_name: string
  license_url: string
  license_kind: "standard" | "other" | "copyright"
  license_html: Nullable<string>
  user_session: Nullable<UserSession>
}

export interface PromotedSiteView {
  slug: string
  name: string
  description: string
}

export interface WikidotPageSnapshotView {
  source_site: string
  source_revision_count: number
  source_updated_at: string
  imported_rating: Nullable<number>
  comments: Nullable<number>
}

export interface WikidotPageBreadcrumbView {
  slug: string
  title: string
}

/* ----- Preload ----- */

export async function preloadView(
  siteId: number,
  locales: string[],
  sessionToken: Optional<string>
): Promise<Viewer> {
  return client.request("preload_view", {
    site_id: siteId,
    locales,
    session_token: sessionToken
  })
}

export interface ClientUserSession {
  user: Partial<UserModel>
}

export interface PreloadData {
  site: Viewer["site"]
  site_settings: Viewer["site_settings"]
  promoted_sites: Viewer["promoted_sites"]
  site_file_domain: Viewer["site_file_domain"]
  license_name: Viewer["license_name"]
  license_url: Viewer["license_url"]
  license_kind: Viewer["license_kind"]
  license_html: Viewer["license_html"]
  user_session: Nullable<ClientUserSession>
  locales: string[]
}

export type PreloadDataAsync = () => Promise<PreloadData>

/* ----- Page View ----- */
export interface PageRoute {
  slug: Optional<string>
  extra: Optional<string>
}
interface PageViewDataBase {
  options: PageOptions
  redirect_page: Nullable<string>
  redirect_kind?: Nullable<"wikidot_module">
  wikitext: string
  compiled_body_html: string
  compiled_body_styles: string[]
  compiled_top_bar_html: Optional<string>
  compiled_side_bar_html: Optional<string>
  theme: ThemeSetting
}

export interface PageMetaTagView {
  name: string
  content: string
  all_pages: boolean
}

export type LegacyBrowserAction =
  | { type: "edit" }
  | { type: "history" }
  | { type: "source" }
  | { type: "print" }
  | { type: "set-tags"; index: number; fingerprint: string }

export type RateBrowserAction =
  | { type: "rate"; index: number; fingerprint: string; value: -1 | 1 | 2 | 3 | 4 | 5 }
  | { type: "rate-cancel"; index: number; fingerprint: string }

export interface RateBrowserActionRegistry {
  site_id: number
  page_id: number
  revision_id: number
  current_value: Nullable<number>
  actions: RateBrowserAction[]
}

export type MembershipBrowserAction =
  | {
      type: "join"
      page_id: number
      revision_id: number
      index: number
      fingerprint: string
    }
  | {
      type: "application"
      page_id: number
      revision_id: number
      index: number
      fingerprint: string
    }
  | {
      type: "password"
      page_id: number
      revision_id: number
      index: number
      fingerprint: string
    }

interface PageViewFound {
  type: "found"
  data: PageViewDataBase & {
    page: PageModel
    page_revision: PageRevisionModel
    wikidot_snapshot: Nullable<WikidotPageSnapshotView>
    wikidot_breadcrumbs: WikidotPageBreadcrumbView[]
    attributions: PageAttribution[]
    page_rating: PageRatingSettings
    page_discussion: PageDiscussionSettings
    data_form: Nullable<DataFormEditor>
    legacy_actions: LegacyBrowserAction[]
    rate_actions: Nullable<RateBrowserActionRegistry>
    membership_actions: MembershipBrowserAction[]
    /**
     * Deepwell recognizes an executable, non-literal ThemePreviewer noUi
     * path.
     */
    theme_previewer_no_ui?: boolean
    /** Source-owned blank ThemePreviewer selection on a saved page. */
    theme_previewer_blank?: boolean
    meta_tags: PageMetaTagView[]
  }
}

export interface PageRatingSettings {
  enabled: boolean
  permission: "registered" | "members"
  visibility: "visible" | "anonymous"
  rating_type: "plus" | "plus_minus" | "stars"
}
export interface PageDiscussionSettings {
  enabled: boolean
}
export interface DataFormValueDefinition {
  value: string
  label: string
}
export interface DataFormFieldDefinition {
  name: string
  label: string
  hint: string
  field_type: Nullable<string>
  values: DataFormValueDefinition[]
  default_value: Nullable<string>
  configured_value: Nullable<string>
  options: Record<string, unknown>
  width: number
  height: number
  match_pattern: Nullable<string>
  match_error: Nullable<string>
  before: string
  after: string
  join: boolean
  pagepath_category: Nullable<string>
  pagepath_max_level: Nullable<number>
}
export interface DataFormPagepathNode {
  fullname: string
  name: string
  parent: Nullable<string>
}
export interface DataFormDefinition {
  fields: DataFormFieldDefinition[]
  default_layout: boolean
}
export interface DataFormEditor {
  definition: DataFormDefinition
  values: Record<string, string>
  pagepaths: Record<string, DataFormPagepathNode[]>
}
interface PageViewMissing {
  type: "missing"
  data: PageViewDataBase & {
    new_page_wikitext: Nullable<string>
    page_templates: PageTemplateSummary[]
    selected_template_page_id: Nullable<number>
    data_form: Nullable<DataFormEditor>
    /**
     * Deepwell's create decision for this viewer (the same authority
     * page_create enforces). Template and data-form source is only present
     * when this is true.
     */
    can_create: boolean
  }
}
interface PageViewPermissions {
  type: "permissions"
  data: PageViewDataBase & {
    banned: boolean
  }
}

export type PageView = PageViewFound | PageViewMissing | PageViewPermissions

export async function pageView(
  siteId: number,
  locales: string[],
  route: Nullable<PageRoute>,
  sessionToken: Optional<string>
): Promise<PageView> {
  return client.request("page_view", {
    site_id: siteId,
    locales,
    session_token: sessionToken,
    route
  })
}

export type ArticleView = Viewer & {
  page: PageView
  article_page_cache_key: Optional<string>
  public_content_cache_fence: Optional<string>
  anonymous_permission_cache_fence: Optional<string>
}

export async function articleView(
  siteId: number,
  locales: string[],
  route: Nullable<PageRoute>,
  sessionToken: Optional<string>
): Promise<ArticleView> {
  return client.request("article_view", {
    site_id: siteId,
    locales,
    session_token: sessionToken,
    route
  })
}

export interface ArticleViewCacheMetadata {
  article_page_cache_key: Optional<string>
  public_content_cache_fence: Optional<string>
  anonymous_permission_cache_fence: Optional<string>
}

export async function articleViewCacheMetadata(
  siteId: number,
  locales: string[],
  route: Nullable<PageRoute>
): Promise<ArticleViewCacheMetadata> {
  return client.request("article_view_cache_metadata", {
    site_id: siteId,
    locales,
    session_token: null,
    route
  })
}

/* ----- Admin View ----- */
interface AdminViewSiteFound {
  type: "site_found"
  data: {
    categories: PageCategoryModel[]
    page_templates: PageTemplateSummary[]
    is_master_admin: boolean
  }
}

export interface PageTemplateSummary {
  page_id: number
  slug: string
  title: string
  wikitext: string
}
interface AdminViewAdminPermissions {
  type: "admin_permissions"
  data: {
    html: string
  }
}

interface UserSession {
  session: SessionModel
  user: UserModel
}

export async function adminView(
  siteId: number,
  locales: string[],
  sessionToken: Optional<string>
): Promise<AdminViewSiteFound | AdminViewAdminPermissions> {
  return client.request("admin_view", {
    site_id: siteId,
    locales,
    session_token: sessionToken
  })
}
