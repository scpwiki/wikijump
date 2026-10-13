import type { Locales } from "../types"
export type Optional<T> = T | undefined
export type Nullable<T> = T | null
export type JsonValue =
  string | number | boolean | null | { [key: string]: JsonValue } | JsonValue[]
export type TranslateKeys = {
  [P in keyof Locales]?: Record<string, string | number>
}
export type TranslatedKeys = Partial<Locales>

// deepwell src/models/site.rs
export interface SiteModel {
  site_id: number
  created_at: string
  updated_at: Nullable<string>
  deleted_at: Nullable<string>
  from_wikidot: boolean
  slug: string
  name: string
  tagline: string
  description: string
  locale: string
  default_page: string
  top_bar_page: Nullable<string>
  side_bar_page: Nullable<string>
  preferred_domain: Nullable<string>
  layout: Nullable<Layout>
  license: License
  forum_max_nest_level: number
  favicon_source: Nullable<string>
  ios_icon_source: Nullable<string>
  windows_tile_source: Nullable<string>
  settings_revision: number
  welcome_page: string
  google_analytics_enabled: boolean
  google_analytics_profile: Nullable<string>
  show_top_toolbar: boolean
  show_bottom_toolbar: boolean
  promote_on_other_sites: boolean
  educational: boolean
}

// deepwell src/models/page_category.rs
export interface PageCategoryModel {
  category_id: number
  created_at: string
  updated_at: Nullable<string>
  site_id: number
  slug: string
  layout: Nullable<Layout>
  top_bar_page: Nullable<string>
  side_bar_page: Nullable<string>
  template_page_id: Nullable<number>
  license: Nullable<string>
  license_other: Nullable<string>
  rating_enabled: Nullable<boolean>
  rating_permission: Nullable<PageRatingPermission>
  rating_visibility: Nullable<PageRatingVisibility>
  rating_type: Nullable<PageRatingType>
  per_page_discussion: Nullable<boolean>
  settings_revision: number
  theme_kind: "inherit" | "built_in" | "external" | "custom"
  theme_builtin_id: Nullable<number>
  theme_external_url: Nullable<string>
  theme_custom_css: Nullable<string>
  autonumber_enabled: boolean
  autonumber_next: number
}

export interface SiteSettings {
  revision: number
  welcome_page: string
  google_analytics: {
    enabled: boolean
    profile: Nullable<string>
  }
  toolbars: {
    top: boolean
    bottom: boolean
    promote: boolean
  }
  membership: {
    application_enabled: boolean
    password_enabled: boolean
    password_configured: boolean
  }
}

export type ThemeSetting =
  | { type: "inherit" }
  | { type: "built_in"; id: number }
  | { type: "external"; url: string }
  | { type: "custom"; css: string }

export type PageRatingPermission = "registered" | "members"
export type PageRatingVisibility = "visible" | "anonymous"
export type PageRatingType = "plus" | "plus_minus" | "stars"

// deepwell src/models/session.rs
export interface SessionModel {
  session_token: string
  user_id: number
  created_at: string
  expires_at: string
  ip_address: string
  user_agent: string
  restricted: boolean
}

// deepwell src/models/user.rs
export interface UserModel {
  user_id: number
  user_type: UserType
  created_at: string
  updated_at: Nullable<string>
  deleted_at: Nullable<string>
  from_wikidot: boolean
  name: string
  slug: string
  name_changes_left: number
  last_name_change_added_at: string
  last_renamed_at: Nullable<string>
  email: string
  email_verified_at: Nullable<string>
  email_validation_info: Nullable<JsonValue>
  email_validation_at: Nullable<string>
  password: string
  multi_factor_secret: Nullable<string>
  multi_factor_recovery_codes: Nullable<string[]>
  locales: string[]
  avatar_s3_hash: Nullable<number[]>
  forum_signature: Nullable<string>
  real_name: Nullable<string>
  gender: Nullable<string>
  birthday: Nullable<string>
  location: Nullable<string>
  biography: Nullable<string>
  website: Nullable<string>
  user_page: Nullable<string>
}

// deepwell src/models/wikidot_user.rs plus services/user/structs.rs discriminator
export interface WikidotUserModel {
  user_id: number
  user_type: "wikidot"
  created_at: string
  fetched_at: string
  is_deleted: boolean
  name: Nullable<string>
  slug: Nullable<string>
  avatar_s3_hash: Nullable<number[]>
  real_name: Nullable<string>
  gender: Nullable<string>
  birthday: Nullable<string>
  location: Nullable<string>
  biography: Nullable<string>
  website: Nullable<string>
  karma: number
  is_pro: boolean
}

// deepwell src/models/page.rs
export interface PageModel {
  page_id: number
  created_at: string
  updated_at: Nullable<string>
  deleted_at: Nullable<string>
  from_wikidot: boolean
  site_id: number
  latest_revision_id: Nullable<number>
  page_category_id: number
  slug: string
  discussion_thread_id: Nullable<number>
  layout: Nullable<Layout>
}

// deepwell src/models/page_revision.rs
export interface PageRevisionModel {
  revision_id: number
  revision_type: PageRevisionType
  created_at: string
  updated_at: Nullable<string>
  revision_number: number
  page_id: number
  site_id: number
  user_id: number
  from_wikidot: boolean
  changes: string[]
  wikitext_hash: number[]
  compiled_body_html_hash: number[]
  compiled_top_bar_html_hash: Nullable<number[]>
  compiled_side_bar_html_hash: Nullable<number[]>
  compiled_at: string
  compiled_generator: string
  comments: string
  hidden: string[]
  title: string
  alt_title: Nullable<string>
  slug: string
  tags: string[]
}

// deepwell file_revision_range (with the page timeline's public author
// projection attached, issue #2199)
export interface FileRevisionModel {
  revision_id: number
  revision_type: FileRevisionType
  created_at: string
  revision_number: number
  file_id: number
  page_id: number
  site_id: number
  user_id: number
  author: Nullable<UserInfo>
  name: Nullable<string>
  s3_hash: Nullable<number[]>
  mime: Nullable<string>
  size: Nullable<number>
  changes: string[]
  comments: Nullable<string>
  hidden: string[]
}

// deepwell src/models/page_vote.rs
export interface PageVoteModel {
  page_vote_id: number
  created_at: string
  deleted_at: Nullable<string>
  disabled_at: Nullable<string>
  disabled_by: Nullable<number>
  from_wikidot: boolean
  page_id: number
  user_id: number
  rating_system: "points" | "stars"
  value: number
}

// deepwell src/services/view/options.rs
export interface PageOptions {
  edit: boolean
  title: Nullable<string>
  parent: Nullable<string>
  tags: Nullable<string>
  no_redirect: boolean
  no_render: boolean
  debug: boolean
  renderer: boolean
  comments: boolean
  history: boolean
  offset: Nullable<number>
  data: string
  template: Nullable<number>
}

// deepwell src/services/relation/page_attribution.rs
export interface PageAttribution {
  relation_id: number
  page_id: number
  user_id: number
  created_by: number
  created_at: string
  overwritten_by: Nullable<number>
  overwritten_at: Nullable<string>
  deleted_by: Nullable<number>
  deleted_at: Nullable<string>
  metadata: PageAttributionMetadata
}
export interface PageAttributionMetadata {
  attribution_type: PageAttributionKind
  attribution_date: string
}
export enum PageAttributionKind {
  Author = "author",
  Rewrite = "rewrite",
  Translator = "translator",
  Maintainer = "maintainer"
}

/* ftml src/parsing/error.rs */
// we don't care about the enum types now
export interface ParseError {
  // should be `token: Token`, an enum type
  token: string
  rule: string
  span: [number, number]
  // should be `kind: ParseErrorKind`, also an enum type
  kind: string
}

// ftml src/data/user_info.rs
export interface UserInfo {
  "user-id": number
  "user-slug": string
  "user-name": string
  "user-karma": number
  "user-avatar-data": string
  "user-profile-url": string
}

export enum Layout {
  WIKIDOT = "wikidot",
  WIKIJUMP = "wikijump"
}
export enum PageLockType {
  Wikidot = "wikidot",
  PermissionOnly = "permission-only",
  AuthorOrPermissionOnly = "author-or-permission-only"
}
export enum PagePane {
  None = "none",
  Append = "append",
  Backlinks = "backlinks",
  File = "file",
  History = "history",
  Tags = "tags",
  Layout = "layout",
  Move = "move",
  Parent = "parent",
  Watchers = "watchers",
  SiteTools = "site-tools",
  Vote = "vote",
  Delete = "delete",
  EditMeta = "edit-meta",
  Lock = "lock"
}
export enum UserType {
  Regular = "regular",
  System = "system",
  Site = "site",
  Bot = "bot"
}
export enum License {
  CcBySa40 = "cc-by-sa-4.0",
  CcBy40 = "cc-by-4.0",
  CcByNd40 = "cc-by-nd-4.0",
  CcByNc40 = "cc-by-nc-4.0",
  CcByNcSa40 = "cc-by-nc-sa-4.0",
  CcByNcNd40 = "cc-by-nc-nd-4.0",
  CcBySa30 = "cc-by-sa-3.0",
  CcBy30 = "cc-by-3.0",
  CcByNd30 = "cc-by-nd-3.0",
  CcByNc30 = "cc-by-nc-3.0",
  CcByNcSa30 = "cc-by-nc-sa-3.0",
  CcByNcNd30 = "cc-by-nc-nd-3.0",
  CcBySa25 = "cc-by-sa-2.5",
  CcBy25 = "cc-by-2.5",
  CcByNd25 = "cc-by-nd-2.5",
  CcByNc25 = "cc-by-nc-2.5",
  CcByNcSa25 = "cc-by-nc-sa-2.5",
  CcByNcNd25 = "cc-by-nc-nd-2.5",
  GnuFdl13 = "gnu-fdl-1.3",
  GnuFdl12 = "gnu-fdl-1.2",
  GnuFdl11 = "gnu-fdl-1.1",
  Cc0 = "cc0"
}
export enum PageRevisionType {
  Regular = "regular",
  Rollback = "rollback",
  Undo = "undo",
  Create = "create",
  Delete = "delete",
  Undelete = "undelete",
  Move = "move"
}
export enum FileRevisionType {
  Regular = "regular",
  Rollback = "rollback",
  Create = "create",
  Delete = "delete",
  Undelete = "undelete",
  Move = "move"
}
export enum DeleteOptions {
  Move = "move",
  Delete = "delete"
}
export enum ToastType {
  Success = "success",
  Danger = "danger",
  Warning = "warning",
  Info = "info"
}
// JSON-RPC 2.0 error object as thrown by json-rpc-2.0 client
export interface DeepwellError {
  message: string
  code: number
  data?: JsonValue
}
