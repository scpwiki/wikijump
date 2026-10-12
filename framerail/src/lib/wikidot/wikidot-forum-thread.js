const THREAD_POSTS_MODULE = "forum/ForumViewThreadPostsModule"
const THREAD_LISTENERS = "WIKIDOT.modules.ForumViewThreadModule.listeners"

const POST_FOLD = /^togglePostFold\(event,([1-9][0-9]*)\)$/u
const POST_OPTIONS = /^togglePostOptions\(event,([1-9][0-9]*)\)$/u
const UPDATE_LIST =
  /^WIKIDOT\.modules\.ForumViewThreadPostsModule\.listeners\.updateList\(([1-9][0-9]*)\)$/u

/** @typedef {{ threadId: number | null | undefined }} ForumThreadParameters */

/**
 * Reproduce the read-only part of Wikidot's ForumViewThreadModule and
 * ForumViewThreadPostsModule scripts for a renderer-owned thread body
 * without executing its inline handlers:
 *
 * - Reveal `#thread-container-posts` once, as `initOnce` does;
 * - Fold and unfold one post or all posts (`.folded` on `div.post`);
 * - Toggle the thread options row and a post's options from its template;
 * - Page the posts in place through `forum/ForumViewThreadPostsModule`.
 *   Editing, posting, history and moderation handlers stay inert until
 *   their modules exist.
 *
 * @param {HTMLElement} root
 * @param {ForumThreadParameters} parameters
 */
export const wikidotForumThread = (root, parameters) => {
  let threadId = parameters.threadId

  const showPosts = () => {
    const posts = root.querySelector("#thread-container-posts")
    if (posts instanceof HTMLElement) posts.style.display = "block"
  }

  /** @param {string} postId */
  const togglePostOptions = (postId) => {
    const options = root.querySelector(`#post-options-${postId}`)
    const template = root.querySelector("#post-options-template")
    if (!(options instanceof HTMLElement) || !template) return
    if (options.style.display !== "block") {
      options.innerHTML = template.innerHTML.replaceAll("%POST_ID%", postId)
      const permalink = root.querySelector("#post-options-permalink-template")
      for (const link of options.querySelectorAll("a")) {
        if (link.innerHTML === "permalink" && permalink) {
          link.setAttribute("href", `${permalink.innerHTML}${postId}`)
        }
      }
      options.style.display = "block"
    } else {
      options.style.display = "none"
    }
  }

  /** @param {Element} control */
  const toggleThreadOptions = (control) => {
    const options = root.querySelector("#thread-options-2")
    if (!(options instanceof HTMLElement)) return
    if (options.style.display === "none") {
      options.style.display = "block"
      control.innerHTML = "- Less Options"
    } else {
      options.style.display = "none"
      control.innerHTML = "+ More Options"
    }
  }

  /** @param {string} pageNo */
  const updateList = (pageNo) => {
    const posts = root.querySelector("#thread-container-posts")
    const normalizedThreadId = String(threadId)
    if (
      !posts ||
      !/^[1-9][0-9]*$/u.test(normalizedThreadId) ||
      !Number.isSafeInteger(Number(normalizedThreadId))
    ) {
      return
    }
    const window = root.ownerDocument.defaultView
    if (!window) return
    const form = new URLSearchParams({
      moduleName: THREAD_POSTS_MODULE,
      t: normalizedThreadId,
      pageNo
    })
    void window
      .fetch("/ajax-module-connector.php", {
        method: "POST",
        credentials: "same-origin",
        headers: { "content-type": "application/x-www-form-urlencoded" },
        body: form.toString()
      })
      .then((response) => (response.ok ? response.json() : null))
      .then((response) => {
        if (response?.status === "ok" && typeof response.body === "string") {
          posts.innerHTML = response.body
          root.querySelector("#thread-container")?.scrollIntoView()
        }
      })
      .catch(() => {})
  }

  /** @param {Event} event */
  const handleClick = (event) => {
    const target = event.target
    if (!(target instanceof Element)) return
    const control = target.closest("a[onclick]")
    if (!control || !root.contains(control)) return
    const onclick = control.getAttribute("onclick") ?? ""

    /** @type {(() => void) | null} */
    let action = null
    let match
    if ((match = POST_FOLD.exec(onclick))) {
      const post = root.querySelector(`#post-${match[1]}`)
      action = () => post?.classList.toggle("folded")
    } else if (onclick === `${THREAD_LISTENERS}.foldAll(event)`) {
      action = () => {
        for (const post of root.querySelectorAll("div.post")) post.classList.add("folded")
      }
    } else if (onclick === `${THREAD_LISTENERS}.unfoldAll(event)`) {
      action = () => {
        for (const post of root.querySelectorAll("div.post"))
          post.classList.remove("folded")
      }
    } else if (onclick === `${THREAD_LISTENERS}.toggleThreadOptions(event)`) {
      action = () => toggleThreadOptions(control)
    } else if ((match = POST_OPTIONS.exec(onclick))) {
      const postId = match[1]
      action = () => togglePostOptions(postId)
    } else if ((match = UPDATE_LIST.exec(onclick))) {
      const pageNo = match[1]
      action = () => updateList(pageNo)
    }
    if (!action) return

    event.preventDefault()
    event.stopPropagation()
    event.stopImmediatePropagation?.()
    action()
  }

  showPosts()
  root.addEventListener("click", handleClick, true)
  return {
    /** @param {ForumThreadParameters} nextParameters */
    update(nextParameters) {
      threadId = nextParameters.threadId
      showPosts()
    },
    destroy() {
      root.removeEventListener("click", handleClick, true)
    }
  }
}
