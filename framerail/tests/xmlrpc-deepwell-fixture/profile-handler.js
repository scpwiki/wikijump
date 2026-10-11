import { fixtureState } from "./context.js"
import { sendRpcError } from "./response.js"

/**
 * @param {{
 *   rpcRequest: any
 *   request: import("node:http").IncomingMessage
 *   response: import("node:http").ServerResponse
 * }} input
 */
export const handleProfileRpc = ({ rpcRequest, request, response }) => {
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

  if (
    params.session_token === "fixture-authenticated-session-token" &&
    params.user === undefined
  ) {
    return {
      result: {
        type: "user_found",
        data: { user: fixtureState.authenticatedUser }
      }
    }
  }

  if (params.session_token !== undefined) return undefined

  // Only exact roster slugs resolve; inherited Object.prototype keys such as
  // "constructor" must keep failing closed instead of serving a bogus profile.
  const profile = Object.hasOwn(PROFILE_FIXTURES, params.user)
    ? PROFILE_FIXTURES[params.user]
    : undefined
  if (!profile) return undefined

  return {
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
          slug: params.user,
          name_changes_left: 0,
          last_name_change_added_at: "2026-01-01T00:00:00Z",
          last_renamed_at: null,
          email: "",
          email_verified_at: null,
          email_validation_info: null,
          email_validation_at: null,
          password: "",
          multi_factor_secret: null,
          multi_factor_recovery_codes: null,
          locales: ["en"],
          avatar_s3_hash: null,
          forum_signature: null,
          real_name: null,
          gender: null,
          birthday: null,
          location: null,
          biography: null,
          website: profile.website,
          user_page: null
        }
      }
    }
  }
}

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
 *   { user_id: number; name: string; website: string | null }
 * >}
 */
const PROFILE_FIXTURES = {
  guest: { user_id: 987654, name: "Guest", website: null },
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
  }
}
