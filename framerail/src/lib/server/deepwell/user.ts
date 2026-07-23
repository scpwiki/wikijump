import { limitLocalePreferences } from "$lib/locales"
import { client } from "$lib/server/deepwell"
import { startBlobUpload, uploadToPresignUrl } from "./file"

import type { Nullable, Optional, UserModel, UserType } from "$lib/types"
import type { RequestContext } from "./request-context"

/* ----- User View ----- */
interface UserViewFound {
  type: "user_found"
  data: {
    user: UserModel
  }
}
interface UserViewMissing {
  type: "user_missing"
  data: undefined
}
export async function userView(
  siteId: number,
  locales: string[],
  sessionToken: Optional<string>,
  username?: string
): Promise<UserViewFound | UserViewMissing> {
  return client.request("user_view", {
    site_id: siteId,
    session_token: sessionToken,
    locales,
    user: username
  })
}

/* ----- User Edit ----- */
interface UserEditParams {
  name?: Optional<string>
  email?: Optional<string>
  emailVerified?: Optional<boolean>
  password?: Optional<string>
  locales?: Optional<string[]>
  avatar?: Optional<File>
  realName?: Optional<Nullable<string>>
  gender?: Optional<Nullable<string>>
  birthday?: Optional<Nullable<string>>
  location?: Optional<Nullable<string>>
  biography?: Optional<Nullable<string>>
  website?: Optional<Nullable<string>>
  userPage?: Optional<Nullable<string>>
  bypassFilter?: boolean
}

const directUserEditFields = [
  ["name", "name"],
  ["email", "email"],
  ["emailVerified", "email_verified"],
  ["password", "password"]
] as const

const nullableUserEditFields = [
  ["realName", "real_name"],
  ["gender", "gender"],
  ["location", "location"],
  ["biography", "biography"],
  ["website", "website"],
  ["userPage", "user_page"]
] as const

export async function userEdit(
  userId: number,
  userIpAddr: string,
  params: UserEditParams,
  requestContext: RequestContext
): Promise<UserModel> {
  const data: Record<string, unknown> = {
    bypass_filter: params.bypassFilter ?? false
  }

  for (const [paramName, rpcName] of directUserEditFields) {
    const value = params[paramName]
    if (value !== undefined) data[rpcName] = value
  }
  for (const [paramName, rpcName] of nullableUserEditFields) {
    const value = params[paramName]
    if (value !== undefined) data[rpcName] = value || null
  }

  if (params.birthday !== undefined) {
    data.birthday =
      params.birthday === null || isNaN(Date.parse(params.birthday))
        ? null
        : params.birthday
  }
  if (
    Array.isArray(params.locales) &&
    params.locales.every((v) => typeof v === "string")
  ) {
    data.locales = limitLocalePreferences(params.locales)
  }
  if (params.avatar instanceof File && params.avatar.type.startsWith("image/")) {
    const presign = await startBlobUpload(userId, params.avatar.size)
    await uploadToPresignUrl(presign.presign_url, params.avatar)
    data.avatar_uploaded_blob_id = presign.pending_blob_id
  } else if (params.avatar !== undefined && params.avatar === null) data.avatar = null

  return client.request(
    "user_edit",
    {
      user: userId,
      ip_address: userIpAddr,
      ...data
    },
    requestContext
  )
}

/* ----- User Create ----- */
interface UserCreate {
  user_id: number
  slug: string
}

interface UserCreateInput {
  userType: UserType
  name: string
  email: string
  locales: string[]
  password: string
  ipAddress: string
  bypassFilter?: boolean
  bypassEmailVerification?: boolean
}

export async function userCreate({
  userType,
  name,
  email,
  locales,
  password,
  ipAddress,
  bypassFilter = false,
  bypassEmailVerification = false
}: UserCreateInput): Promise<UserCreate> {
  return client.request("user_create", {
    user_type: userType,
    name,
    email,
    locales,
    password,
    ip_address: ipAddress,
    bypass_filter: bypassFilter,
    bypass_email_verification: bypassEmailVerification
  })
}
