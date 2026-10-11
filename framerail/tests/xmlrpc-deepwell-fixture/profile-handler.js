import { fixtureState } from "./context.js"
import { sendRpcError } from "./response.js"

/**
 * Session token that `session_get` resolves to the avatar-owning fixture
 * user.
 */
const AVATAR_OWNER_SESSION_TOKEN = "fixture-authenticated-session-token"

/** Stored avatar hash (SHA-512, serialized as Deepwell's byte array). */
const AVATAR_HASH_HEX = "ab".repeat(64)
const AVATAR_HASH_BYTES = Array.from(Buffer.from(AVATAR_HASH_HEX, "hex"))

/**
 * Route target whose public user view resolves to `user_missing`, matching
 * the missing-user branch of the public profile route.
 */
const MISSING_PROFILE_FIXTURE = "missing-profile-action-probe"

/** A 1x1 PNG served for the stored avatar hash through `blob_get`. */
const AVATAR_PNG = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII=",
  "base64"
)

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   response: import("node:http").ServerResponse
 * }} input
 */
export const handleProfileRpc = ({ rpcRequest, request, response }) => {
  if (rpcRequest.method === "blob_get") {
    if (rpcRequest.params !== AVATAR_HASH_HEX) return undefined
    return {
      result: {
        data: Array.from(AVATAR_PNG),
        mime: "image/png",
        size: AVATAR_PNG.length,
        created_at: "2026-01-01T00:00:00Z"
      }
    }
  }

  if (rpcRequest.method === "user_edit") {
    const params = rpcRequest.params ?? {}
    if (
      params.user !== fixtureState.authenticatedUser.user_id ||
      request.headers["x-deepwell-session-token"] !==
        "fixture-authenticated-session-token"
    ) {
      return undefined
    }
    fixtureState.pageWriteRequests.userEdit.push({
      paramKeys: Object.keys(params).sort()
    })
    if (params.email === "not-a-valid-email") {
      sendRpcError(response, rpcRequest.id, -32000, "The user's email is invalid")
      return { responded: true }
    }
    for (const key of [
      "name",
      "email",
      "locales",
      "real_name",
      "gender",
      "birthday",
      "location",
      "biography",
      "website",
      "user_page"
    ]) {
      if (Object.hasOwn(params, key)) fixtureState.authenticatedUser[key] = params[key]
    }
    return { result: fixtureState.authenticatedUser }
  }

  if (rpcRequest.method !== "user_view") return undefined

  const params = rpcRequest.params ?? {}
  if (params.site_id !== 6000005 || !Array.isArray(params.locales)) {
    return undefined
  }

  // The own-account profile follows the live fixture user so a saved edit is
  // reflected, and carries a stored avatar so the removal action renders and
  // the `blob_get` fixture above matches. The displayed name stays the avatar
  // fixture's own name.
  if (params.session_token === AVATAR_OWNER_SESSION_TOKEN && params.user === undefined) {
    return {
      result: {
        type: "user_found",
        data: {
          user: {
            ...fixtureState.authenticatedUser,
            name: PROFILE_FIXTURES["avatar-probe"].name,
            avatar_s3_hash: AVATAR_HASH_BYTES
          }
        }
      }
    }
  }

  if (params.session_token !== undefined) return undefined

  // A named missing target keeps the public profile route's HTTP 200
  // missing-user body observable in the browser.
  if (params.user === MISSING_PROFILE_FIXTURE) {
    return { result: { type: "user_missing" } }
  }

  // Only exact roster slugs resolve; inherited Object.prototype keys such as
  // "constructor" must keep failing closed instead of serving a bogus profile.
  const profile = Object.hasOwn(PROFILE_FIXTURES, params.user)
    ? PROFILE_FIXTURES[params.user]
    : undefined
  if (!profile) return undefined

  return userFound({ ...profile, slug: params.user })
}

/**
 * @param {{
 *   user_id: number
 *   name: string
 *   website: string | null
 *   slug: string
 *   avatar?: boolean
 *   email?: string
 *   birthday?: string | null
 *   biography?: string | null
 * }} profile
 */
const userFound = (profile) => ({
  result: {
    type: "user_found",
    data: {
      user: {
        user_id: profile.user_id,
        user_type: "regular",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: null,
        deleted_at: null,
        from_wikidot: false,
        name: profile.name,
        slug: profile.slug,
        name_changes_left: 0,
        last_name_change_added_at: "2026-01-01T00:00:00Z",
        last_renamed_at: null,
        email: profile.email ?? "",
        email_verified_at: null,
        email_validation_info: null,
        email_validation_at: null,
        password: "",
        multi_factor_secret: null,
        multi_factor_recovery_codes: null,
        locales: ["en"],
        avatar_s3_hash: profile.avatar ? AVATAR_HASH_BYTES : null,
        forum_signature: null,
        real_name: null,
        gender: null,
        birthday: profile.birthday ?? null,
        location: null,
        biography: profile.biography ?? null,
        website: profile.website,
        user_page: null
      }
    }
  }
})

/**
 * Public profile fixtures for the website-link contract. `guest` keeps the
 * historical blank-website profile; `website-frozen-probe` mirrors the
 * value frozen in issue #2150's live Wikidot evidence; `website-probe`
 * renders a generic schemeless domain; `website-unsafe-probe` renders an
 * unsafe scheme so the browser can observe both branches of
 * `normalizePublicWebsiteUrl`.
 *
 * @type {Record<
 *   string,
 *   {
 *     user_id: number
 *     name: string
 *     website: string | null
 *     email?: string
 *     birthday?: string | null
 *     biography?: string | null
 *   }
 * >}
 */
const PROFILE_FIXTURES = {
  guest: {
    user_id: 987654,
    name: "Guest",
    website: null,
    email: "private-email-marker-2139@example.test",
    birthday: "private-birthday-marker-2139",
    biography: "private-biography-marker-2139"
  },
  "website-frozen-probe": {
    user_id: 987657,
    name: "Website Frozen Probe",
    website: "scp-wiki.wikidot.com/seekgull"
  },
  "website-probe": { user_id: 987655, name: "Website Probe", website: "example.org" },
  "website-unsafe-probe": {
    user_id: 987656,
    name: "Website Unsafe Probe",
    website: ["javascript", "alert(1)"].join(":")
  },
  "profile-action-probe": {
    user_id: 987658,
    name: "Profile Action Probe",
    website: null
  },
  // Owned by the authenticated fixture session (user 6000008) and stored with an
  // avatar, so the own and public profiles both render the avatar image.
  "avatar-probe": {
    user_id: 6000008,
    name: "Avatar Probe",
    website: null,
    avatar: true
  }
}
