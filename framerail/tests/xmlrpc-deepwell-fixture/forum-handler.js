import { handleForumGetRpc } from "./forum-get-handler.js"
import { handleForumSelectRpc } from "./forum-select-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   response: import("node:http").ServerResponse
 * }} input
 */
export const handleForumRpc = (input) => {
  if (input.rpcRequest.method === "wikidot_forum_module") {
    const {
      site_id: siteId,
      module_name: moduleName,
      parameters
    } = input.rpcRequest.params ?? {}
    if (
      siteId !== 6000005 ||
      moduleName !== "forum/ForumStartModule" ||
      typeof parameters !== "object" ||
      parameters === null ||
      Array.isArray(parameters) ||
      Object.keys(parameters).some(
        (key) => key !== "hidden" || parameters[key] !== "true"
      )
    ) {
      return null
    }
    const { fixtureState } = input
    fixtureState.forumModuleRequests.push({
      headers: { ...input.request.headers },
      params: structuredClone(input.rpcRequest.params)
    })
    const hidden = parameters.hidden === "true"
    return {
      result: {
        status: "ok",
        // The RSS paragraph mirrors the fragment render_forum_start emits in
        // deepwell/src/services/render/forum_modules.rs; the Rust test pins that markup.
        body: hidden
          ? '<div data-forum-mode="hidden"><h2>Hidden</h2><h3>Per page discussions</h3><h3>Deleted threads</h3><h3>Changelog</h3><a href="/forum/start">Back to ordinary categories</a></div><p style="text-align: right"><span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span> RSS: <a href="/feed/forum/threads.xml">New threads</a> | <a href="/feed/forum/posts.xml">New posts</a></p>'
          : '<div data-forum-mode="ordinary"><h2>Changelog</h2><a href="/forum/start/hidden/show">Show hidden</a></div><p style="text-align: right"><span class="rss-icon"><img src="/common--theme/base/images/feed/feed-icon-14x14.png" alt="rss icon"/></span> RSS: <a href="/feed/forum/threads.xml">New threads</a> | <a href="/feed/forum/posts.xml">New posts</a></p>',
        js_include: []
      }
    }
  }
  return handleForumSelectRpc(input) ?? handleForumGetRpc(input)
}
