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
      [
        "fixture-session-token",
        "fixture-authenticated-session-token",
        "fixture-owner-session-token"
      ].includes(rpcRequest.params.session_token))
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
        rpcRequest.params.session_token === "fixture-owner-session-token"
          ? {
              session_id: 6000009,
              user: {
                user_id: 123,
                user_type: "regular",
                name: "Fixture Page Owner",
                slug: "fixture-page-owner",
                locales: ["en"]
              }
            }
          : rpcRequest.params.session_token === "fixture-authenticated-session-token"
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
    rpcRequest.method === "admin_view" &&
    hasExactKeys(rpcRequest.params, ["site_id", "locales", "session_token"]) &&
    isFixtureSiteId(rpcRequest.params?.site_id) &&
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
            site_id: rpcRequest.params.site_id,
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
    rpcRequest.method === "admin_view" &&
    hasExactKeys(rpcRequest.params, ["site_id", "locales", "session_token"]) &&
    isFixtureSiteId(rpcRequest.params?.site_id) &&
    Array.isArray(rpcRequest.params.locales) &&
    rpcRequest.params.session_token === "fixture-authenticated-session-token"
  ) {
    result = {
      type: "admin_permissions",
      data: { html: "Unauthorized: site manager permissions are required." }
    }
  } else if (
    rpcRequest.method === "admin_view" &&
    (hasExactKeys(rpcRequest.params, ["site_id", "locales"]) ||
      hasExactKeys(rpcRequest.params, ["site_id", "locales", "session_token"])) &&
    isFixtureSiteId(rpcRequest.params?.site_id) &&
    Array.isArray(rpcRequest.params.locales) &&
    (rpcRequest.params.session_token === null ||
      rpcRequest.params.session_token === undefined)
  ) {
    result = {
      type: "admin_permissions",
      data: { html: "Unauthorized: site manager permissions are required." }
    }
  } else if (
    rpcRequest.method === "category_get_all" &&
    rpcRequest.params?.site === "scp-wiki"
  ) {
    result = [{ slug: "_default" }, { slug: "nav" }]
  } else if (
    rpcRequest.method === "membership_application_list" &&
    hasExactKeys(rpcRequest.params, ["site_id"]) &&
    isFixtureSiteId(rpcRequest.params.site_id)
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
          "mfa-code": "MFA code",
          "user-profile-info.remove-avatar": "Remove profile image",
          "user-profile-info.remove-avatar.pending":
            "The profile image will be removed when you save.",
          "user-profile-info.keep-avatar": "Keep profile image"
        }
    const basePageActionLabels = isJapanese
      ? {
          delete: "削除",
          edit: "編集",
          files: "ファイル",
          history: "履歴",
          layout: "レイアウト",
          options: "オプション",
          parents: "親ページ",
          vote: "評価"
        }
      : {
          delete: "Delete",
          edit: "Edit",
          files: "Files",
          history: "History",
          layout: "Layout",
          options: "Options",
          parents: "Parents",
          vote: "Vote"
        }
    const pageActionLabels = isJapanese
      ? {
          "wiki-page-view-source": "ページソース",
          "wiki-page-action-append": "追加",
          "wiki-page-action-edit-sections": "セクションを編集",
          "wiki-page-action-edit-meta": "メタを編集",
          "wiki-page-action-watchers": "ウォッチャー",
          "wiki-page-action-backlinks": "バックリンク",
          "wiki-page-action-lock": "ページロック",
          "wiki-page-action-rename-move": "リネーム"
        }
      : {
          "wiki-page-view-source": "View Source",
          "wiki-page-action-append": "Append",
          "wiki-page-action-edit-sections": "Edit Sections",
          "wiki-page-action-edit-meta": "Edit Meta",
          "wiki-page-action-watchers": "Watchers",
          "wiki-page-action-backlinks": "Backlinks",
          "wiki-page-action-lock": "Lock Page",
          "wiki-page-action-rename-move": "Move"
        }
    const pageLabels = isJapanese
      ? {
          "wiki-page-layout": "ページレイアウト",
          "wiki-page-meta-tag-name": "メタタグ名",
          "wiki-page-meta-tag-content": "メタタグの内容"
        }
      : {
          "wiki-page-layout": "Page layout",
          "wiki-page-meta-tag-name": "Meta tag name",
          "wiki-page-meta-tag-content": "Meta tag content"
        }
    const voteLabels = isJapanese
      ? {
          "wiki-page-vote": "ページの評価",
          "wiki-page-vote.description": "このページの内容を評価してみましょう。",
          "wiki-page-vote.set": "投票する",
          "wiki-page-vote.remove": "投票を取り消す",
          "wiki-page-vote.list": "投票者一覧",
          "wiki-page-vote.score": "評価",
          "wiki-page-vote.like": "好き",
          "wiki-page-vote.dislike": "好きじゃない",
          "wiki-page-vote.cancel": "投票を取り消す",
          "wiki-page-vote.who-rated": "誰がこのページに評価したかを閲覧",
          "wiki-page-vote.user": "ユーザー"
        }
      : {
          "wiki-page-vote": "Page rating",
          "wiki-page-vote.description": "Simply rate contents of this page.",
          "wiki-page-vote.set": "Cast vote",
          "wiki-page-vote.remove": "Cancel vote",
          "wiki-page-vote.list": "List votes",
          "wiki-page-vote.score": "Rating",
          "wiki-page-vote.like": "I like it",
          "wiki-page-vote.dislike": "I don't like it",
          "wiki-page-vote.cancel": "Cancel my vote",
          "wiki-page-vote.who-rated": "Look who rated this page",
          "wiki-page-vote.user": "User"
        }
    const labels = { ...authLabels, ...basePageActionLabels, ...pageActionLabels }
    result = Object.fromEntries(
      Object.keys(rpcRequest.params.messages).map((key) => [
        key,
        labels[key] ?? pageLabels[key] ?? voteLabels[key] ?? key
      ])
    )
  } else {
    return undefined
  }

  return { result }
}
