import { limitLocalePreferences } from "$lib/locales"
import { client } from "$lib/server/deepwell"
import { startBlobUpload, uploadToPresignUrl } from "./file"

import type { Nullable, Optional, UserModel, UserType } from "$lib/types"
import type { RequestContext } from "../load/request-ctx"

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

function setNullableProfileField(
  data: Record<string, unknown>,
  key: string,
  value: Optional<Nullable<string>>
) {
  if (value !== undefined) {
    data[key] = value || null
  }
}

export async function userEdit(
  userId: number,
  userIpAddr: string,
  params: UserEditParams,
  requestContext: RequestContext
): Promise<UserModel> {
  const data: Record<string, unknown> = {
    bypass_filter: params.bypassFilter ?? false
  }
  if (params.name !== undefined) {
    data.name = params.name
  }
  if (params.email !== undefined) {
    data.email = params.email
  }
  if (params.emailVerified !== undefined) {
    data.email_verified = params.emailVerified
  }
  if (params.password !== undefined) {
    data.password = params.password
  }
  if (params.birthday !== undefined) {
    data.birthday =
      params.birthday === null || isNaN(Date.parse(params.birthday))
        ? null
        : params.birthday
  }
  setNullableProfileField(data, "real_name", params.realName)
  setNullableProfileField(data, "gender", params.gender)
  setNullableProfileField(data, "location", params.location)
  setNullableProfileField(data, "biography", params.biography)
  setNullableProfileField(data, "website", params.website)
  setNullableProfileField(data, "user_page", params.userPage)
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
export async function userCreate(
  userType: UserType,
  name: string,
  email: string,
  locales: string[],
  password: string,
  ipAddress: string,
  bypassFilter = false,
  bypassEmailVerification = false
): Promise<UserCreate> {
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
