import { fixtureState, hasExactKeys, isFixtureSiteId } from "./context.js"

/** @param {{ rpcRequest: any }} input */
export const handleSiteRpc = ({ rpcRequest }) => {
  let result

  if (
    rpcRequest.method === "preload_view" &&
    (hasExactKeys(rpcRequest.params, ["site_id", "locales"]) ||
      hasExactKeys(rpcRequest.params, ["site_id", "locales", "session_token"])) &&
    isFixtureSiteId(rpcRequest.params?.site_id) &&
    Array.isArray(rpcRequest.params.locales) &&
    (rpcRequest.params.session_token === null ||
      rpcRequest.params.session_token === undefined ||
      ["fixture-session-token", "fixture-authenticated-session-token"].includes(
        rpcRequest.params.session_token
      ))
  ) {
    result = {
      site: {
        site_id: rpcRequest.params.site_id,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: null,
        deleted_at: null,
        from_wikidot: false,
        slug: rpcRequest.params.site_id === 6000006 ? "www" : "scp-wiki",
        name: rpcRequest.params.site_id === 6000006 ? "Wikijump" : "SCP Foundation",
        tagline:
          rpcRequest.params.site_id === 6000006
            ? "Fighting Ozone Pollution"
            : "Secure, Contain, Protect",
        description: "Fixture site",
        locale: "en",
        default_page: "main",
        top_bar_page: null,
        side_bar_page: null,
        preferred_domain: null,
        layout: rpcRequest.params.site_id === 6000006 ? "wikijump" : "wikidot",
        license: "cc-by-sa-3.0",
        forum_max_nest_level: 0,
        favicon_source: null,
        ios_icon_source: null,
        windows_tile_source: null,
        settings_revision: 4,
        welcome_page: "main",
        google_analytics_enabled: false,
        google_analytics_profile: null,
        show_top_toolbar: false,
        show_bottom_toolbar: false
      },
      site_settings: {
        revision: 4,
        welcome_page: "main",
        membership: {
          application_enabled: false,
          password_enabled: false
        },
        google_analytics: { enabled: false, profile: null },
        toolbars: { top: false, bottom: false, promote: true }
      },
      promoted_sites: [],
      site_file_domain: "scp-wiki.wjfiles.localhost",
      license_name: "CC BY-SA 3.0",
      license_url: "https://creativecommons.org/licenses/by-sa/3.0/",
      license_kind: "standard",
      license_html: null,
      user_session:
        rpcRequest.params.session_token === "fixture-authenticated-session-token"
          ? {
              session_id: 6000008,
              user: {
                user_id: 6000008,
                user_type: "regular",
                name: "Fixture Member",
                slug: "fixture-member",
                locales: ["en"]
              }
            }
          : null
    }
  } else if (
    rpcRequest.method === "admin_view" &&
    hasExactKeys(rpcRequest.params, ["site_id", "locales", "session_token"]) &&
    rpcRequest.params?.site_id === 6000005 &&
    Array.isArray(rpcRequest.params.locales) &&
    rpcRequest.params.session_token === "fixture-session-token"
  ) {
    result = {
      type: "site_found",
      data: {
        categories: [
          {
            category_id: 100,
            created_at: "2026-01-01T00:00:00Z",
            updated_at: null,
            site_id: 6000005,
            slug: "_default",
            layout: null,
            top_bar_page: null,
            side_bar_page: null,
            template_page_id: null,
            license: null,
            license_other: null,
            rating_enabled: null,
            rating_permission: null,
            rating_visibility: null,
            rating_type: null,
            per_page_discussion: null,
            settings_revision: 2,
            theme_kind: "built_in",
            theme_builtin_id: 1,
            theme_external_url: null,
            theme_custom_css: null,
            autonumber_enabled: false,
            autonumber_next: 1
          }
        ],
        page_templates: []
      }
    }
  } else if (
    rpcRequest.method === "category_get_all" &&
    rpcRequest.params?.site === "scp-wiki"
  ) {
    result = [{ slug: "_default" }, { slug: "nav" }]
  } else if (
    rpcRequest.method === "membership_application_list" &&
    hasExactKeys(rpcRequest.params, ["site_id"]) &&
    rpcRequest.params.site_id === 6000005
  ) {
    result = []
  } else if (
    rpcRequest.method === "site_get" &&
    hasExactKeys(rpcRequest.params, ["site"]) &&
    ["scp-wiki", "www", "missing-site"].includes(rpcRequest.params.site)
  ) {
    fixtureState.pageReadRequests.siteGet.push(rpcRequest.params)
    result =
      rpcRequest.params.site === "scp-wiki"
        ? { site_id: 6000005 }
        : rpcRequest.params.site === "www"
          ? { site_id: 6000006 }
          : null
  } else if (
    rpcRequest.method === "translate" &&
    hasExactKeys(rpcRequest.params, ["locales", "messages", "strip_message_keys"]) &&
    Array.isArray(rpcRequest.params.locales) &&
    typeof rpcRequest.params.messages === "object" &&
    rpcRequest.params.messages !== null &&
    Array.isArray(rpcRequest.params.strip_message_keys)
  ) {
    const isJapanese = rpcRequest.params.locales.some((locale) =>
      `${locale}`.toLowerCase().startsWith("ja")
    )
    const authLabels = isJapanese
      ? {
          "error-form.password-mismatch": "パスワードが一致しません。",
          "error-form.password-too-short": "パスワードは15文字以上で入力してください。"
        }
      : {
          specifier: "Email or Username",
          password: "Password",
          "password.placeholder": "Enter password...",
          "confirm-password": "Confirm Password",
          "mfa-code": "MFA code"
        }
    result = Object.fromEntries(
      Object.keys(rpcRequest.params.messages).map((key) => [key, authLabels[key] ?? key])
    )
  } else {
    return undefined
  }

  return { result }
}
