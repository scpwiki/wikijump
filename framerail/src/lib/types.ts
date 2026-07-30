import type { Locales } from "../types"
export type Optional<T> = T | undefined
export type Nullable<T> = T | null
export type JsonValue =
  | string
  | number
  | boolean
  | null
  | { [key: string]: JsonValue }
  | JsonValue[]
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
  preferred_domain: Nullable<string>
  layout: Nullable<Layout>
  license: License
}

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
  real_name: Nullable<string>
  gender: Nullable<string>
  birthday: Nullable<string>
  location: Nullable<string>
  biography: Nullable<string>
  website: Nullable<string>
  user_page: Nullable<string>
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

// deepwell src/models/file_revision.rs
export interface FileRevisionModel {
  revision_id: number
  revision_type: FileRevisionType
  created_at: string
  revision_number: number
  file_id: number
  page_id: number
  site_id: number
  user_id: number
  name: string
  s3_hash: number[]
  mime: string
  size: number
  changes: string[]
  comments: string
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
  File = "file",
  History = "history",
  Layout = "layout",
  Move = "move",
  Parent = "parent",
  Vote = "vote",
  Delete = "delete",
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
// JSON-RPC 2.0 error object as thrown by json-rpc-2.0 client
export interface DeepwellError {
  message: string
  code: number
  data?: JsonValue
}
