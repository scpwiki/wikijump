import { Buffer } from "node:buffer"
import { request as httpRequest } from "node:http"
import { request as httpsRequest } from "node:https"

import {
  getXmlRpcWriteContext,
  getXmlRpcWritePrincipal,
  type XmlRpcWriteContext
} from "$lib/server/xmlrpc/authentication"
import {
  requestXmlRpcDeepwell as requestDeepwell,
  type DeepwellRequestContext
} from "$lib/server/xmlrpc/deepwell-client"
import {
  XmlRpcFault,
  type XmlRpcCall,
  type XmlRpcValue
} from "$lib/server/xmlrpc/protocol"

interface DeepwellCategory {
  slug: string
}

interface DeepwellSite {
  site_id: number
}

interface DeepwellPage {
  page_id?: number
  revision_id: number
  page_created_at: string
  page_updated_at: string | null
  page_revision_count: number
  revision_created_at: string
  revision_user_id: number
  title: string
  slug: string
  tags: string[]
  rating: number
  wikitext?: string | null
  compiled_body_html?: string | null
  compiled_body_styles?: string[] | null
}

interface DeepwellPageRevision {
  revision_number: number
  user_id: number
}

interface DeepwellBlobUpload {
  pending_blob_id: string
  presign_url: string
}

interface DeepwellFile {
  file_id: number
  file_created_at: string
  file_updated_at: string | null
  revision_id: number
  revision_created_at: string
  revision_user_id: number
  name: string
  data?: number[] | string | null
  mime: string
  size: number
  revision_comments: string
}

interface DeepwellForumPostSummary {
  comments: number
  commented_at: string | null
  commented_by: string | null
}

interface DeepwellPageView {
  type: string
  data?: {
    page?: { slug?: string }
  }
}

interface DeepwellForumPost {
  id: number
  fullname: string
  reply_to: number | null
  title: string
  content: string
  html: string
  created_by: string
  created_at: string
}

interface DeepwellParentRelationship {
  parent_page_id: number
}

type DeepwellStringParams = Record<string, string | string[] | undefined>
const MAX_XML_RPC_FILTER_VALUES = 100
const PAGE_SELECT_DECIMAL_RATING_PATTERN =
  /^[+-]?(?:(?:\d+(?:\.\d*)?)|(?:\.\d+))(?:[eE][+-]?\d+)?$/
export async function selectCategories(call: XmlRpcCall): Promise<string[]> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const categories = expectDeepwellCategories(
    await requestDeepwell("category_get_all", {
      site
    }),
    "category_get_all"
  )

  return categories.map((category) => category.slug)
}

function expectDeepwellCategories(value: unknown, method: string): DeepwellCategory[] {
  if (
    !Array.isArray(value) ||
    value.some(
      (category) =>
        !isXmlRpcStruct(category) ||
        typeof category.slug !== "string" ||
        category.slug.length === 0
    )
  ) {
    throw new XmlRpcFault(-32603, `Malformed Deepwell response: ${method}`)
  }

  return value as DeepwellCategory[]
}

function expectDeepwellStringArray(value: unknown, method: string): string[] {
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string")) {
    throw new XmlRpcFault(-32603, `Malformed Deepwell response: ${method}`)
  }

  return value
}

export async function selectTags(call: XmlRpcCall, requestIp: string): Promise<string[]> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const categories = getOptionalStructStringArray(params, "categories")
  const pages = getOptionalStructStringArray(params, "pages")

  if (categories && categories.length > MAX_XML_RPC_FILTER_VALUES) {
    throw new XmlRpcFault(
      -32602,
      `tags.select categories is limited to ${MAX_XML_RPC_FILTER_VALUES} entries`
    )
  }
  if (pages && pages.length > MAX_XML_RPC_FILTER_VALUES) {
    throw new XmlRpcFault(
      -32602,
      `tags.select pages is limited to ${MAX_XML_RPC_FILTER_VALUES} entries`
    )
  }

  const deepwellParams: {
    site: string
    categories?: string[]
    pages?: string[]
  } = { site }
  if (categories) {
    deepwellParams.categories = categories
  }
  if (pages) {
    deepwellParams.pages = pages
  }

  const principal = await getXmlRpcWritePrincipal(requestIp)

  return expectDeepwellStringArray(
    await requestDeepwell("page_tags_select", deepwellParams, {
      sessionToken: principal.sessionToken
    }),
    "page_tags_select"
  )
}

export async function selectPages(call: XmlRpcCall): Promise<string[]> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const deepwellParams: DeepwellStringParams & {
    site: string
    pagetype?: string
    categories?: string[]
    tags_any?: string[]
    tags_all?: string[]
    tags_none?: string[]
    parent?: string
    created_by?: string
    rating?: string
    order?: string
  } = { site }

  addOptionalValidatedStringField(
    deepwellParams,
    params,
    "pagetype",
    validatePageSelectType
  )
  addOptionalStringArrayField(deepwellParams, params, "categories", "pages.select")
  addOptionalStringArrayField(deepwellParams, params, "tags_any", "pages.select")
  addOptionalStringArrayField(deepwellParams, params, "tags_all", "pages.select")
  addOptionalStringArrayField(deepwellParams, params, "tags_none", "pages.select")
  addOptionalStringField(deepwellParams, params, "parent")
  addOptionalStringField(deepwellParams, params, "created_by")
  addOptionalValidatedStringField(
    deepwellParams,
    params,
    "rating",
    validatePageSelectRating
  )
  addOptionalValidatedStringField(
    deepwellParams,
    params,
    "order",
    validatePageSelectOrder
  )

  return expectDeepwellStringArray(
    await requestDeepwell("page_select", deepwellParams),
    "page_select"
  )
}

export async function getPagesMeta(
  call: XmlRpcCall,
  requestIp: string
): Promise<Record<string, XmlRpcValue>> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const pages = getRequiredStructStringArray(params, "pages")

  if (pages.length > 10) {
    throw new XmlRpcFault(-32602, "pages.get_meta pages is limited to 10 entries")
  }

  const siteId = await getDeepwellSiteId(site)
  if (pages.length === 0) {
    return {}
  }

  const entries: [string, XmlRpcValue][] = []
  for (const pageReference of pages) {
    const page = await getDeepwellPage(siteId, pageReference, false)
    if (!page || !(await canXmlRpcViewPage(siteId, page.slug, requestIp))) {
      continue
    }

    const [parentPage, creatorUserId, postSummary] = await Promise.all([
      getDeepwellDirectParentPage(siteId, page.slug),
      getDeepwellPageCreatorUserId(siteId, page),
      getDeepwellForumPostSummary(siteId, page.slug)
    ])
    entries.push([
      page.slug,
      buildXmlRpcPageMeta(page, parentPage?.slug ?? null, creatorUserId, postSummary)
    ])
  }

  return Object.fromEntries(entries)
}

export async function getPageOne(
  call: XmlRpcCall,
  requestIp: string
): Promise<Record<string, XmlRpcValue>> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const pageReference = getRequiredStructString(params, "page")
  const siteId = await getDeepwellSiteId(site)

  return buildXmlRpcPage(site, siteId, pageReference, requestIp)
}

export async function savePageOne(
  call: XmlRpcCall,
  requestIp: string
): Promise<Record<string, XmlRpcValue>> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const pageReference = getRequiredStructString(params, "page")
  const title = getOptionalStructString(params, "title")
  const content = getOptionalStructString(params, "content")
  const tags = getOptionalStructStringArray(params, "tags")
  const parentFullname = getOptionalStructString(params, "parent_fullname")
  const saveMode = getOptionalStructString(params, "save_mode") ?? "create_or_update"
  const renameAs = getOptionalStructString(params, "rename_as")
  const revisionComment =
    getOptionalStructString(params, "revision_comment") ?? "XML-RPC page save"

  if (!["create", "update", "create_or_update"].includes(saveMode)) {
    throw new XmlRpcFault(-32602, `Unsupported pages.save_one save_mode: ${saveMode}`)
  }

  const siteId = await getDeepwellSiteId(site)
  let page = await getDeepwellPage(siteId, pageReference, true)

  if (saveMode === "create" && page) {
    throw new XmlRpcFault(409, "Argument page invalid: page already exists")
  }
  if (saveMode === "update" && !page) {
    throw new XmlRpcFault(406, "Argument page invalid: page does not exist")
  }

  const currentPageReference = page?.slug ?? pageReference
  if (parentFullname !== null && parentFullname !== "-") {
    const parentPage = await getDeepwellPage(siteId, parentFullname, false)
    if (!parentPage) {
      throw new XmlRpcFault(
        406,
        "Argument parent_fullname invalid: parent page does not exist"
      )
    }
  }
  if (renameAs !== null && renameAs !== currentPageReference) {
    const targetPage = await getDeepwellPage(siteId, renameAs, false)
    if (targetPage) {
      throw new XmlRpcFault(409, "Argument rename_as invalid: page already exists")
    }
  }

  const writeContext = await getXmlRpcWriteContext(
    siteId,
    currentPageReference,
    requestIp
  )

  const createdPage = !page
  if (!page) {
    await requestDeepwell(
      "page_create",
      {
        site_id: siteId,
        wikitext: content ?? "",
        title: title ?? pageReference,
        alt_title: null,
        slug: pageReference,
        layout: "wikidot",
        revision_comments: revisionComment,
        user_id: writeContext.userId,
        ip_address: writeContext.ipAddress
      },
      writeContext
    )
    page = await requireDeepwellPage(siteId, pageReference, true)
  }

  const editBody: { wikitext?: string; title?: string; tags?: string[] } = {}
  if (!createdPage && content !== null) {
    editBody.wikitext = content
  }
  if (!createdPage && title !== null) {
    editBody.title = title
  }
  if (tags !== null) {
    editBody.tags = tags
  }

  if (Object.keys(editBody).length > 0) {
    await requestDeepwell(
      "page_edit",
      {
        site_id: siteId,
        page: page.slug,
        last_revision_id: page.revision_id,
        revision_comments: revisionComment,
        user_id: writeContext.userId,
        ip_address: writeContext.ipAddress,
        ...editBody
      },
      { ...writeContext, page: page.slug }
    )
    page = await requireDeepwellPage(siteId, page.slug, true)
  }

  if (parentFullname !== null) {
    await replaceDeepwellParents(siteId, page.slug, parentFullname, writeContext)
  }

  let finalPageReference = page.slug
  if (renameAs !== null && renameAs !== page.slug) {
    await requestDeepwell(
      "page_move",
      {
        site_id: siteId,
        page: page.slug,
        last_revision_id: page.revision_id,
        new_slug: renameAs,
        revision_comments: revisionComment,
        user_id: writeContext.userId,
        ip_address: writeContext.ipAddress
      },
      { ...writeContext, page: page.slug }
    )
    finalPageReference = renameAs
  }

  return buildXmlRpcPage(site, siteId, finalPageReference, requestIp, writeContext)
}

export async function selectFiles(call: XmlRpcCall): Promise<string[]> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const pageReference = getRequiredStructString(params, "page")
  const siteId = await getDeepwellSiteId(site)
  const page = await requireDeepwellPage(siteId, pageReference, false)
  const pageId = requireDeepwellPageId(page)
  const files = await getDeepwellPageFiles(siteId, pageId)

  return files.map((file) => file.name)
}

export async function getFilesMeta(
  call: XmlRpcCall
): Promise<Record<string, XmlRpcValue>> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const pageReference = getRequiredStructString(params, "page")
  const files = getRequiredStructStringArray(params, "files")

  if (files.length > 10) {
    throw new XmlRpcFault(-32602, "files.get_meta files is limited to 10 entries")
  }
  if (files.length === 0) {
    return {}
  }

  const siteId = await getDeepwellSiteId(site)
  const page = await requireDeepwellPage(siteId, pageReference, false)
  const pageId = requireDeepwellPageId(page)
  const entries = await Promise.all(
    files.map(async (fileName): Promise<[string, XmlRpcValue] | null> => {
      const file = await getDeepwellFile(siteId, pageId, fileName, false)
      return file ? [file.name, buildXmlRpcFileMeta(site, page.slug, file)] : null
    })
  )

  return Object.fromEntries(
    entries.filter((entry): entry is [string, XmlRpcValue] => entry !== null)
  )
}

export async function getFileOne(call: XmlRpcCall): Promise<Record<string, XmlRpcValue>> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const pageReference = getRequiredStructString(params, "page")
  const fileName = getRequiredStructString(params, "file")
  const siteId = await getDeepwellSiteId(site)
  const page = await requireDeepwellPage(siteId, pageReference, false)
  const pageId = requireDeepwellPageId(page)
  const file = await getDeepwellFile(siteId, pageId, fileName, true)
  if (!file) {
    throw new XmlRpcFault(406, "Argument file invalid: file does not exist")
  }

  return {
    ...buildXmlRpcFileMeta(site, page.slug, file),
    content: deepwellFileContentBase64(file)
  }
}

export async function saveFileOne(
  call: XmlRpcCall,
  requestIp: string
): Promise<Record<string, XmlRpcValue>> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const pageReference = getRequiredStructString(params, "page")
  const fileName = getRequiredStructString(params, "file")
  const content = getRequiredStructString(params, "content")
  const comment = getOptionalStructString(params, "comment")
  const saveMode = getOptionalStructString(params, "save_mode") ?? "create_or_update"
  const revisionComment =
    getOptionalStructString(params, "revision_comment") ?? comment ?? "XML-RPC file save"

  if (!["create", "update", "create_or_update"].includes(saveMode)) {
    throw new XmlRpcFault(-32602, `Unsupported files.save_one save_mode: ${saveMode}`)
  }

  const siteId = await getDeepwellSiteId(site)
  const page = await requireDeepwellPage(siteId, pageReference, false)
  const pageId = requireDeepwellPageId(page)
  const existing = await getDeepwellFile(siteId, pageId, fileName, false)

  if (saveMode === "create" && existing) {
    throw new XmlRpcFault(409, "Argument file invalid: file already exists")
  }
  if (saveMode === "update" && !existing) {
    throw new XmlRpcFault(406, "Argument file invalid: file does not exist")
  }

  const writeContext = await getXmlRpcWriteContext(siteId, page.slug, requestIp)
  const contentBytes = decodeXmlRpcBase64(content)
  const pendingBlobId = await uploadXmlRpcFileContent(contentBytes, writeContext)

  if (existing) {
    await requestDeepwell(
      "file_edit",
      {
        site_id: siteId,
        page_id: pageId,
        file_id: existing.file_id,
        last_revision_id: existing.revision_id,
        uploaded_blob_id: pendingBlobId,
        revision_comments: revisionComment,
        user_id: writeContext.userId,
        ip_address: writeContext.ipAddress,
        bypass_filter: true
      },
      writeContext
    )
  } else {
    await requestDeepwell(
      "file_create",
      {
        site_id: siteId,
        page_id: pageId,
        name: fileName,
        uploaded_blob_id: pendingBlobId,
        revision_comments: revisionComment,
        user_id: writeContext.userId,
        ip_address: writeContext.ipAddress,
        bypass_filter: true
      },
      writeContext
    )
  }

  const saved = await getDeepwellFile(siteId, pageId, fileName, false)
  if (!saved) {
    throw new XmlRpcFault(406, "Argument file invalid: file does not exist")
  }
  return buildXmlRpcFileMeta(site, page.slug, saved)
}

export async function selectPosts(call: XmlRpcCall): Promise<number[]> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const page = getOptionalStructString(params, "page")
  const replyTo = getOptionalStructStringOrInt(params, "reply_to")
  const createdBy = getOptionalStructString(params, "created_by")
  const siteId = await getDeepwellSiteId(site)

  const posts = await requestDeepwell("forum_post_select", {
    site_id: siteId,
    page: page ?? undefined,
    reply_to: replyTo ?? undefined,
    created_by: createdBy ?? undefined
  })
  if (
    !Array.isArray(posts) ||
    posts.some((post) => typeof post !== "number" || !Number.isInteger(post))
  ) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: forum_post_select")
  }

  return posts
}

export async function getPosts(call: XmlRpcCall): Promise<Record<string, XmlRpcValue>> {
  const params = getStructParam(call, 0, "params")
  const site = getRequiredStructString(params, "site")
  const posts = getRequiredStructStringOrIntArray(params, "posts")

  if (posts.length > 10) {
    throw new XmlRpcFault(-32602, "posts.get posts is limited to 10 entries")
  }

  const siteId = await getDeepwellSiteId(site)
  const result = expectDeepwellForumPosts(
    await requestDeepwell("forum_post_get", {
      site_id: siteId,
      posts
    }),
    "forum_post_get"
  )

  return Object.fromEntries(
    result.map((post) => [
      String(post.id),
      {
        id: post.id,
        fullname: post.fullname,
        reply_to: post.reply_to,
        title: post.title,
        content: post.content,
        html: post.html,
        created_by: post.created_by,
        created_at: post.created_at
      }
    ])
  )
}

function decodeXmlRpcBase64(content: string): Buffer {
  const normalized = content.replace(/\s+/g, "")
  if (
    normalized.length % 4 !== 0 ||
    !/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/u.test(normalized)
  ) {
    throw new XmlRpcFault(-32602, "Argument content invalid: malformed base64")
  }
  return Buffer.from(normalized, "base64")
}

async function uploadXmlRpcFileContent(
  content: Buffer,
  context: XmlRpcWriteContext
): Promise<string> {
  const upload = await requestDeepwell(
    "blob_upload",
    {
      user_id: context.userId,
      blob_size: content.length
    },
    context
  )
  if (!isDeepwellBlobUpload(upload)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: blob_upload")
  }

  await putPresignedBlob(upload.presign_url, content)
  return upload.pending_blob_id
}

async function putPresignedBlob(url: string, content: Buffer): Promise<void> {
  let signed: URL
  try {
    signed = new URL(url)
  } catch {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: blob_upload presign_url")
  }
  if (signed.protocol !== "http:" && signed.protocol !== "https:") {
    throw new XmlRpcFault(-32603, `Unsupported blob upload protocol: ${signed.protocol}`)
  }

  const target = localPresignConnectBase(signed) ?? signed
  const requestUrl = new URL(
    `${target.protocol}//${target.host}${signed.pathname}${signed.search}`
  )
  const transport = requestUrl.protocol === "https:" ? httpsRequest : httpRequest

  await new Promise<void>((resolve, reject) => {
    const request = transport(
      requestUrl,
      {
        method: "PUT",
        headers: {
          Host: signed.host,
          "Content-Length": String(content.length)
        }
      },
      (response) => {
        const chunks: Buffer[] = []
        response.on("data", (chunk: Buffer) => chunks.push(chunk))
        response.on("end", () => {
          if (
            response.statusCode &&
            response.statusCode >= 200 &&
            response.statusCode < 300
          ) {
            resolve()
            return
          }

          const body = Buffer.concat(chunks).toString("utf8")
          reject(
            new Error(
              `Blob upload failed: HTTP ${response.statusCode} connect=${requestUrl.origin} signed_host=${signed.host} ${body.slice(0, 500)}`
            )
          )
        })
      }
    )

    request.setTimeout(30_000, () => {
      request.destroy(
        new Error(
          `Blob upload timed out: connect=${requestUrl.origin} signed_host=${signed.host}`
        )
      )
    })
    request.on("error", (error) => reject(error))
    request.end(content)
  }).catch((error) => {
    const message = error instanceof Error ? error.message : String(error)
    throw new XmlRpcFault(-32603, message)
  })
}

function localPresignConnectBase(url: URL): URL | null {
  const localRewriteEnabled =
    process.env.WIKIJUMP_XMLRPC_LOCAL_FILE_UPLOAD === "1" ||
    process.env.FRAMERAIL_ENV === "local" ||
    process.env.NODE_ENV === "development"
  if (
    !localRewriteEnabled ||
    url.hostname !== "files" ||
    process.env.DEEPWELL_HOST === "deepwell"
  ) {
    return null
  }
  return new URL(`http://127.0.0.1:${url.port || "9000"}`)
}

async function buildXmlRpcPage(
  site: string,
  siteId: number,
  pageReference: string,
  requestIp: string,
  principal?: Pick<XmlRpcWriteContext, "sessionToken" | "userId">
): Promise<Record<string, XmlRpcValue>> {
  const pageMetadata = await getDeepwellPage(siteId, pageReference, false)
  if (!pageMetadata) {
    throw new XmlRpcFault(406, "Argument page invalid: page does not exist")
  }
  if (!(await canXmlRpcViewPage(siteId, pageMetadata.slug, requestIp, principal))) {
    throw new XmlRpcFault(403, "XML-RPC user is not allowed to view this page", 403)
  }

  const page = await requireDeepwellPage(siteId, pageMetadata.slug, true)
  const [parentPage, creatorUserId, postSummary] = await Promise.all([
    getDeepwellDirectParentPage(siteId, page.slug),
    getDeepwellPageCreatorUserId(siteId, page),
    getDeepwellForumPostSummary(siteId, page.slug)
  ])
  const parentFullname = parentPage?.slug ?? null
  const parentTitle = parentPage?.title ?? null
  const children = expectDeepwellStringArray(
    await requestDeepwell("page_select", {
      site,
      parent: page.slug
    }),
    "page_select"
  )

  return {
    ...buildXmlRpcPageMeta(page, parentFullname, creatorUserId, postSummary),
    parent_title: parentTitle,
    children: children.length,
    content: page.wikitext ?? "",
    html: page.compiled_body_html ?? ""
  }
}

async function canXmlRpcViewPage(
  siteId: number,
  page: string,
  requestIp: string,
  authenticatedPrincipal?: Pick<XmlRpcWriteContext, "sessionToken" | "userId">
): Promise<boolean> {
  const principal = authenticatedPrincipal ?? (await getXmlRpcWritePrincipal(requestIp))
  const view = await requestDeepwell(
    "page_view",
    {
      site_id: siteId,
      locales: [],
      session_token: principal.sessionToken,
      route: { slug: page, extra: "" }
    },
    {
      sessionToken: principal.sessionToken,
      siteId,
      page
    }
  )
  if (!isDeepwellPageView(view)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: page_view")
  }

  return view.type === "found" && view.data?.page?.slug === page
}

async function getDeepwellSiteId(site: string): Promise<number> {
  const deepwellSite = await requestDeepwell("site_get", { site })
  if (deepwellSite === null) {
    throw new XmlRpcFault(406, "Argument site invalid: site does not exist")
  }
  if (
    !isXmlRpcStruct(deepwellSite) ||
    typeof deepwellSite.site_id !== "number" ||
    !Number.isInteger(deepwellSite.site_id)
  ) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: site_get")
  }

  return (deepwellSite as unknown as DeepwellSite).site_id
}

async function getDeepwellPage(
  siteId: number,
  page: string,
  includeBody: boolean
): Promise<DeepwellPage | null> {
  const deepwellPage = await requestDeepwell("page_get", {
    site_id: siteId,
    page,
    details: {
      wikitext: includeBody,
      compiled_html: includeBody
    }
  })
  if (deepwellPage === null) {
    return null
  }
  if (!isDeepwellPage(deepwellPage, includeBody)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: page_get")
  }

  return deepwellPage
}

async function requireDeepwellPage(
  siteId: number,
  page: string,
  includeBody: boolean
): Promise<DeepwellPage> {
  const deepwellPage = await getDeepwellPage(siteId, page, includeBody)
  if (!deepwellPage) {
    throw new XmlRpcFault(406, "Argument page invalid: page does not exist")
  }
  return deepwellPage
}

function requireDeepwellPageId(page: DeepwellPage): number {
  if (typeof page.page_id !== "number" || !Number.isInteger(page.page_id)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: page_get")
  }
  return page.page_id
}

async function getDeepwellPageFiles(
  siteId: number,
  pageId: number
): Promise<DeepwellFile[]> {
  return expectDeepwellFiles(
    await requestDeepwell("page_get_files", {
      site_id: siteId,
      page_id: pageId,
      deleted: false
    }),
    "page_get_files",
    false
  )
}

async function getDeepwellFile(
  siteId: number,
  pageId: number,
  file: string,
  includeData: boolean
): Promise<DeepwellFile | null> {
  const deepwellFile = await requestDeepwell("file_get", {
    site_id: siteId,
    page_id: pageId,
    file,
    details: {
      data: includeData
    }
  })
  if (deepwellFile === null) {
    return null
  }
  if (!isDeepwellFile(deepwellFile, includeData)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: file_get")
  }
  return deepwellFile
}

async function getDeepwellDirectParentPage(
  siteId: number,
  page: string
): Promise<DeepwellPage | null> {
  const relationships = expectDeepwellParentRelationships(
    await requestDeepwell("parent_relationships_get", {
      site_id: siteId,
      page,
      relationship_type: "parents"
    }),
    "parent_relationships_get"
  )
  const parentPageId = relationships[0]?.parent_page_id
  if (parentPageId === undefined) {
    return null
  }

  const parentPage = await requestDeepwell("page_get_direct", {
    site_id: siteId,
    page_id: parentPageId,
    allow_deleted: false,
    details: {
      wikitext: false,
      compiled_html: false
    }
  })
  if (parentPage === null) {
    return null
  }
  if (!isDeepwellPage(parentPage, false)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: page_get_direct")
  }

  return parentPage
}

async function getDeepwellForumPostSummary(
  siteId: number,
  page: string
): Promise<DeepwellForumPostSummary> {
  const summary = await requestDeepwell("forum_post_page_summary", {
    site_id: siteId,
    page
  })
  if (!isDeepwellForumPostSummary(summary)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: forum_post_page_summary")
  }

  return summary
}

async function getDeepwellParentFullnames(
  siteId: number,
  page: string,
  context: DeepwellRequestContext
): Promise<string[]> {
  return expectDeepwellStringArray(
    await requestDeepwell(
      "parent_get_all",
      {
        site_id: siteId,
        page
      },
      { ...context, page }
    ),
    "parent_get_all"
  )
}

async function replaceDeepwellParents(
  siteId: number,
  page: string,
  parentFullname: string,
  context: XmlRpcWriteContext
): Promise<void> {
  const parents = await getDeepwellParentFullnames(siteId, page, context)
  const remove =
    parentFullname === "-"
      ? parents
      : parents.filter((parent) => parent !== parentFullname)
  const add =
    parentFullname === "-" || parents.includes(parentFullname) ? [] : [parentFullname]

  if (add.length === 0 && remove.length === 0) {
    return
  }

  await requestDeepwell(
    "parent_update",
    {
      site_id: siteId,
      child: page,
      add: add.length > 0 ? add : undefined,
      remove: remove.length > 0 ? remove : undefined
    },
    { ...context, page }
  )
}

async function getDeepwellPageCreatorUserId(
  siteId: number,
  page: DeepwellPage
): Promise<number> {
  if (typeof page.page_id !== "number" || !Number.isInteger(page.page_id)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: page_get")
  }

  const firstRevision = await requestDeepwell("page_revision_get", {
    site_id: siteId,
    page_id: page.page_id,
    revision_number: 0,
    details: {
      wikitext: false,
      compiled_html: false
    }
  })
  if (!isDeepwellPageRevision(firstRevision)) {
    throw new XmlRpcFault(-32603, "Malformed Deepwell response: page_revision_get")
  }

  return firstRevision.user_id
}

function isDeepwellPage(value: unknown, includeBody: boolean): value is DeepwellPage {
  return (
    isXmlRpcStruct(value) &&
    typeof value.revision_id === "number" &&
    Number.isInteger(value.revision_id) &&
    typeof value.page_created_at === "string" &&
    (typeof value.page_updated_at === "string" || value.page_updated_at === null) &&
    typeof value.page_revision_count === "number" &&
    Number.isInteger(value.page_revision_count) &&
    typeof value.revision_created_at === "string" &&
    typeof value.revision_user_id === "number" &&
    Number.isInteger(value.revision_user_id) &&
    typeof value.title === "string" &&
    typeof value.slug === "string" &&
    Array.isArray(value.tags) &&
    value.tags.every((tag) => typeof tag === "string") &&
    typeof value.rating === "number" &&
    Number.isFinite(value.rating) &&
    (!includeBody ||
      ((typeof value.wikitext === "string" || value.wikitext === null) &&
        (typeof value.compiled_body_html === "string" ||
          value.compiled_body_html === null) &&
        Array.isArray(value.compiled_body_styles) &&
        value.compiled_body_styles.every((style) => typeof style === "string")))
  )
}

function isDeepwellPageView(value: unknown): value is DeepwellPageView {
  return (
    isXmlRpcStruct(value) &&
    typeof value.type === "string" &&
    (value.type !== "found" ||
      (isXmlRpcStruct(value.data) &&
        isXmlRpcStruct(value.data.page) &&
        typeof value.data.page.slug === "string"))
  )
}

function isDeepwellBlobUpload(value: unknown): value is DeepwellBlobUpload {
  return (
    isXmlRpcStruct(value) &&
    typeof value.pending_blob_id === "string" &&
    value.pending_blob_id.length > 0 &&
    typeof value.presign_url === "string" &&
    value.presign_url.length > 0
  )
}

function isDeepwellFile(value: unknown, includeData: boolean): value is DeepwellFile {
  return (
    isXmlRpcStruct(value) &&
    typeof value.file_id === "number" &&
    Number.isInteger(value.file_id) &&
    typeof value.file_created_at === "string" &&
    (typeof value.file_updated_at === "string" || value.file_updated_at === null) &&
    typeof value.revision_id === "number" &&
    Number.isInteger(value.revision_id) &&
    typeof value.revision_created_at === "string" &&
    typeof value.revision_user_id === "number" &&
    Number.isInteger(value.revision_user_id) &&
    typeof value.name === "string" &&
    typeof value.mime === "string" &&
    typeof value.size === "number" &&
    Number.isInteger(value.size) &&
    typeof value.revision_comments === "string" &&
    (!includeData ||
      value.data === null ||
      value.data === undefined ||
      typeof value.data === "string" ||
      (Array.isArray(value.data) &&
        value.data.every(
          (byte) =>
            typeof byte === "number" && Number.isInteger(byte) && byte >= 0 && byte <= 255
        )))
  )
}

function isDeepwellForumPostSummary(value: unknown): value is DeepwellForumPostSummary {
  return (
    isXmlRpcStruct(value) &&
    typeof value.comments === "number" &&
    Number.isInteger(value.comments) &&
    (typeof value.commented_at === "string" || value.commented_at === null) &&
    (typeof value.commented_by === "string" || value.commented_by === null)
  )
}

function isDeepwellForumPost(value: unknown): value is DeepwellForumPost {
  return (
    isXmlRpcStruct(value) &&
    typeof value.id === "number" &&
    Number.isInteger(value.id) &&
    typeof value.fullname === "string" &&
    ((typeof value.reply_to === "number" && Number.isInteger(value.reply_to)) ||
      value.reply_to === null) &&
    typeof value.title === "string" &&
    typeof value.content === "string" &&
    typeof value.html === "string" &&
    typeof value.created_by === "string" &&
    typeof value.created_at === "string"
  )
}

function isDeepwellPageRevision(value: unknown): value is DeepwellPageRevision {
  return (
    isXmlRpcStruct(value) &&
    typeof value.revision_number === "number" &&
    Number.isInteger(value.revision_number) &&
    value.revision_number === 0 &&
    typeof value.user_id === "number" &&
    Number.isInteger(value.user_id)
  )
}

function expectDeepwellFiles(
  value: unknown,
  method: string,
  includeData: boolean
): DeepwellFile[] {
  if (!Array.isArray(value) || value.some((file) => !isDeepwellFile(file, includeData))) {
    throw new XmlRpcFault(-32603, `Malformed Deepwell response: ${method}`)
  }

  return value
}

function expectDeepwellForumPosts(value: unknown, method: string): DeepwellForumPost[] {
  if (!Array.isArray(value) || value.some((post) => !isDeepwellForumPost(post))) {
    throw new XmlRpcFault(-32603, `Malformed Deepwell response: ${method}`)
  }

  return value
}

function expectDeepwellParentRelationships(
  value: unknown,
  method: string
): DeepwellParentRelationship[] {
  if (
    !Array.isArray(value) ||
    value.some(
      (relationship) =>
        !isXmlRpcStruct(relationship) ||
        typeof relationship.parent_page_id !== "number" ||
        !Number.isInteger(relationship.parent_page_id)
    )
  ) {
    throw new XmlRpcFault(-32603, `Malformed Deepwell response: ${method}`)
  }

  return value as DeepwellParentRelationship[]
}

function buildXmlRpcPageMeta(
  page: DeepwellPage,
  parentFullname: string | null,
  creatorUserId: number,
  postSummary: DeepwellForumPostSummary
): Record<string, XmlRpcValue> {
  const creatorId = String(creatorUserId)
  const updaterId = String(page.revision_user_id)

  return {
    fullname: page.slug,
    title: page.title,
    created_at: page.page_created_at,
    created_by: creatorId,
    updated_at: page.page_updated_at ?? page.revision_created_at ?? page.page_created_at,
    updated_by: updaterId,
    parent_fullname: parentFullname,
    tags: page.tags,
    rating: Math.round(page.rating),
    revisions: page.page_revision_count,
    comments: postSummary.comments,
    commented_at: postSummary.commented_at,
    commented_by: postSummary.commented_by
  }
}

function buildXmlRpcFileMeta(
  site: string,
  page: string,
  file: DeepwellFile
): Record<string, XmlRpcValue> {
  return {
    size: file.size,
    comment: file.revision_comments,
    mime_type: file.mime,
    mime_description: file.mime,
    uploaded_by: String(file.revision_user_id),
    uploaded_at: file.revision_created_at,
    download_url: `/local--files/${site}/${page}/${encodeURIComponent(file.name)}`
  }
}

function deepwellFileContentBase64(file: DeepwellFile): string {
  if (file.data === null || file.data === undefined) {
    return ""
  }
  if (typeof file.data === "string") {
    return Buffer.from(file.data, "hex").toString("base64")
  }
  return Buffer.from(file.data).toString("base64")
}

export function expectParamCount(call: XmlRpcCall, expectedCount: number): void {
  if (call.params.length !== expectedCount) {
    throw new XmlRpcFault(
      -32602,
      `${call.methodName} expects ${expectedCount} parameter${expectedCount === 1 ? "" : "s"}`
    )
  }
}

export function expectUsersGetMeParams(call: XmlRpcCall): void {
  if (call.params.length === 0) {
    return
  }

  if (call.params.length === 1) {
    const value = call.params[0]
    if (
      (Array.isArray(value) && value.length === 0) ||
      (isXmlRpcStruct(value) && Object.keys(value).length === 0)
    ) {
      return
    }
  }

  throw new XmlRpcFault(
    -32602,
    "users.get_me expects no parameters or one empty struct/array parameter"
  )
}

export function getStringParam(call: XmlRpcCall, index: number, name: string): string {
  const value = call.params[index]
  if (typeof value !== "string") {
    throw new XmlRpcFault(-32602, `Expected string parameter: ${name}`)
  }
  return value
}

function getStructParam(
  call: XmlRpcCall,
  index: number,
  name: string
): Record<string, XmlRpcValue> {
  const value = call.params[index]
  if (!isXmlRpcStruct(value)) {
    throw new XmlRpcFault(-32602, `Expected struct parameter: ${name}`)
  }
  return value
}

function getRequiredStructString(
  params: Record<string, XmlRpcValue>,
  name: string
): string {
  const value = params[name]
  if (typeof value !== "string" || value.length === 0) {
    throw new XmlRpcFault(-32602, `Expected string field: ${name}`)
  }
  return value
}

function getRequiredStructStringArray(
  params: Record<string, XmlRpcValue>,
  name: string
): string[] {
  const value = getOptionalStructStringArray(params, name)
  if (value === null) {
    throw new XmlRpcFault(-32602, `Expected string array field: ${name}`)
  }
  return value
}

function getRequiredStructStringOrIntArray(
  params: Record<string, XmlRpcValue>,
  name: string
): string[] {
  const value = params[name]
  if (!Array.isArray(value)) {
    throw new XmlRpcFault(-32602, `Expected string or integer array field: ${name}`)
  }
  return value.map((entry) => {
    if (typeof entry === "string") {
      return entry
    }
    if (typeof entry === "number" && Number.isSafeInteger(entry)) {
      return String(entry)
    }
    throw new XmlRpcFault(-32602, `Expected string or integer array field: ${name}`)
  })
}

function getOptionalStructStringArray(
  params: Record<string, XmlRpcValue>,
  name: string
): string[] | null {
  const value = params[name]
  if (value === undefined || value === null) {
    return null
  }
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string")) {
    throw new XmlRpcFault(-32602, `Expected string array field: ${name}`)
  }
  return value
}

function getOptionalStructString(
  params: Record<string, XmlRpcValue>,
  name: string
): string | null {
  const value = params[name]
  if (value === undefined || value === null) {
    return null
  }
  if (typeof value !== "string") {
    throw new XmlRpcFault(-32602, `Expected string field: ${name}`)
  }
  return value
}

function getOptionalStructStringOrInt(
  params: Record<string, XmlRpcValue>,
  name: string
): string | null {
  const value = params[name]
  if (value === undefined || value === null) {
    return null
  }
  if (typeof value === "string") {
    return value
  }
  if (typeof value === "number" && Number.isSafeInteger(value)) {
    return String(value)
  }
  throw new XmlRpcFault(-32602, `Expected string or integer field: ${name}`)
}

function addOptionalStringField(
  target: DeepwellStringParams,
  params: Record<string, XmlRpcValue>,
  name: string
): void {
  const value = getOptionalStructString(params, name)
  if (value !== null) {
    target[name] = value
  }
}

function addOptionalValidatedStringField(
  target: DeepwellStringParams,
  params: Record<string, XmlRpcValue>,
  name: string,
  validate: (value: string) => void
): void {
  const value = getOptionalStructString(params, name)
  if (value !== null) {
    validate(value)
    target[name] = value
  }
}

function addOptionalStringArrayField(
  target: DeepwellStringParams,
  params: Record<string, XmlRpcValue>,
  name: string,
  methodName: string
): void {
  const value = getOptionalStructStringArray(params, name)
  if (value !== null) {
    if (value.length > MAX_XML_RPC_FILTER_VALUES) {
      throw new XmlRpcFault(
        -32602,
        `${methodName} ${name} is limited to ${MAX_XML_RPC_FILTER_VALUES} entries`
      )
    }
    target[name] = value
  }
}

function validatePageSelectType(value: string): void {
  switch (value.trim().toLowerCase()) {
    case "":
    case "*":
    case "all":
    case "normal":
    case "page":
    case "pages":
    case "hidden":
      return
    default:
      throw new XmlRpcFault(-32602, `Unsupported pages.select pagetype: ${value}`)
  }
}

function validatePageSelectRating(value: string): void {
  const trimmed = value.trim()
  if (trimmed === "") {
    return
  }
  const number = trimmed.replace(/^(>=|<=|!=|==|>|<|=)/, "").trim()
  if (
    number === "" ||
    !PAGE_SELECT_DECIMAL_RATING_PATTERN.test(number) ||
    !Number.isFinite(Number(number))
  ) {
    throw new XmlRpcFault(-32602, `Invalid pages.select rating filter: ${value}`)
  }
}

function validatePageSelectOrder(value: string): void {
  const trimmed = value.trim()
  if (trimmed === "") {
    return
  }

  const parts = trimmed.split(/\s+/)
  if (parts.length > 2) {
    throw new XmlRpcFault(-32602, `Invalid pages.select order expression: ${value}`)
  }

  const field = parts[0]?.toLowerCase()
  switch (field) {
    case "created_at":
    case "created":
    case "updated_at":
    case "updated":
    case "fullname":
    case "full_name":
    case "slug":
    case "name":
    case "title":
    case "rating":
    case "score":
      break
    default:
      throw new XmlRpcFault(-32602, `Unsupported pages.select order field: ${field}`)
  }

  const direction = parts[1]?.toLowerCase()
  if (
    direction !== undefined &&
    direction !== "asc" &&
    direction !== "ascending" &&
    direction !== "desc" &&
    direction !== "descending"
  ) {
    throw new XmlRpcFault(
      -32602,
      `Unsupported pages.select order direction: ${direction}`
    )
  }
}

export function getArrayParam(
  call: XmlRpcCall,
  index: number,
  name: string
): XmlRpcValue[] {
  const value = call.params[index]
  if (!Array.isArray(value)) {
    throw new XmlRpcFault(-32602, `Expected array parameter: ${name}`)
  }
  return value
}

export function isXmlRpcStruct(value: unknown): value is Record<string, XmlRpcValue> {
  return typeof value === "object" && value !== null && !Array.isArray(value)
}
