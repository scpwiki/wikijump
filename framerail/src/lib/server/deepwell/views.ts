import { client } from "$lib/server/deepwell"

import type {
  Nullable,
  Optional,
  PageAttribution,
  PageModel,
  PageOptions,
  PageRevisionModel,
  SessionModel,
  SiteModel,
  UserModel
} from "$lib/types"

export interface Viewer {
  site: SiteModel
  site_file_domain: string
  license_name: string
  license_url: string
  user_session: Nullable<UserSession>
}

export interface WikidotPageSnapshotView {
  source_revision_count: number
  source_updated_at: string
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

export type PreloadDataAsync = () => Promise<Viewer & { locales: string[] }>

/* ----- Page View ----- */
export interface PageRoute {
  slug: Optional<string>
  extra: Optional<string>
}
interface PageViewDataBase {
  options: PageOptions
  redirect_page: Nullable<string>
  wikitext: string
  compiled_body_html: string
  compiled_top_bar_html: Optional<string>
  compiled_side_bar_html: Optional<string>
}

interface PageViewFound {
  type: "found"
  data: PageViewDataBase & {
    page: PageModel
    page_revision: PageRevisionModel
    wikidot_snapshot: Nullable<WikidotPageSnapshotView>
    attributions: PageAttribution[]
  }
}
interface PageViewMissing {
  type: "missing"
  data: PageViewDataBase
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

/* ----- Admin View ----- */
interface AdminViewSiteFound {
  type: "site_found"
  data: undefined
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
