/* eslint-disable */
/* tslint:disable */
/*
 * ---------------------------------------------------------------
 * ## THIS FILE WAS GENERATED VIA SWAGGER-TYPESCRIPT-API        ##
 * ##                                                           ##
 * ## AUTHOR: acacode                                           ##
 * ## SOURCE: https://github.com/acacode/swagger-typescript-api ##
 * ---------------------------------------------------------------
 */

/** Describes the pagination property present with all paginated responses. */
export interface Paginated {
  pagination: { cursor: number; limit: number; pages: number }
}

export enum ReferenceTypes {
  User = "user",
  Page = "page",
  Message = "message",
  File = "file",
  Report = "report",
  Abuse = "abuse",
  ForumGroup = "forum-group",
  ForumCategory = "forum-category",
  ForumThread = "forum-thread",
  ForumPost = "forum-post"
}

/**
 * Describes a page *slug*, a string consisting of an optional category and
 * name. It is formatted as `category:name` if a category is included. If a
 * category is not included, it is simply `name`.
 *
 * @example Category:page
 *
 * @format slug
 */
export type Slug = string

/** @example Mywiki */
export type SiteName = string

/** @example ExampleUsername */
export type Username = string

/** @format email */
export type Email = string

/**
 * An integer that uniquely points to a resource.
 *
 * @example 1234
 */
export type Reference = number

/**
 * A binary chunk of data representing a file.
 *
 * @example FFD8FFDB00430006040506050406060506070706...
 *
 * @format binary
 */
export type FileData = File

/**
 * A base64 encoded chunk of data.
 *
 * @example Y3VyaW91cyBhcmVuJ3QgeW91
 *
 * @format byte
 */
export type Base64 = string

/**
 * A chunk of text in FTML format.
 *
 * @example [[div]]/some wikitext/[[/div]]
 *
 * @format ftml
 */
export type Wikitext = string

/**
 * A chunk of text in HTML format.
 *
 * @example <div><i>some html</i></div>
 *
 * @format html
 */
export type HTML = string

export type LoginSpecifier = Email | Username

/** Private account settings that can govern some of Wikijump's behavior. */
export interface AccountSettings {
  acceptsInvites: boolean
  language: string
  allowMessages: "registered" | "co-members" | "nobody"
}

/** Private account settings that can govern some of Wikijump's behavior. */
export interface AccountSettingsPatch {
  acceptsInvites?: boolean
  language?: string
  allowMessages?: "registered" | "co-members" | "nobody"
}

/** Describes a notification intended to inform a user of some sort of event. */
export interface Notification {
  level: "trivial" | "info" | "important" | "error"
  type: "account" | "pm" | "site" | "forum" | "page" | "other"
  name: string
  source: string

  /** @format date-time */
  time: string
  payload: { message: string }
}

export interface NotificationList {
  notifications: Notification[]
}

/** Describes a user's administrative role and membership status. */
export enum UserRole {
  Guest = "guest",
  Registered = "registered",
  Member = "member",
  Moderator = "moderator",
  Admin = "admin",
  MasterAdmin = "master-admin",
  PlatformAdmin = "platform-admin"
}

/** Basic level of information needed to describe a user. */
export interface UserIdentity {
  /** An integer that uniquely points to a resource. */
  id: Reference
  username: Username
  tinyavatar: Base64 | null

  /**
   * @min 0
   * @max 5
   */
  karma: number

  /** Describes a user's administrative role and membership status. */
  role: UserRole
}

/** Describes a user in an intermediate amount of detail. */
export type UserInfo = UserIdentity & {
  about: string
  avatar: string | null
  signature: HTML | null
  since: string
  lastActive: string
  blocked: boolean
}

/** Fully describes a user and their personalization preferences. */
export type UserProfile = UserInfo & {
  realname: string
  pronouns: string | null
  birthday: string | null
  location: string | null
  links: Record<string, string>
}

/** Partial object that is used to update a user's profile. */
export interface UserProfilePatch {
  about?: string

  /** A chunk of text in FTML format. */
  signature?: Wikitext
  gender?: string

  /** @format date */
  birthday?: string
  location?: string
  links?: Record<string, string>
}

export interface UserBlockedList {
  users: UserIdentity[]
}

export interface Membership {
  site: SiteName

  /** Describes a user's administrative role and membership status. */
  role: UserRole
}

export interface MembershipList {
  memberships: Membership[]
}

export interface MembershipRole {
  role: "member" | "moderator" | "admin"
}

export interface Application {
  /** An integer that uniquely points to a resource. */
  id?: Reference

  /** Basic level of information needed to describe a user. */
  sender: UserIdentity
  message: string

  /** @format date-time */
  time: string
}

export type ApplicationList = Paginated & { applications: Application[] }

export interface ApplicationSend {
  message: string
}

export interface ApplicationSendList {
  applications: { site: SiteName; message: string }[]
}

export interface Invite {
  /** Basic level of information needed to describe a user. */
  sender: UserIdentity
  site: SiteName
  messsage?: string

  /** @format date-time */
  time: string
}

export interface InviteList {
  invites: Invite[]
}

export interface InviteSend {
  site: SiteName
  message: string
}

/**
 * Represents an FTML syntax tree.
 *
 * @format ftmltree
 */
export type FTMLSyntaxTree = Record<string, any>

export interface WikitextObj {
  /** A chunk of text in FTML format. */
  wikitext: Wikitext
}

export interface HTMLObj {
  /** A chunk of text in HTML format. */
  html: HTML
}

export interface Page {
  /** An integer that uniquely points to a resource. */
  id: Reference

  /**
   * Describes a page *slug*, a string consisting of an optional category
   * and name. It is formatted as `category:name` if a category is
   * included. If a category is not included, it is simply `name`.
   */
  slug: Slug

  /** An integer that uniquely points to a resource. */
  category: Reference
  parent: Slug | null
  children: Slug[]
  title: string
  tags: TagList
  score: number
  revision: number

  /** @format date-time */
  created: string

  /** Basic level of information needed to describe a user. */
  creator: UserIdentity

  /** @format date-time */
  updated: string

  /** Basic level of information needed to describe a user. */
  updater: UserIdentity

  /** A chunk of text in HTML format. */
  html?: HTML

  /** A chunk of text in FTML format. */
  wikitext?: Wikitext
}

export interface Revision {
  revision: number

  /** @format date-time */
  updated: string

  /** Basic level of information needed to describe a user. */
  updater: UserIdentity
  hidden: boolean
  message: string
  flags: ("created" | "content" | "file" | "title" | "revert" | "tag" | "slug")[]
}

export type RevisionHistory = Paginated & { revisions: number; history: Revision[] }

export type TagList = string[]

export type CastVotePlus = 0 | 1

export type CastVotePlusMinus = -1 | 0 | 1

export type CastVoteStar = 1 | 2 | 3 | 4 | 5

export type CastVote = CastVotePlusMinus | CastVoteStar

/**
 * Describes the score/rating of a page.
 *
 * Wikijump has three different ways of rating a page:
 *
 * - `plus`
 * - `plusminus`
 * - `star`
 *
 * You will find the format used in the `format` property.
 */
export type Score =
  | { format: "plus"; score: number; count: number; totals: { "0": number; "1": number } }
  | {
      format: "plusminus"
      score: number
      count: number
      totals: { "0": number; "1": number; "-1": number }
    }
  | {
      format: "star"
      score: number
      count: number
      totals: { "1": number; "2": number; "3": number; "4": number; "5": number }
    }

export type Vote =
  | { format: "plus"; time: string; vote: CastVotePlus }
  | { format: "plusminus"; time: string; vote: CastVotePlusMinus }
  | { format: "star"; time: string; vote: CastVoteStar }

export type VoterList = (Paginated & Score) & { voters: Vote[] }

/** A file MIME type and description. */
export interface Mime {
  type: string
  description: string
}

export interface FileMetadata {
  /** An integer that uniquely points to a resource. */
  id: Reference

  /** @example 20kb */
  size: number
  comment: string

  /** A file MIME type and description. */
  mime: Mime

  /** Basic level of information needed to describe a user. */
  uploader: UserIdentity

  /** @format date-time */
  uploaded: string

  /** @format url */
  url: string
}

export interface FileSiteMetadata {
  max: number
  used: number
  count: number
  available: number
}

export interface FileUpload {
  filename: string
  comment: string

  /** A binary chunk of data representing a file. */
  content: FileData
}

export interface ReportSend {
  reason: string
}

export interface Report {
  /** An integer that uniquely points to a resource. */
  id: Reference

  /** An integer that uniquely points to a resource. */
  target?: Reference

  /** Basic level of information needed to describe a user. */
  sender: UserIdentity
  reason: string

  /** @format date-time */
  time: string
}

export interface ReportList {
  reports: Report[]
}

export interface Message {
  /** An integer that uniquely points to a resource. */
  id: Reference
  read: boolean
  archived?: boolean

  /** @format date-time */
  time: string

  /** Basic level of information needed to describe a user. */
  from: UserIdentity
  subject: string

  /** A chunk of text in HTML format. */
  html?: HTML
}

export type MessageList = Paginated & { messages: Message[] }

export interface MessageSend {
  subject: string

  /** A chunk of text in FTML format. */
  wikitext?: Wikitext
}

export enum ForumSortingTypes {
  Newest = "newest",
  Oldset = "oldset"
}

export interface ForumCreationContext {
  /** Basic level of information needed to describe a user. */
  by: UserIdentity

  /** @format date-time */
  time: string
}

export interface Forum {
  threadCount: number
  postCount: number
  groups: ForumGroup[]
}

export interface ForumGroup {
  /** An integer that uniquely points to a resource. */
  id: Reference
  title: string
  summary: string
  categories: ForumCategory[]
}

export interface ForumCategory {
  /** An integer that uniquely points to a resource. */
  id: Reference

  /** An integer that uniquely points to a resource. */
  group: Reference
  title: string
  summary: string
  threadCount: number
  postCount: number
  last: ForumCreationContext
  permissions?: {
    createPosts: ("guest" | "registered" | "member")[]
    createThreads: ("guest" | "registered" | "member")[]
    edit: ("guest" | "registered" | "member" | "author")[]
  }
}

export interface ForumThread {
  /** An integer that uniquely points to a resource. */
  id: Reference

  /** An integer that uniquely points to a resource. */
  category: Reference

  /** An integer that uniquely points to a resource. */
  group: Reference
  title: string
  stickied: boolean
  locked: boolean
  postCount: number
  created: ForumCreationContext
  last: ForumCreationContext
}

export interface ForumPost {
  /** An integer that uniquely points to a resource. */
  id: Reference

  /** An integer that uniquely points to a resource. */
  category: Reference

  /** An integer that uniquely points to a resource. */
  group: Reference

  /** An integer that uniquely points to a resource. */
  thread: Reference
  parent: Reference | null
  created: ForumCreationContext
  revision: number
  replyCount: number

  /** A chunk of text in HTML format. */
  html?: HTML

  /** A chunk of text in FTML format. */
  wikitext?: Wikitext
  replies?: ForumPostList
}

export interface ForumGroupList {
  groups: ForumGroup[]
}

export interface ForumCategoryList {
  categories: ForumCategory[]
}

export type ForumThreadList = Paginated & {
  order: ForumSortingTypes
  threads: ForumThread[]
}

export type ForumPostList = Paginated & { order: ForumSortingTypes; posts: ForumPost[] }

export interface Category {
  /** An integer that uniquely points to a resource. */
  id: Reference
  name: string
  license: string | null
  ratings: "disabled" | "plus" | "plusminus" | "star" | null
  discussions: boolean | null
  autonumber: boolean
  permissions: {
    createPages: ("guest" | "registered" | "member")[]
    renamePages: ("guest" | "registered" | "member" | "creator")[]
    deletePages: ("guest" | "registered" | "member" | "creator")[]
    uploadFiles: ("guest" | "registered" | "member" | "creator")[]
    changeFiles: ("guest" | "registered" | "member" | "creator")[]
    showOptions: ("guest" | "registered" | "member" | "creator")[]
    edit: ("guest" | "registered" | "member" | "creator")[]
  }
}

export interface CategoryDefault {
  /** An integer that uniquely points to a resource. */
  id: Reference
  name: "_default"
  license: string
  ratings: "disabled" | "plus" | "plusminus" | "star"
  discussions: boolean
  autonumber: false
  permissions: {
    createPages: ("guest" | "registered" | "member")[]
    renamePages: ("guest" | "registered" | "member" | "creator")[]
    deletePages: ("guest" | "registered" | "member" | "creator")[]
    uploadFiles: ("guest" | "registered" | "member" | "creator")[]
    changeFiles: ("guest" | "registered" | "member" | "creator")[]
    showOptions: ("guest" | "registered" | "member" | "creator")[]
    edit: ("guest" | "registered" | "member" | "creator")[]
  }
}

export interface CategoryPatch {
  license?: string | null
  ratings?: "disabled" | "plus" | "plusminus" | "star" | null
  discussions?: boolean | null
  autonumber?: boolean
  permissions?: {
    createPages?: ("guest" | "registered" | "member")[]
    renamePages?: ("guest" | "registered" | "member" | "creator")[]
    deletePages?: ("guest" | "registered" | "member" | "creator")[]
    uploadFiles?: ("guest" | "registered" | "member" | "creator")[]
    changeFiles?: ("guest" | "registered" | "member" | "creator")[]
    showOptions?: ("guest" | "registered" | "member" | "creator")[]
    edit?: ("guest" | "registered" | "member" | "creator")[]
  }
}

export interface CategoryDefaultPatch {
  license?: string
  ratings?: "disabled" | "plus" | "plusminus" | "star"
  discussions?: boolean
  autonumber?: false
  permissions?: {
    createPages?: ("guest" | "registered" | "member")[]
    renamePages?: ("guest" | "registered" | "member" | "creator")[]
    deletePages?: ("guest" | "registered" | "member" | "creator")[]
    uploadFiles?: ("guest" | "registered" | "member" | "creator")[]
    changeFiles?: ("guest" | "registered" | "member" | "creator")[]
    showOptions?: ("guest" | "registered" | "member" | "creator")[]
    edit?: ("guest" | "registered" | "member" | "creator")[]
  }
}

export type CategoryList = Paginated & { categories: Category[] }

export interface SiteSettings {
  general: {
    address: string
    title: string
    subtitle: string
    language: string
    description: string
    defaultPage: Slug
    welcomePage: Slug
  }
  integrations: { googleAnalytics: string | null }
  security: (
    | { policy: "open"; cloning: boolean; fileHotLinking: boolean }
    | {
        policy: "closed"
        cloning: boolean
        fileHotLinking: boolean
        usersCanApply: boolean
        sitePassword: string
      }
    | {
        policy: "private"
        usersCanApply: boolean
        sitePassword: string
        guestDefaultPage: Slug
        guestHideNav: boolean
        extraUsers: UserIdentity[]
      }
  ) & { guestAllowLinks: boolean; userLinkMinKarma: boolean }
  appearance: { userKarma: boolean; toolbar: { top: boolean; bottom: boolean } }
  forum: {
    nestingDepth: number
    permissions: {
      createPosts: ("guest" | "registered" | "member")[]
      createThreads: ("guest" | "registered" | "member")[]
      edit: ("guest" | "registered" | "member" | "author")[]
    }
  }
}

export interface SiteSettingsPatch {
  general?: {
    address?: string
    title?: string
    subtitle?: string
    language?: string
    description?: string
    defaultPage?: Slug
    welcomePage?: Slug
  }
  integrations?: { googleAnalytics?: string | null }
  security?: (
    | { policy?: "open"; cloning?: boolean; fileHotLinking?: boolean }
    | {
        policy?: "closed"
        cloning?: boolean
        fileHotLinking?: boolean
        usersCanApply?: boolean
        sitePassword?: string
      }
    | {
        policy?: "private"
        usersCanApply?: boolean
        sitePassword?: string
        guestDefaultPage?: Slug
        guestHideNav?: boolean
        extraUsers?: Reference[]
      }
  ) & { guestAllowLinks?: boolean; userLinkMinKarma?: boolean }
  appearance?: { userKarma?: boolean; toolbar?: { top?: boolean; bottom?: boolean } }
  forum?: {
    nestingDepth?: number
    permissions?: {
      createPosts?: ("guest" | "registered" | "member")[]
      createThreads?: ("guest" | "registered" | "member")[]
      edit?: ("guest" | "registered" | "member" | "author")[]
    }
  }
}

export interface CreateSiteSettings {
  address: string
  title: string
  subtitle: string
  language: string
  description: string

  /**
   * Describes a page *slug*, a string consisting of an optional category
   * and name. It is formatted as `category:name` if a category is
   * included. If a category is not included, it is simply `name`.
   */
  defaultPage: Slug

  /**
   * Describes a page *slug*, a string consisting of an optional category
   * and name. It is formatted as `category:name` if a category is
   * included. If a category is not included, it is simply `name`.
   */
  welcomePage: Slug
  policy: "open" | "closed" | "private"
}

export interface SiteNewsletter {
  to: ("member" | "moderator" | "admin")[]
  title: string

  /** A chunk of text in FTML format. */
  wikitext: Wikitext
}

export interface SiteTransfer {
  site: SiteName

  /** An integer that uniquely points to a resource. */
  current: Reference

  /** An integer that uniquely points to a resource. */
  next: Reference
}

export type QueryParamsType = Record<string | number, any>
export type ResponseFormat = keyof Omit<Body, "body" | "bodyUsed">

export interface FullRequestParams extends Omit<RequestInit, "body"> {
  /** Set parameter to `true` for call `securityWorker` for this request */
  secure?: boolean
  /** Request path */
  path: string
  /** Content type of request body */
  type?: ContentType
  /** Query params */
  query?: QueryParamsType
  /** Format of response (i.e. response.json() -> format: "json") */
  format?: ResponseFormat
  /** Request body */
  body?: unknown
  /** Base url */
  baseUrl?: string
  /** Request cancellation token */
  cancelToken?: CancelToken
}

export type RequestParams = Omit<FullRequestParams, "body" | "method" | "query" | "path">

export interface ApiConfig<SecurityDataType = unknown> {
  baseUrl?: string
  baseApiParams?: Omit<RequestParams, "baseUrl" | "cancelToken" | "signal">
  securityWorker?: (
    securityData: SecurityDataType | null
  ) => Promise<RequestParams | void> | RequestParams | void
  customFetch?: typeof fetch
}

export interface HttpResponse<D extends unknown, E extends unknown = unknown>
  extends Response {
  data: D
  error: E
}

type CancelToken = Symbol | string | number

export enum ContentType {
  Json = "application/json",
  FormData = "multipart/form-data",
  UrlEncoded = "application/x-www-form-urlencoded"
}

export class HttpClient<SecurityDataType = unknown> {
  public baseUrl: string = "https://wikijump.com/api--v1"
  private securityData: SecurityDataType | null = null
  private securityWorker?: ApiConfig<SecurityDataType>["securityWorker"]
  private abortControllers = new Map<CancelToken, AbortController>()
  private customFetch = (...fetchParams: Parameters<typeof fetch>) =>
    fetch(...fetchParams)

  private baseApiParams: RequestParams = {
    credentials: "same-origin",
    headers: {},
    redirect: "follow",
    referrerPolicy: "no-referrer"
  }

  constructor(apiConfig: ApiConfig<SecurityDataType> = {}) {
    Object.assign(this, apiConfig)
  }

  public setSecurityData = (data: SecurityDataType | null) => {
    this.securityData = data
  }

  private encodeQueryParam(key: string, value: any) {
    const encodedKey = encodeURIComponent(key)
    return `${encodedKey}=${encodeURIComponent(
      typeof value === "number" ? value : `${value}`
    )}`
  }

  private addQueryParam(query: QueryParamsType, key: string) {
    return this.encodeQueryParam(key, query[key])
  }

  private addArrayQueryParam(query: QueryParamsType, key: string) {
    const value = query[key]
    return value.map((v: any) => this.encodeQueryParam(key, v)).join("&")
  }

  protected toQueryString(rawQuery?: QueryParamsType): string {
    const query = rawQuery || {}
    const keys = Object.keys(query).filter(key => "undefined" !== typeof query[key])
    return keys
      .map(key =>
        Array.isArray(query[key])
          ? this.addArrayQueryParam(query, key)
          : this.addQueryParam(query, key)
      )
      .join("&")
  }

  protected addQueryParams(rawQuery?: QueryParamsType): string {
    const queryString = this.toQueryString(rawQuery)
    return queryString ? `?${queryString}` : ""
  }

  private contentFormatters: Record<ContentType, (input: any) => any> = {
    [ContentType.Json]: (input: any) =>
      input !== null && (typeof input === "object" || typeof input === "string")
        ? JSON.stringify(input)
        : input,
    [ContentType.FormData]: (input: any) =>
      Object.keys(input || {}).reduce((formData, key) => {
        const property = input[key]
        formData.append(
          key,
          property instanceof Blob
            ? property
            : typeof property === "object" && property !== null
            ? JSON.stringify(property)
            : `${property}`
        )
        return formData
      }, new FormData()),
    [ContentType.UrlEncoded]: (input: any) => this.toQueryString(input)
  }

  private mergeRequestParams(
    params1: RequestParams,
    params2?: RequestParams
  ): RequestParams {
    return {
      ...this.baseApiParams,
      ...params1,
      ...(params2 || {}),
      headers: {
        ...(this.baseApiParams.headers || {}),
        ...(params1.headers || {}),
        ...((params2 && params2.headers) || {})
      }
    }
  }

  private createAbortSignal = (cancelToken: CancelToken): AbortSignal | undefined => {
    if (this.abortControllers.has(cancelToken)) {
      const abortController = this.abortControllers.get(cancelToken)
      if (abortController) {
        return abortController.signal
      }
      return void 0
    }

    const abortController = new AbortController()
    this.abortControllers.set(cancelToken, abortController)
    return abortController.signal
  }

  public abortRequest = (cancelToken: CancelToken) => {
    const abortController = this.abortControllers.get(cancelToken)

    if (abortController) {
      abortController.abort()
      this.abortControllers.delete(cancelToken)
    }
  }

  public request = async <T = any, E = any>({
    body,
    secure,
    path,
    type,
    query,
    format,
    baseUrl,
    cancelToken,
    ...params
  }: FullRequestParams): Promise<T> => {
    const secureParams =
      ((typeof secure === "boolean" ? secure : this.baseApiParams.secure) &&
        this.securityWorker &&
        (await this.securityWorker(this.securityData))) ||
      {}
    const requestParams = this.mergeRequestParams(params, secureParams)
    const queryString = query && this.toQueryString(query)
    const payloadFormatter = this.contentFormatters[type || ContentType.Json]
    const responseFormat = format || requestParams.format

    return this.customFetch(
      `${baseUrl || this.baseUrl || ""}${path}${queryString ? `?${queryString}` : ""}`,
      {
        ...requestParams,
        headers: {
          ...(type && type !== ContentType.FormData ? { "Content-Type": type } : {}),
          ...(requestParams.headers || {})
        },
        signal: cancelToken ? this.createAbortSignal(cancelToken) : void 0,
        body: typeof body === "undefined" || body === null ? null : payloadFormatter(body)
      }
    ).then(async response => {
      const r = response as HttpResponse<T, E>
      r.data = null as unknown as T
      r.error = null as unknown as E

      const data = !responseFormat
        ? r
        : await response[responseFormat]()
            .then(data => {
              if (r.ok) {
                r.data = data
              } else {
                r.error = data
              }
              return r
            })
            .catch(e => {
              r.error = e
              return r
            })

      if (cancelToken) {
        this.abortControllers.delete(cancelToken)
      }

      if (!response.ok) throw data
      return data.data
    })
  }
}

/**
 * @license GNU Affero General Public License 3.0 (AGPL 3.0)
 *   (http://www.gnu.org/licenses/agpl-3.0.html)
 * @version 0.1.0
 * @title Wikijump
 * @baseUrl https://wikijump.com/api--v1
 * @contact Wikijump Github (https://github.com/scpwiki/wikijump)
 *
 * ## Introduction
 *
 * This page documents the Wikijump API, which is a HTTP interface
 * that lets you easily interact with Wikijump programatically.
 *
 * The API is defined as a OpenAPI 3.0.3 schema file.
 * The implementation of this API in the Wikijump backend very closely matches this schema.
 *
 * Wikijump (including this API) is subject to change as the project evolves.
 *
 * ## Usage
 *
 * The Wikijump API is centered around the JSON format for both sending and receiving.
 * The data expected/returned by the API will always be wrapped inside an object, even if that object contains only
 * a single property. The following code-block demonstrates an example payload:
 * ```json
 * {
 *   "login": "fake@example.com",
 *   "password": "pa$$word"
 * }
 * ```
 *
 * The only exceptions to the 'JSON only' standard are files and avatars, which are sent directly.
 *
 * If you plan to use this API with some sort of service, e.g. an IRC bot, it should be noted that this API is
 * fundamentally geared towards supporting frontends. Endpoints usually point to individual resources or
 * interactions, which means that any sort of service requesting information on many resources will have to send
 * a large amount of API requests if it were to use this API as a frontend would.
 *
 * However, the Wikijump API has a special endpoint, `/query`, which allows complex and "closer to the metal"
 * transactions with the Wikijump database. Services, but also frontends, can use this endpoint to
 * request information efficiently on a multitude of resources without excessive API requests and bandwidth usage.
 *
 * The `/query` endpoint is currently not available.
 *
 * ## Changes from Wikidot
 *
 * > Breaking changes to previously private components of the Wikidot interface or API
 * will not be mentioned in this section.
 *
 * The following features have been deprecated, removed, or rendered nonfunctional:
 * - Contacts
 * - Account types (pro accounts)
 * - OneSignal
 * - Advertising
 * - XML-RPC API
 * - Pingbacks
 * - Twitter integration ("Tweet My Wiki")
 * - Site traffic statistics (Google Analytics is still available!)
 * - Per-site user profiles
 * - Inter-site promotion
 * - Custom layouts
 * - Custom footer
 *
 * The following features have been made available universally:
 * - HTTPS (required - Wikijump is designed to use HTTPS only)
 * - Blocking site cloning
 * - Disabling the display of karma site-wide
 *
 * These changes reflect Wikijump's transition away from Wikidot's business model and outdated infrastructure.
 *
 * Finally, the following features **will be supported**, but aren't quite available in this API yet:
 * - Watching
 * - Activity pages (user, site, etc.) (requires `/query`)
 * - Searching (requires `/query`)
 *
 * ## Constructing Navigable URLS
 *
 * (todo)
 */
export class Api<SecurityDataType extends unknown> extends HttpClient<SecurityDataType> {
  /**
   * INCOMPLETE - STUB
   *
   * @tags query
   * @name QueryRequest
   * @request POST:/query
   * @secure
   */
  queryRequest = (params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/query`,
      method: "POST",
      secure: true,
      ...params
    })

  /**
   * Resolves an ID and returns what type of object it refers to.
   *
   * @tags util
   * @name UtilResolveId
   * @request GET:/util/resolveid/{id}
   */
  utilResolveId = (id: Reference, params: RequestParams = {}) =>
    this.request<{ type: ReferenceTypes }, void>({
      path: `/util/resolveid/${id}`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Attempts a login. The login specifier can be either a username or an
   * email address.
   *
   * @tags auth
   * @name AuthLogin
   * @request POST:/auth/login
   */
  authLogin = (
    data: { login: LoginSpecifier; password: string; remember?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<{ csrf: string }, void>({
      path: `/auth/login`,
      method: "POST",
      body: data,
      type: ContentType.Json,
      format: "json",
      ...params
    })

  /**
   * Logs the client out.
   *
   * @tags auth
   * @name AuthLogout
   * @request DELETE:/auth/logout
   * @secure
   */
  authLogout = (params: RequestParams = {}) =>
    this.request<any, void>({
      path: `/auth/logout`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets the authentication state of the client.
   *
   * @tags auth
   * @name AuthCheck
   * @request POST:/auth/check
   */
  authCheck = (params: RequestParams = {}) =>
    this.request<{ sessionValid: boolean; authed: boolean }, void>({
      path: `/auth/check`,
      method: "POST",
      format: "json",
      ...params
    })

  /**
   * Refreshes the client's session.
   *
   * @tags auth
   * @name AuthRefresh
   * @request POST:/auth/refresh
   * @secure
   */
  authRefresh = (params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/auth/refresh`,
      method: "POST",
      secure: true,
      ...params
    })

  /**
   * Registers an account. Does not automatically login. Email validation
   * will be required.
   *
   * @tags account
   * @name AccountRegister
   * @request POST:/account/register
   */
  accountRegister = (
    data: { username: Username; email: Email; password: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/account/register`,
      method: "POST",
      body: data,
      type: ContentType.Json,
      ...params
    })

  /**
   * Starts the deletion process for an account. Requires additional email
   * validation for the process to complete.
   *
   * @tags account
   * @name AccountRequestDeletion
   * @request POST:/account/request-deletion
   * @secure
   */
  accountRequestDeletion = (params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/account/request-deletion`,
      method: "POST",
      secure: true,
      ...params
    })

  /**
   * Starts the password recovery routine.
   *
   * @tags account
   * @name AccountStartRecovery
   * @request POST:/account/start-recovery
   */
  accountStartRecovery = (data: { email: Email }, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/account/start-recovery`,
      method: "POST",
      body: data,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the current email address.
   *
   * @tags account
   * @name AccountGetEmail
   * @request GET:/account/email
   * @secure
   */
  accountGetEmail = (params: RequestParams = {}) =>
    this.request<{ email: Email }, void>({
      path: `/account/email`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Updates the current email address. Does not immediately change the
   * email, as the change must be verified through a link that is sent to
   * the requested email.
   *
   * @tags account
   * @name AccountUpdateEmail
   * @request PUT:/account/email
   * @secure
   */
  accountUpdateEmail = (
    data: { oldEmail: Email; newEmail: Email },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/account/email`,
      method: "PUT",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Updates the current password.
   *
   * @tags account
   * @name AccountUpdatePassword
   * @request PUT:/account/password
   * @secure
   */
  accountUpdatePassword = (
    data: { oldPassword: string; newPassword: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/account/password`,
      method: "PUT",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the current username.
   *
   * @tags account
   * @name AccountGetUsername
   * @request GET:/account/username
   * @secure
   */
  accountGetUsername = (params: RequestParams = {}) =>
    this.request<{ username: Username }, void>({
      path: `/account/username`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Updates the current username.
   *
   * @tags account
   * @name AccountUpdateUsername
   * @request PUT:/account/username
   * @secure
   */
  accountUpdateUsername = (data: { username: Username }, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/account/username`,
      method: "PUT",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the current account settings.
   *
   * @tags account
   * @name AccountGetSettings
   * @request GET:/account/settings
   * @secure
   */
  accountGetSettings = (params: RequestParams = {}) =>
    this.request<AccountSettings, void>({
      path: `/account/settings`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Update (patch) the client's user details.
   *
   * @tags account
   * @name AccountUpdateSettings
   * @request PATCH:/account/settings
   * @secure
   */
  accountUpdateSettings = (data: AccountSettingsPatch, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/account/settings`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the client's current notifications.
   *
   * @tags notification
   * @name NotificationGet
   * @request GET:/notification
   * @secure
   */
  notificationGet = (params: RequestParams = {}) =>
    this.request<NotificationList, void>({
      path: `/notification`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Dismisses all of the client's notifications.
   *
   * @tags notification
   * @name NotificationDismissAll
   * @request DELETE:/notification
   * @secure
   */
  notificationDismissAll = (params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/notification`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets the client's user details.
   *
   * @tags user, avatars
   * @name UserClientGet
   * @request GET:/user
   * @secure
   */
  userClientGet = (
    query?: { detail?: "identity" | "info" | "profile"; avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<UserIdentity | UserInfo | UserProfile, void>({
      path: `/user`,
      method: "GET",
      query: query,
      secure: true,
      ...params
    })

  /**
   * Update (patch) the client's user details.
   *
   * @tags user
   * @name UserClientUpdateProfile
   * @request PATCH:/user
   * @secure
   */
  userClientUpdateProfile = (data: UserProfilePatch, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/user`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the client's avatar.
   *
   * @tags user, not-json
   * @name UserClientGetAvatar
   * @request GET:/user/avatar
   */
  userClientGetAvatar = (params: RequestParams = {}) =>
    this.request<FileData | null, void>({
      path: `/user/avatar`,
      method: "GET",
      ...params
    })

  /**
   * Sets the client's avatar.
   *
   * @tags user, not-json
   * @name UserClientSetAvatar
   * @request PUT:/user/avatar
   * @secure
   */
  userClientSetAvatar = (data: FileData, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/user/avatar`,
      method: "PUT",
      body: data,
      secure: true,
      ...params
    })

  /**
   * Removes the client's avatar.
   *
   * @tags user
   * @name UserClientRemoveAvatar
   * @request DELETE:/user/avatar
   * @secure
   */
  userClientRemoveAvatar = (params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/user/avatar`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets the list of users the client has blocked.
   *
   * @tags user
   * @name UserClientGetBlocked
   * @request GET:/user/blocked
   * @secure
   */
  userClientGetBlocked = (params: RequestParams = {}) =>
    this.request<UserBlockedList, void>({
      path: `/user/blocked`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets a user's details.
   *
   * @tags user, avatars
   * @name UserGet
   * @request GET:/user/{path_type}/{path}
   */
  userGet = (
    pathType: "id" | "name",
    path: Username | Reference,
    query?: { avatars?: boolean; detail?: "identity" | "info" | "profile" },
    params: RequestParams = {}
  ) =>
    this.request<UserIdentity | UserInfo | UserProfile, void>({
      path: `/user/${pathType}/${path}`,
      method: "GET",
      query: query,
      ...params
    })

  /**
   * Resets a user's profile. > This endpoint is only available to platform
   * administrators.
   *
   * @tags user, moderation, platform-admin
   * @name UserResetProfile
   * @request DELETE:/user/{path_type}/{path}
   * @secure
   */
  userResetProfile = (
    pathType: "id" | "name",
    path: Username | Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}`,
      method: "DELETE",
      query: query,
      secure: true,
      ...params
    })

  /**
   * Gets a user's avatar.
   *
   * @tags user, not-json
   * @name UserGetAvatar
   * @request GET:/user/{path_type}/{path}/avatar
   */
  userGetAvatar = (
    pathType: "id" | "name",
    path: Username | Reference,
    params: RequestParams = {}
  ) =>
    this.request<FileData | null, void>({
      path: `/user/${pathType}/${path}/avatar`,
      method: "GET",
      ...params
    })

  /**
   * Removes a user's avatar. > This endpoint is only available to platform
   * administrators.
   *
   * @tags user, moderation, platform-admin
   * @name UserRemoveAvatar
   * @request DELETE:/user/{path_type}/{path}/avatar
   * @secure
   */
  userRemoveAvatar = (
    pathType: "id" | "name",
    path: Username | Reference,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}/avatar`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets whether or not the client has a user blocked.
   *
   * @tags user
   * @name UserGetBlocked
   * @request GET:/user/{path_type}/{path}/block
   * @secure
   */
  userGetBlocked = (
    pathType: "id" | "name",
    path: Username | Reference,
    params: RequestParams = {}
  ) =>
    this.request<{ blocked: boolean }, void>({
      path: `/user/${pathType}/${path}/block`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Updates whether or not the client has a user blocked.
   *
   * @tags user
   * @name UserUpdateBlocked
   * @request PUT:/user/{path_type}/{path}/block
   * @secure
   */
  userUpdateBlocked = (
    pathType: "id" | "name",
    path: Username | Reference,
    data: { blocked: boolean },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}/block`,
      method: "PUT",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the sites the client is a member of.
   *
   * @tags membership
   * @name MembershipGetList
   * @request GET:/membership
   * @secure
   */
  membershipGetList = (params: RequestParams = {}) =>
    this.request<MembershipList, void>({
      path: `/membership`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets the sites the client has requested to join.
   *
   * @tags membership
   * @name MembershipGetApplications
   * @request GET:/membership/applications
   * @secure
   */
  membershipGetApplications = (params: RequestParams = {}) =>
    this.request<ApplicationSendList, void>({
      path: `/membership/applications`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets the sites the client has been invited to join.
   *
   * @tags membership
   * @name MembershipGetInvites
   * @request GET:/membership/invites
   * @secure
   */
  membershipGetInvites = (params: RequestParams = {}) =>
    this.request<InviteList, void>({
      path: `/membership/invites`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets the client membership status for a site.
   *
   * @tags membership
   * @name MembershipSiteGet
   * @request GET:/membership/site/{site}
   * @secure
   */
  membershipSiteGet = (site: string, params: RequestParams = {}) =>
    this.request<Membership | null, void>({
      path: `/membership/site/${site}`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Requests to join a site (application).
   *
   * @tags membership
   * @name MembershipSiteApply
   * @request POST:/membership/site/{site}
   * @secure
   */
  membershipSiteApply = (
    site: string,
    data: ApplicationSend,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/membership/site/${site}`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Leaves a site.
   *
   * @tags membership
   * @name MembershipSiteLeave
   * @request DELETE:/membership/site/{site}
   * @secure
   */
  membershipSiteLeave = (site: string, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/membership/site/${site}`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets the sites a user is a member of.
   *
   * @tags membership
   * @name MembershipUserGetList
   * @request GET:/member/{path_type}/{path}/membership
   */
  membershipUserGetList = (
    pathType: "id" | "name",
    path: Username | Reference,
    params: RequestParams = {}
  ) =>
    this.request<MembershipList, void>({
      path: `/member/${pathType}/${path}/membership`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Gets a user's membership status for a site.
   *
   * @tags membership
   * @name MembershipUserSiteGet
   * @request GET:/member/{path_type}/{path}/membership/{site}
   */
  membershipUserSiteGet = (
    site: string,
    pathType: "id" | "name",
    path: Username | Reference,
    params: RequestParams = {}
  ) =>
    this.request<Membership | null, void>({
      path: `/member/${pathType}/${path}/membership/${site}`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Gets the role of a user.
   *
   * @tags membership
   * @name MembershipUserGetRole
   * @request GET:/member/{path_type}/{path}/role
   */
  membershipUserGetRole = (
    pathType: "id" | "name",
    path: Username | Reference,
    params: RequestParams = {}
  ) =>
    this.request<MembershipRole, void>({
      path: `/member/${pathType}/${path}/role`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Sets the role of a user.
   *
   * @tags membership
   * @name MembershipUserSetRole
   * @request POST:/member/{path_type}/{path}/role
   * @secure
   */
  membershipUserSetRole = (
    pathType: "id" | "name",
    path: Username | Reference,
    data: MembershipRole,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/member/${pathType}/${path}/role`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Invites a user to join a site.
   *
   * @tags membership
   * @name MembershipUserInvite
   * @request POST:/member/{path_type}/{path}/invite
   * @secure
   */
  membershipUserInvite = (
    pathType: "id" | "name",
    path: Username | Reference,
    data: InviteSend,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/member/${pathType}/${path}/invite`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Creates a new page.
   *
   * @tags page
   * @name PageCreate
   * @request POST:/page
   * @secure
   */
  pageCreate = (
    data: { slug: Slug; title?: string; wikitext?: Wikitext },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets a page.
   *
   * @tags page, avatars
   * @name PageGet
   * @request GET:/page/{path_type}/{path}
   */
  pageGet = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    query?: {
      type?:
        | "all"
        | "metadata-html"
        | "metadata"
        | "wikitext"
        | "html"
        | "syntaxtree"
        | "none"
      avatars?: boolean
    },
    params: RequestParams = {}
  ) =>
    this.request<Page | WikitextObj | HTMLObj | FTMLSyntaxTree, void>({
      path: `/page/${pathType}/${path}`,
      method: "GET",
      query: query,
      ...params
    })

  /**
   * Updates a page.
   *
   * @tags page
   * @name PageUpdate
   * @request PATCH:/page/{path_type}/{path}
   * @secure
   */
  pageUpdate = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    data: { title?: string; wikitext?: Wikitext },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Deletes a page.
   *
   * @tags page
   * @name PageDelete
   * @request DELETE:/page/{path_type}/{path}
   * @secure
   */
  pageDelete = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Restores a previously deleted page.
   *
   * @tags page
   * @name PageRestore
   * @request POST:/page/id/{path}/restore
   * @secure
   */
  pageRestore = (path: Reference, data: { slug: Slug }, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/page/id/${path}/restore`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Changes the path/slug/name of a page.
   *
   * @tags page
   * @name PageRename
   * @request POST:/page/{path_type}/{path}/rename
   * @secure
   */
  pageRename = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    data: { slug: Slug },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/rename`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the update/revision history of a page.
   *
   * @tags revision, paginated, avatars
   * @name RevisionPageGetHistory
   * @request GET:/page/{path_type}/{path}/revision
   */
  revisionPageGetHistory = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    query?: { cursor?: number; limit?: number; avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<RevisionHistory, void>({
      path: `/page/${pathType}/${path}/revision`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets the page corresponding to a revision.
   *
   * @tags revision, avatars
   * @name RevisionGet
   * @request GET:/page/{path_type}/{path}/revision/{revision}
   */
  revisionGet = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    revision: number,
    query?: {
      type?:
        | "all"
        | "metadata-html"
        | "metadata"
        | "wikitext"
        | "html"
        | "syntaxtree"
        | "none"
      avatars?: boolean
    },
    params: RequestParams = {}
  ) =>
    this.request<Page | WikitextObj | HTMLObj | FTMLSyntaxTree, void>({
      path: `/page/${pathType}/${path}/revision/${revision}`,
      method: "GET",
      query: query,
      ...params
    })

  /**
   * Updates the metadata of a revision.
   *
   * @tags revision
   * @name RevisionUpdateMetadata
   * @request PATCH:/page/{path_type}/{path}/revision/{revision}
   * @secure
   */
  revisionUpdateMetadata = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    revision: number,
    data: { hidden?: boolean; message?: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/revision/${revision}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Resets a page to a past revision.
   *
   * @tags revision
   * @name RevisionResetToRevision
   * @request POST:/page/{path_type}/{path}/revision/{revision}
   * @secure
   */
  revisionResetToRevision = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    revision: number,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/revision/${revision}`,
      method: "POST",
      secure: true,
      ...params
    })

  /**
   * Gets the tags of a page.
   *
   * @tags tag
   * @name TagPageGet
   * @request GET:/page/{path_type}/{path}/tags
   */
  tagPageGet = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    params: RequestParams = {}
  ) =>
    this.request<{ tags: TagList }, void>({
      path: `/page/${pathType}/${path}/tags`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Updates the tags of a page.
   *
   * @tags tag
   * @name TagPageUpdate
   * @request PUT:/page/{path_type}/{path}/tags
   * @secure
   */
  tagPageUpdate = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    data: { tags: TagList },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/tags`,
      method: "PUT",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the score of a page.
   *
   * @tags vote
   * @name VotePageGetScore
   * @request GET:/page/{path_type}/{path}/score
   */
  votePageGetScore = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    params: RequestParams = {}
  ) =>
    this.request<Score, void>({
      path: `/page/${pathType}/${path}/score`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Gets the voters and votes of a page.
   *
   * @tags vote, paginated, avatars
   * @name VotePageGetVoters
   * @request GET:/page/{path_type}/{path}/voters
   */
  votePageGetVoters = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    query?: { cursor?: number; limit?: number; avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<VoterList, void>({
      path: `/page/${pathType}/${path}/voters`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets the client's voting state on a page, if any.
   *
   * @tags vote
   * @name VotePageGet
   * @request GET:/page/{path_type}/{path}/vote
   * @secure
   */
  votePageGet = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    params: RequestParams = {}
  ) =>
    this.request<{ vote: CastVote | null }, void>({
      path: `/page/${pathType}/${path}/vote`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Updates/sets the client's voting state on a page.
   *
   * @tags vote
   * @name VotePageUpdateVote
   * @request PUT:/page/{path_type}/{path}/vote
   * @secure
   */
  votePageUpdateVote = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    data: { vote: CastVote },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/vote`,
      method: "PUT",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Removes the client's voting state on a page.
   *
   * @tags vote
   * @name VotePageRemoveVote
   * @request DELETE:/page/{path_type}/{path}/vote
   * @secure
   */
  votePageRemoveVote = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/vote`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets metadata on all files attached to a page.
   *
   * @tags file, avatars
   * @name FilePageGetMetadata
   * @request GET:/page/{path_type}/{path}/file
   */
  filePageGetMetadata = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<{ files: FileMetadata[] }, void>({
      path: `/page/${pathType}/${path}/file`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Adds a new file to a page.
   *
   * @tags file, not-json
   * @name FilePageAdd
   * @request POST:/page/{path_type}/{path}/file
   * @secure
   */
  filePageAdd = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    data: FileUpload,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/file`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.FormData,
      ...params
    })

  /**
   * Gets metadata on the files attached directly to the site instance. >
   * This does not include files attached to *pages*.
   *
   * @tags file, paginated, avatars
   * @name FileSiteGetMetadata
   * @request GET:/file
   */
  fileSiteGetMetadata = (query?: { avatars?: boolean }, params: RequestParams = {}) =>
    this.request<Paginated & { files: FileMetadata[] }, void>({
      path: `/file`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Adds a new file to a site instance.
   *
   * @tags file, not-json
   * @name FileSiteAdd
   * @request POST:/file
   * @secure
   */
  fileSiteAdd = (data: FileUpload, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/file`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.FormData,
      ...params
    })

  /**
   * Gets a file.
   *
   * @tags file, not-json
   * @name FileGet
   * @request GET:/file/{id}
   */
  fileGet = (id: Reference, params: RequestParams = {}) =>
    this.request<FileData, void>({
      path: `/file/${id}`,
      method: "GET",
      ...params
    })

  /**
   * Deletes a file.
   *
   * @tags file
   * @name FileDelete
   * @request DELETE:/file/{id}
   * @secure
   */
  fileDelete = (id: Reference, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/file/${id}`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets the site's file-system metadata, e.g. remaining file space.
   *
   * @tags file
   * @name FileGetSiteMetadata
   * @request GET:/file/metadata
   */
  fileGetSiteMetadata = (params: RequestParams = {}) =>
    this.request<FileSiteMetadata, void>({
      path: `/file/metadata`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Gets a file's metadata.
   *
   * @tags file, avatars
   * @name FileGetMetadata
   * @request GET:/file/{id}/metadata
   */
  fileGetMetadata = (
    id: Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<FileMetadata, void>({
      path: `/file/${id}/metadata`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets the reports against a user.
   *
   * @tags report, avatars
   * @name ReportUserGet
   * @request GET:/user/{path_type}/{path}/report
   * @secure
   */
  reportUserGet = (
    pathType: "id" | "name",
    path: Username | Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ReportList, void>({
      path: `/user/${pathType}/${path}/report`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Reports a user.
   *
   * @tags report
   * @name ReportUserSend
   * @request POST:/user/{path_type}/{path}/report
   * @secure
   */
  reportUserSend = (
    pathType: "id" | "name",
    path: Username | Reference,
    data: ReportSend,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}/report`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets a page's reports.
   *
   * @tags report, avatars
   * @name ReportPageGet
   * @request GET:/page/{path_type}/{path}/report
   * @secure
   */
  reportPageGet = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ReportList, void>({
      path: `/page/${pathType}/${path}/report`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Reports a page.
   *
   * @tags report
   * @name ReportPageSend
   * @request POST:/page/{path_type}/{path}/report
   * @secure
   */
  reportPageSend = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    data: ReportSend,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/report`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets a report.
   *
   * @tags report, avatars
   * @name ReportGet
   * @request GET:/report/{id}
   * @secure
   */
  reportGet = (
    id: Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<Report, void>({
      path: `/report/${id}`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets the reports against a site. > This endpoint is only available to
   * platform administrators.
   *
   * @tags abuse, platform-admin, avatars
   * @name AbuseSiteGet
   * @request GET:/abuse
   * @secure
   */
  abuseSiteGet = (query?: { avatars?: boolean }, params: RequestParams = {}) =>
    this.request<ReportList, void>({
      path: `/abuse`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Reports a site.
   *
   * @tags abuse
   * @name AbuseSiteSend
   * @request POST:/abuse
   * @secure
   */
  abuseSiteSend = (data: ReportSend, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/abuse`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the reports against a user. > This endpoint is only available to
   * platform administrators.
   *
   * @tags abuse, platform-admin, avatars
   * @name AbuseUserGet
   * @request GET:/user/{path_type}/{path}/abuse
   * @secure
   */
  abuseUserGet = (
    pathType: "id" | "name",
    path: Username | Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ReportList, void>({
      path: `/user/${pathType}/${path}/abuse`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Reports a user.
   *
   * @tags abuse
   * @name AbuseUserSend
   * @request POST:/user/{path_type}/{path}/abuse
   * @secure
   */
  abuseUserSend = (
    pathType: "id" | "name",
    path: Username | Reference,
    data: ReportSend,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}/abuse`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets a page's reports. > This endpoint is only available to platform
   * administrators.
   *
   * @tags abuse, platform-admin, avatars
   * @name AbusePageGet
   * @request GET:/page/{path_type}/{path}/abuse
   * @secure
   */
  abusePageGet = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ReportList, void>({
      path: `/page/${pathType}/${path}/abuse`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Reports a page.
   *
   * @tags abuse
   * @name AbusePageSend
   * @request POST:/page/{path_type}/{path}/abuse
   * @secure
   */
  abusePageSend = (
    pathType: "id" | "slug",
    path: Slug | Reference,
    data: ReportSend,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/page/${pathType}/${path}/abuse`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets a report. > This endpoint is only available to platform administrators.
   *
   * @tags abuse, platform-admin, avatars
   * @name AbuseGet
   * @request GET:/abuse/{id}
   * @secure
   */
  abuseGet = (id: Reference, query?: { avatars?: boolean }, params: RequestParams = {}) =>
    this.request<Report, void>({
      path: `/abuse/${id}`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets all of the client's messages.
   *
   * @tags message, paginated, avatars
   * @name MessageGetList
   * @request GET:/message
   * @secure
   */
  messageGetList = (
    query?: {
      cursor?: number
      limit?: number
      detail?: "with-html" | "metadata"
      archived?: boolean
      avatars?: boolean
    },
    params: RequestParams = {}
  ) =>
    this.request<MessageList, void>({
      path: `/message`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets a message.
   *
   * @tags message, avatars
   * @name MessageGet
   * @request GET:/message/{id}
   * @secure
   */
  messageGet = (
    id: Reference,
    query?: { detail?: "with-html" | "metadata"; avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<Message, void>({
      path: `/message/${id}`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Updates the metadata of a message, such as read or unread.
   *
   * @tags message
   * @name MessageUpdate
   * @request PATCH:/message/{id}
   * @secure
   */
  messageUpdate = (
    id: Reference,
    data: { read?: boolean; archived?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/message/${id}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Deletes a message.
   *
   * @tags message
   * @name MessageDelete
   * @request DELETE:/message/{id}
   * @secure
   */
  messageDelete = (id: Reference, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/message/${id}`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Messages a user.
   *
   * @tags message
   * @name MessageSend
   * @request POST:/user/{path_type}/{path}/message
   * @secure
   */
  messageSend = (
    pathType: "id" | "name",
    path: Username | Reference,
    data: MessageSend,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}/message`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the groups and categories of a forum.
   *
   * @tags forum, forum-misc, avatars
   * @name ForumGet
   * @request GET:/forum
   */
  forumGet = (query?: { avatars?: boolean }, params: RequestParams = {}) =>
    this.request<Forum, void>({
      path: `/forum`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets the groups of a forum.
   *
   * @tags forum, forum-group, avatars
   * @name ForumGroupGetList
   * @request GET:/forum/group
   */
  forumGroupGetList = (query?: { avatars?: boolean }, params: RequestParams = {}) =>
    this.request<ForumGroupList, void>({
      path: `/forum/group`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets a group.
   *
   * @tags forum, forum-group, avatars
   * @name ForumGroupGet
   * @request GET:/forum/group/{id}
   */
  forumGroupGet = (
    id: Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ForumGroup, void>({
      path: `/forum/group/${id}`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Updates a group.
   *
   * @tags forum, forum-group
   * @name ForumGroupUpdate
   * @request PATCH:/forum/group/{id}
   * @secure
   */
  forumGroupUpdate = (
    id: Reference,
    data: { title?: string; summary?: string; order?: Reference[] },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/group/${id}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Creates a new category inside of a group.
   *
   * @tags forum, forum-group
   * @name ForumGroupAddCategory
   * @request POST:/forum/group/{id}
   * @secure
   */
  forumGroupAddCategory = (
    id: Reference,
    data: { title?: string; summary?: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/group/${id}`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Deletes a group.
   *
   * @tags forum, forum-group
   * @name ForumGroupDelete
   * @request DELETE:/forum/group/{id}
   * @secure
   */
  forumGroupDelete = (id: Reference, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/forum/group/${id}`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets the categories of a group.
   *
   * @tags forum, forum-group, avatars
   * @name ForumGroupGetCategories
   * @request GET:/forum/group/{id}/categories
   */
  forumGroupGetCategories = (
    id: Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ForumCategoryList, void>({
      path: `/forum/group/${id}/categories`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets the categories of a forum.
   *
   * @tags forum, forum-category, avatars
   * @name ForumCategoryGetList
   * @request GET:/forum/category
   */
  forumCategoryGetList = (query?: { avatars?: boolean }, params: RequestParams = {}) =>
    this.request<ForumCategoryList, void>({
      path: `/forum/category`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets a category.
   *
   * @tags forum, forum-category, avatars
   * @name ForumCategoryGet
   * @request GET:/forum/category/{id}
   */
  forumCategoryGet = (
    id: Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ForumCategory, void>({
      path: `/forum/category/${id}`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Updates a category.
   *
   * @tags forum, forum-category
   * @name ForumCategoryUpdate
   * @request PATCH:/forum/category/{id}
   * @secure
   */
  forumCategoryUpdate = (
    id: Reference,
    data: { title?: string; summary?: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/category/${id}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Creates a new thread inside of a category.
   *
   * @tags forum, forum-category
   * @name ForumCategoryAddThread
   * @request POST:/forum/category/{id}
   * @secure
   */
  forumCategoryAddThread = (
    id: Reference,
    data: { title?: string; summary?: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/category/${id}`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Deletes a category.
   *
   * @tags forum, forum-category
   * @name ForumCategoryDelete
   * @request DELETE:/forum/category/{id}
   * @secure
   */
  forumCategoryDelete = (id: Reference, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/forum/category/${id}`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets the threads of a category.
   *
   * @tags forum, forum-category, paginated, avatars
   * @name ForumCategoryGetThreads
   * @request GET:/forum/category/{id}/threads
   */
  forumCategoryGetThreads = (
    id: Reference,
    query?: { cursor?: number; limit?: number; avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ForumThreadList, void>({
      path: `/forum/category/${id}/threads`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets a thread.
   *
   * @tags forum, forum-thread, avatars
   * @name ForumThreadGet
   * @request GET:/forum/thread/{id}
   */
  forumThreadGet = (
    id: Reference,
    query?: { avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<ForumThread, void>({
      path: `/forum/thread/${id}`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Updates a thread.
   *
   * @tags forum, forum-thread
   * @name ForumThreadUpdate
   * @request PATCH:/forum/thread/{id}
   * @secure
   */
  forumThreadUpdate = (
    id: Reference,
    data: { title?: string; summary?: string; stickied?: boolean; locked?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/thread/${id}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Creates a new post inside of a thread.
   *
   * @tags forum, forum-thread
   * @name ForumThreadAddPost
   * @request POST:/forum/thread/{id}
   * @secure
   */
  forumThreadAddPost = (
    id: Reference,
    data: { title?: string; wikitext?: Wikitext },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/thread/${id}`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Deletes a thread.
   *
   * @tags forum, forum-thread
   * @name ForumThreadDelete
   * @request DELETE:/forum/thread/{id}
   * @secure
   */
  forumThreadDelete = (
    id: Reference,
    query?: { permanent?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/thread/${id}`,
      method: "DELETE",
      query: query,
      secure: true,
      ...params
    })

  /**
   * Gets the posts of a thread.
   *
   * @tags forum, forum-thread, paginated, avatars
   * @name ForumThreadGetPosts
   * @request GET:/forum/thread/{id}/posts
   */
  forumThreadGetPosts = (
    id: Reference,
    query?: {
      cursor?: number
      limit?: number
      detail?: "none" | "metadata" | "with-html" | "full"
      depth?: number
      avatars?: boolean
    },
    params: RequestParams = {}
  ) =>
    this.request<ForumPostList, void>({
      path: `/forum/thread/${id}/posts`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets a post.
   *
   * @tags forum, forum-post, avatars
   * @name ForumPostGet
   * @request GET:/forum/post/{id}
   */
  forumPostGet = (
    id: Reference,
    query?: {
      detail?: "none" | "metadata" | "with-html" | "full"
      depth?: number
      avatars?: boolean
    },
    params: RequestParams = {}
  ) =>
    this.request<ForumPost, void>({
      path: `/forum/post/${id}`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Updates a post.
   *
   * @tags forum, forum-post
   * @name ForumPostUpdate
   * @request PATCH:/forum/post/{id}
   * @secure
   */
  forumPostUpdate = (
    id: Reference,
    data: { title?: string; wikitext?: Wikitext },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/post/${id}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Replies to a post with another post.
   *
   * @tags forum, forum-post
   * @name ForumPostReply
   * @request POST:/forum/post/{id}
   * @secure
   */
  forumPostReply = (
    id: Reference,
    data: { title?: string; wikitext?: Wikitext },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/post/${id}`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Deletes a post.
   *
   * @tags forum, forum-post
   * @name ForumPostDelete
   * @request DELETE:/forum/post/{id}
   * @secure
   */
  forumPostDelete = (
    id: Reference,
    query?: { permanent?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/post/${id}`,
      method: "DELETE",
      query: query,
      secure: true,
      ...params
    })

  /**
   * Gets the replies to a post.
   *
   * @tags forum, forum-post, paginated, avatars
   * @name ForumPostGetReplies
   * @request GET:/forum/post/{id}/replies
   */
  forumPostGetReplies = (
    id: Reference,
    query?: {
      cursor?: number
      limit?: number
      detail?: "none" | "metadata" | "with-html" | "full"
      depth?: number
      avatars?: boolean
    },
    params: RequestParams = {}
  ) =>
    this.request<ForumPostList, void>({
      path: `/forum/post/${id}/replies`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets the update/revision history of a post.
   *
   * @tags forum, forum-post, paginated, avatars
   * @name ForumPostRevisionGetHistory
   * @request GET:/forum/post/{id}/revision
   */
  forumPostRevisionGetHistory = (
    id: Reference,
    query?: { cursor?: number; limit?: number; avatars?: boolean },
    params: RequestParams = {}
  ) =>
    this.request<RevisionHistory, void>({
      path: `/forum/post/${id}/revision`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Gets the post corresponding to a revision.
   *
   * @tags forum, forum-post, avatars
   * @name ForumPostRevisionGet
   * @request GET:/forum/post/{id}/revision/{revision}
   */
  forumPostRevisionGet = (
    id: Reference,
    revision: number,
    query?: { avatars?: boolean; detail?: "none" | "metadata" | "with-html" | "full" },
    params: RequestParams = {}
  ) =>
    this.request<ForumPost, void>({
      path: `/forum/post/${id}/revision/${revision}`,
      method: "GET",
      query: query,
      format: "json",
      ...params
    })

  /**
   * Updates the metadata of a revision.
   *
   * @tags forum, forum-post
   * @name ForumPostRevisionUpdateMetadata
   * @request PATCH:/forum/post/{id}/revision/{revision}
   * @secure
   */
  forumPostRevisionUpdateMetadata = (
    id: Reference,
    revision: number,
    data: { hidden?: boolean; message?: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/post/${id}/revision/${revision}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Resets a forum post to a past revision.
   *
   * @tags forum, forum-post
   * @name ForumPostResetToRevision
   * @request POST:/forum/post/{id}/revision/{revision}
   * @secure
   */
  forumPostResetToRevision = (
    id: Reference,
    revision: number,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/forum/post/${id}/revision/${revision}`,
      method: "POST",
      secure: true,
      ...params
    })

  /**
   * Kicks a user from a site.
   *
   * @tags moderation
   * @name ModerationKick
   * @request PUT:/user/{path_type}/{path}/kick
   * @secure
   */
  moderationKick = (
    pathType: "id" | "name",
    path: Username | Reference,
    data: { reason: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}/kick`,
      method: "PUT",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the list of users banned from a site.
   *
   * @tags moderation
   * @name ModerationBanGetList
   * @request GET:/moderation/banned
   * @secure
   */
  moderationBanGetList = (params: RequestParams = {}) =>
    this.request<
      { banned: { user: UserIdentity; until: string | null; reason: string }[] },
      void
    >({
      path: `/moderation/banned`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets if a user is banned.
   *
   * @tags moderation
   * @name ModerationBanGet
   * @request GET:/user/{path_type}/{path}/ban
   * @secure
   */
  moderationBanGet = (
    pathType: "id" | "name",
    path: Username | Reference,
    params: RequestParams = {}
  ) =>
    this.request<{ banned: boolean; until: string | null; reason: string }, void>({
      path: `/user/${pathType}/${path}/ban`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Bans a user. Providing `null` for `until` describes a perma-ban.
   *
   * @tags moderation
   * @name ModerationBan
   * @request PUT:/user/{path_type}/{path}/ban
   * @secure
   */
  moderationBan = (
    pathType: "id" | "name",
    path: Username | Reference,
    data: { until: string | null; reason: string },
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}/ban`,
      method: "PUT",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Unbans a user, if they were banned to begin with.
   *
   * @tags moderation
   * @name ModerationUnban
   * @request DELETE:/user/{path_type}/{path}/ban
   * @secure
   */
  moderationUnban = (
    pathType: "id" | "name",
    path: Username | Reference,
    params: RequestParams = {}
  ) =>
    this.request<void, void>({
      path: `/user/${pathType}/${path}/ban`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets the list of categories on a site.
   *
   * @tags category
   * @name CategoryGetList
   * @request GET:/category
   */
  categoryGetList = (params: RequestParams = {}) =>
    this.request<CategoryList, void>({
      path: `/category`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Gets the default category.
   *
   * @tags category
   * @name CategoryDefaultGet
   * @request GET:/category/default
   */
  categoryDefaultGet = (params: RequestParams = {}) =>
    this.request<CategoryDefault, void>({
      path: `/category/default`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Update (patch) the default category.
   *
   * @tags category
   * @name CategoryDefaultPatch
   * @request PATCH:/category/default
   * @secure
   */
  categoryDefaultPatch = (data: CategoryDefaultPatch, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/category/default`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets a category.
   *
   * @tags category
   * @name CategoryGet
   * @request GET:/category/id/{id}
   */
  categoryGet = (id: Reference, params: RequestParams = {}) =>
    this.request<Category, void>({
      path: `/category/id/${id}`,
      method: "GET",
      format: "json",
      ...params
    })

  /**
   * Update (patch) a category.
   *
   * @tags category
   * @name CategoryPatch
   * @request PATCH:/category/id/{id}
   * @secure
   */
  categoryPatch = (id: Reference, data: CategoryPatch, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/category/id/${id}`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the site's settings.
   *
   * @tags site
   * @name SiteSettingsGet
   * @request GET:/site/settings
   * @secure
   */
  siteSettingsGet = (params: RequestParams = {}) =>
    this.request<SiteSettings, void>({
      path: `/site/settings`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Update (patch) the site's settings.
   *
   * @tags site
   * @name SiteSettingsPatch
   * @request PATCH:/site/settings
   * @secure
   */
  siteSettingsPatch = (data: SiteSettingsPatch, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/site/settings`,
      method: "PATCH",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Gets the site's pending applications.
   *
   * @tags site, paginated
   * @name SiteApplicationGetList
   * @request GET:/site/application
   * @secure
   */
  siteApplicationGetList = (
    query?: { cursor?: number; limit?: number },
    params: RequestParams = {}
  ) =>
    this.request<ApplicationList, void>({
      path: `/site/application`,
      method: "GET",
      query: query,
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Gets an application.
   *
   * @tags site
   * @name SiteApplicationGet
   * @request GET:/site/application/{id}
   * @secure
   */
  siteApplicationGet = (id: Reference, params: RequestParams = {}) =>
    this.request<Application, void>({
      path: `/site/application/${id}`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Accepts an application.
   *
   * @tags site
   * @name SiteApplicationAccept
   * @request POST:/site/application/{id}
   * @secure
   */
  siteApplicationAccept = (id: Reference, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/site/application/${id}`,
      method: "POST",
      secure: true,
      ...params
    })

  /**
   * Rejects an application.
   *
   * @tags site
   * @name SiteApplicationReject
   * @request DELETE:/site/application/{id}
   * @secure
   */
  siteApplicationReject = (id: Reference, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/site/application/${id}`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Gets a backup of the site.
   *
   * @tags site, not-json
   * @name SiteBackupGet
   * @request GET:/site/backup
   * @secure
   */
  siteBackupGet = (params: RequestParams = {}) =>
    this.request<FileData, void>({
      path: `/site/backup`,
      method: "GET",
      secure: true,
      ...params
    })

  /**
   * Creates a new site.
   *
   * @tags site
   * @name SiteCreate
   * @request POST:/site/create
   * @secure
   */
  siteCreate = (data: CreateSiteSettings, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/site/create`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Starts the deletion process for the site. Requires additional email
   * validation for the process to complete.
   *
   * @tags site
   * @name SiteRequestDeletion
   * @request POST:/site/request-deletion
   * @secure
   */
  siteRequestDeletion = (params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/site/request-deletion`,
      method: "POST",
      secure: true,
      ...params
    })

  /**
   * Gets the site's current notifications.
   *
   * @tags site
   * @name SiteNotificationGet
   * @request GET:/site/notification
   * @secure
   */
  siteNotificationGet = (params: RequestParams = {}) =>
    this.request<NotificationList, void>({
      path: `/site/notification`,
      method: "GET",
      secure: true,
      format: "json",
      ...params
    })

  /**
   * Dismisses all of the site's notifications.
   *
   * @tags site
   * @name SiteNotificationDismissAll
   * @request DELETE:/site/notification
   * @secure
   */
  siteNotificationDismissAll = (params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/site/notification`,
      method: "DELETE",
      secure: true,
      ...params
    })

  /**
   * Sends a site newsletter.
   *
   * @tags site
   * @name SiteNewsletterSend
   * @request POST:/site/newsletter
   * @secure
   */
  siteNewsletterSend = (data: SiteNewsletter, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/site/newsletter`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })

  /**
   * Transfers the site master-admin status to another user.
   *
   * @tags site
   * @name SiteTransfer
   * @request POST:/site/transfer
   * @secure
   */
  siteTransfer = (data: SiteTransfer, params: RequestParams = {}) =>
    this.request<void, void>({
      path: `/site/transfer`,
      method: "POST",
      body: data,
      secure: true,
      type: ContentType.Json,
      ...params
    })
}
