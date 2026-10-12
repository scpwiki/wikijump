import { handleForumGetRpc } from "./forum-get-handler.js"
import { handleForumSelectRpc } from "./forum-select-handler.js"

/**
 * @param {{
 *   rpcRequest: any
 *   response: import("node:http").ServerResponse
 * }} input
 */
const FIXTURE_THREAD_ID = 17768232

/** @param {number} postId @param {string} title @param {string} replies */
const fixtureThreadPost = (postId, title, replies = "") =>
  `<div class="post-container" id="fpc-${postId}"><div class="post" id="post-${postId}"><div class="long"><div class="head"><div class="options"><a href="javascript:;" onclick="togglePostFold(event,${postId})" class="btn btn-default btn-small btn-sm">Fold</a></div><div class="title" id="post-title-${postId}">${title}</div></div><div class="content" id="post-content-${postId}"><p>${title} body</p></div><div class="options"><a href="javascript:;" onclick="togglePostOptions(event,${postId})" class="btn btn-default btn-small btn-sm">Options</a></div><div id="post-options-${postId}" class="options" style="display: none"></div></div><div class="short"><a class="options btn btn-default btn-mini btn-xs" href="javascript:;" onclick="togglePostFold(event,${postId})" c>Unfold</a><a class="title" href="javascript:;" onclick="togglePostFold(event,${postId})">${title}</a></div></div>${replies}</div>`

/** @param {number} page */
const fixtureThreadPager = (page) =>
  page === 1
    ? '<div class="pager"><span class="pager-no">page 1 of 2</span><span class="current">1</span><span class="target"><a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadPostsModule.listeners.updateList(2)">2</a></span><span class="target"><a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadPostsModule.listeners.updateList(2)">next &raquo;</a></span></div>'
    : '<div class="pager"><span class="pager-no">page 2 of 2</span><span class="target"><a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadPostsModule.listeners.updateList(1)">&laquo; previous</a></span><span class="target"><a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadPostsModule.listeners.updateList(1)">1</a></span><span class="current">2</span></div>'

/** @param {number} page */
const fixtureThreadPosts = (page) =>
  page === 1
    ? fixtureThreadPager(1) +
      fixtureThreadPost(7000001, "Root one", fixtureThreadPost(7000002, "Reply to one")) +
      fixtureThreadPager(1)
    : fixtureThreadPager(2) +
      fixtureThreadPost(7000011, "Root eleven") +
      fixtureThreadPager(2)

const fixtureThreadBody = () =>
  '<div class="forum-thread-box "><div class="description-block well">This is the discussion related to the wiki page <a href="/scp-9506">National Fog Safety Initiative</a>.</div>' +
  '<div class="options"><a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.unfoldAll(event)" class="btn btn-default btn-small btn-sm">Unfold All</a> ' +
  '<a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.foldAll(event)" class="btn btn-default btn-small btn-sm">Fold All</a> ' +
  '<a href="javascript:;" id="thread-toggle-options" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.toggleThreadOptions(event)" class="btn btn-default btn-small btn-sm"><i class="icon-plus"></i> More Options</a></div>' +
  '<div id="thread-options-2" class="options" style="display: none"><a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.editThreadMeta(event)" class="btn btn-default btn-small btn-sm">Edit Title &amp; Description</a></div>' +
  '<div id="thread-container" class="thread-container"><div id="thread-container-posts" style="display: none">' +
  fixtureThreadPosts(1) +
  '</div></div><div style="display:none" id="post-options-template"><a href="javascript:;" onclick="WIKIDOT.modules.ForumViewThreadModule.listeners.showPermalink(event,\'%POST_ID%\')" class="btn btn-default btn-small btn-sm">Permanent Link</a></div>' +
  `<div style="display:none" id="post-options-permalink-template">/forum/t-${FIXTURE_THREAD_ID}/national-fog-safety-initiative#post-</div></div>` +
  `<script type="text/javascript">WIKIDOT.forumThreadId = ${FIXTURE_THREAD_ID};</script>`

/**
 * @param {{
 *   rpcRequest: any
 *   response: import("node:http").ServerResponse
 *   fixtureState: any
 *   request: import("node:http").IncomingMessage
 * }} input
 */
const handleForumThreadRpc = (input) => {
  const {
    site_id: siteId,
    module_name: moduleName,
    parameters
  } = input.rpcRequest.params ?? {}
  if (siteId !== 6000005 || typeof parameters !== "object" || parameters === null) {
    return null
  }
  const keys = Object.keys(parameters).sort().join(",")
  if (
    moduleName === "forum/ForumViewThreadModule" &&
    keys === "t" &&
    parameters.t === String(FIXTURE_THREAD_ID)
  ) {
    input.fixtureState.forumModuleRequests.push({
      headers: { ...input.request.headers },
      params: structuredClone(input.rpcRequest.params)
    })
    return { result: { status: "ok", body: fixtureThreadBody(), js_include: [] } }
  }
  if (
    moduleName === "forum/ForumViewThreadPostsModule" &&
    keys === "pageNo,t" &&
    parameters.t === String(FIXTURE_THREAD_ID) &&
    ["1", "2"].includes(parameters.pageNo)
  ) {
    input.fixtureState.forumModuleRequests.push({
      headers: { ...input.request.headers },
      params: structuredClone(input.rpcRequest.params)
    })
    return {
      result: {
        status: "ok",
        body: fixtureThreadPosts(Number(parameters.pageNo)),
        js_include: []
      }
    }
  }
  return null
}

export const handleForumRpc = (input) => {
  if (input.rpcRequest.method === "wikidot_forum_module") {
    const thread = handleForumThreadRpc(input)
    if (thread) return thread
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
