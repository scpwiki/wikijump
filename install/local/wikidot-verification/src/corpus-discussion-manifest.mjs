import crypto from "node:crypto";
import { parseFragment, serialize } from "parse5";

export const discussionSha256 = (value) =>
  crypto.createHash("sha256").update(value).digest("hex");
const attr = (node, name) =>
  node?.attrs?.find((a) => a.name === name)?.value ?? null;
const hasClass = (node, name) =>
  (attr(node, "class") ?? "").split(/\s+/u).includes(name);
const children = (node) => node?.childNodes ?? [];
const all = (node, predicate) =>
  children(node).flatMap((child) => [
    ...(predicate(child) ? [child] : []),
    ...all(child, predicate),
  ]);
const text = (node) =>
  node?.nodeName === "#text" ? node.value : children(node).map(text).join("");
const fail = (message) => {
  throw new Error(`discussion import: ${message}`);
};
const integer = (value) =>
  Number.isSafeInteger(value) && value > 0 && value <= 2147483647;
const utc = (value) => {
  if (
    typeof value !== "string" ||
    !/(?:Z|[+-]\d{2}:\d{2})$/u.test(value) ||
    !Number.isFinite(Date.parse(value))
  )
    fail("source timestamps must declare their UTC offset");
  return new Date(value).toISOString();
};
const canonical = (value) =>
  Array.isArray(value)
    ? value.map(canonical)
    : value && typeof value === "object"
      ? Object.fromEntries(
          Object.keys(value)
            .sort()
            .map((key) => [key, canonical(value[key])]),
        )
      : value;
export const canonicalDiscussionJson = (value) =>
  JSON.stringify(canonical(value));
// The shared corpus hashes json.dumps(sort_keys=True, ensure_ascii=False)
// with Python's default comma/colon spacing, not compact JSON.
export const canonicalCorpusPayloadJson = (value) =>
  Array.isArray(value)
    ? `[${value.map(canonicalCorpusPayloadJson).join(", ")}]`
    : value && typeof value === "object"
      ? `{${Object.keys(value)
          .sort()
          .map(
            (key) =>
              `${JSON.stringify(key)}: ${canonicalCorpusPayloadJson(value[key])}`,
          )
          .join(", ")}}`
      : JSON.stringify(value);

// The high-level source client reserializes HTML through BeautifulSoup, which
// orders attributes differently from raw XML-RPC HTML. Compare parsed attribute
// names and values; preserve every node, value, style and internal text byte.
export function serializeDiscussionContent(node) {
  for (const item of [node, ...all(node, () => true)]) {
    item.attrs?.sort((a, b) => a.name.localeCompare(b.name, "en"));
  }
  return serialize(node).trim();
}

// XML-RPC retains URL_AVATARS in authored starred-user images. Resolve only
// the two observed user image attributes, using the corresponding live image
// authority. Text, links, other tokens, user IDs and timestamps remain exact.
export function resolveObservedAvatarTemplates(corpusHtml, liveContent) {
  const saved = parseFragment(corpusHtml);
  const savedImages = all(saved, (node) => node.tagName === "img");
  const liveImages = all(liveContent, (node) => node.tagName === "img");
  if (savedImages.length !== liveImages.length)
    return serializeDiscussionContent(saved);
  for (let index = 0; index < savedImages.length; index++) {
    const image = savedImages[index],
      live = liveImages[index];
    const src = attr(image, "src") ?? "";
    const match = src.match(
      /^\(%%%URL_AVATARS%%%\)(\/avatar\.php\?userid=([1-9]\d*)(?:&.*)?)$/u,
    );
    if (
      !match ||
      !hasClass(image.parentNode?.parentNode, "printuser") ||
      !hasClass(image.parentNode?.parentNode, "avatarhover")
    )
      continue;
    let url;
    try {
      url = new URL(attr(live, "src"));
    } catch {
      continue;
    }
    if (
      !["http:", "https:"].includes(url.protocol) ||
      url.host !== "www.wikidot.com" ||
      url.username ||
      url.password ||
      `${url.pathname}${url.search}` !== match[1]
    )
      continue;
    image.attrs.find((a) => a.name === "src").value =
      `${url.origin}${match[1]}`;
    const style = image.attrs.find((a) => a.name === "style");
    if (
      style?.value ===
      `background-image:url((%%%URL_AVATARS%%%)/userkarma.php?u=${match[2]})`
    )
      style.value = `background-image:url(${url.origin}/userkarma.php?u=${match[2]})`;
  }
  return serializeDiscussionContent(saved);
}

function root(buffer, sourceSiteId) {
  const html = buffer.toString("utf8");
  const site = html.match(/WIKIREQUEST\.info\.siteId\s*=\s*(\d+)/u)?.[1];
  if (Number(site) !== sourceSiteId)
    fail("source documents belong to different sites");
  return parseFragment(html);
}

function categoryFromDocuments(threadRoot, visibleRoot, allRoot, categoryId) {
  const groups = all(allRoot, (node) => hasClass(node, "forum-group"));
  const matching = all(
    allRoot,
    (node) =>
      node.tagName === "a" &&
      (attr(node, "href") ?? "").startsWith(`/forum/c-${categoryId}/`),
  );
  if (matching.length !== 1)
    fail(
      "category identity is missing or ambiguous in the complete forum structure",
    );
  const link = matching[0];
  let group = link;
  while (group && !hasClass(group, "forum-group")) group = group.parentNode;
  if (!group || !groups.includes(group))
    fail("category has no observed forum group");
  const head = children(group).find((node) => hasClass(node, "head"));
  const title = children(head).find((node) => hasClass(node, "title"));
  const description = children(head).find((node) =>
    hasClass(node, "description"),
  );
  let row = link;
  while (row && row.tagName !== "tr") row = row.parentNode;
  const categoryDescription = all(row, (node) => hasClass(node, "description"));
  if (categoryDescription.length !== 1 || !title || !description)
    fail("forum category metadata is incomplete");
  const groupLinks = all(
    group,
    (node) =>
      node.tagName === "a" &&
      /^\/forum\/c-[1-9]\d*\//u.test(attr(node, "href") ?? ""),
  );
  const groupName = text(title).trim();
  const name = text(link).trim();
  const breadcrumbs = all(threadRoot, (node) =>
    hasClass(node, "forum-breadcrumbs"),
  );
  const breadcrumb = all(
    breadcrumbs[0],
    (node) =>
      node.tagName === "a" &&
      (attr(node, "href") ?? "").startsWith(`/forum/c-${categoryId}/`),
  );
  if (
    breadcrumb.length !== 1 ||
    text(breadcrumb[0]).trim() !== `${groupName} / ${name}`
  )
    fail("thread and forum structure disagree about its category");
  return {
    id: categoryId,
    name,
    description: text(categoryDescription[0]).trim(),
    group_name: groupName,
    group_description: text(description).trim(),
    group_visible:
      all(
        visibleRoot,
        (node) =>
          node.tagName === "a" &&
          (attr(node, "href") ?? "").startsWith(`/forum/c-${categoryId}/`),
      ).length === 1,
    group_sort_index: groups.indexOf(group),
    sort_index: groupLinks.indexOf(link),
  };
}

/** Build a source snapshot from retained observations and immutable corpus posts.
 * No acquisition, actor login, rendering, or database mutation happens here.
 */
export function buildCorpusDiscussionManifest({
  sourceSite,
  slug,
  pageDom,
  threadDocument,
  forumVisibleDocument,
  forumAllDocument,
  observation,
  pageMeta,
  membership,
  corpusPosts,
  sourceReference,
  siteId,
  pageId,
  provenance,
}) {
  if (
    !Number.isSafeInteger(siteId) ||
    siteId <= 0 ||
    !Number.isSafeInteger(pageId) ||
    pageId <= 0
  )
    fail("target site and page IDs must be explicit");
  if (observation.source_site !== sourceSite || pageMeta.fullname !== slug)
    fail("source page identity mismatch");
  if (observation.source_document_sha256 !== discussionSha256(threadDocument))
    fail("thread document evidence hash mismatch");
  const pageRoot = root(pageDom, observation.source_site_id);
  const threadRoot = root(threadDocument, observation.source_site_id);
  const visibleRoot = root(forumVisibleDocument, observation.source_site_id);
  const allRoot = root(forumAllDocument, observation.source_site_id);
  const discuss = all(
    pageRoot,
    (node) => node.tagName === "a" && attr(node, "id") === "discuss-button",
  );
  if (discuss.length !== 1)
    fail("source page must expose one existing discussion link");
  const sourceUrl = new URL(sourceReference);
  if (
    sourceUrl.hostname !== `${sourceSite}.wikidot.com` ||
    sourceUrl.username ||
    sourceUrl.password ||
    !["https:", "http:"].includes(sourceUrl.protocol)
  )
    fail("discussion reference is outside the source site");
  if (
    new URL(attr(discuss[0], "href"), sourceUrl.origin).href !== sourceUrl.href
  )
    fail("discussion reference does not match the source page control");
  const threadId = Number(
    sourceUrl.pathname.match(/^\/forum\/t-([1-9]\d*)\//u)?.[1],
  );
  if (!integer(threadId) || threadId !== observation.thread.id)
    fail("observed thread ID mismatch");
  const scriptIds = all(
    threadRoot,
    (node) => node.tagName === "script",
  ).flatMap((node) =>
    [...text(node).matchAll(/WIKIDOT\.forumThreadId\s*=\s*(\d+)\s*;/gu)].map(
      (m) => Number(m[1]),
    ),
  );
  if (scriptIds.length !== 1 || scriptIds[0] !== threadId)
    fail("source document does not bind the observed thread");
  const breadcrumbs = all(threadRoot, (node) =>
    hasClass(node, "forum-breadcrumbs"),
  );
  const categories = all(
    breadcrumbs[0],
    (node) =>
      node.tagName === "a" &&
      /^\/forum\/c-[1-9]\d*\//u.test(attr(node, "href") ?? ""),
  );
  if (categories.length !== 1) fail("thread has no unique source category");
  const categoryId = Number(
    attr(categories[0], "href").match(/^\/forum\/c-(\d+)\//u)[1],
  );
  const category = categoryFromDocuments(
    threadRoot,
    visibleRoot,
    allRoot,
    categoryId,
  );
  if (
    membership.complete !== true ||
    (membership.unconfirmed_post_ids ?? []).length
  )
    fail("corpus post membership is incomplete");
  const ids = [...membership.post_ids].sort((a, b) => a - b);
  const observedIds = observation.posts.map((p) => p.id).sort((a, b) => a - b);
  if (
    ids.length > 1000 ||
    new Set(ids).size !== ids.length ||
    JSON.stringify(ids) !== JSON.stringify(observedIds) ||
    ids.length !== observation.thread.post_count ||
    pageMeta.comments !== ids.length
  )
    fail("source, page metadata, and corpus post inventories differ");
  const corpus = new Map(corpusPosts.map((p) => [p.id, p]));
  if (corpus.size !== ids.length || corpusPosts.length !== ids.length)
    fail("corpus post payloads do not match their complete membership");
  const actors = new Map();
  function actor(value) {
    if (
      value?.type !== "User" ||
      !integer(value.id) ||
      !value.name ||
      !/^[a-z0-9-]+$/u.test(value.slug ?? "")
    )
      fail("unsupported or incomplete source author identity");
    const data = { id: value.id, name: value.name, slug: value.slug };
    const prior = actors.get(data.id);
    if (
      prior &&
      canonicalDiscussionJson(prior) !== canonicalDiscussionJson(data)
    )
      fail("source author identity changes within one observation");
    actors.set(data.id, data);
    return data.id;
  }
  let creator;
  if (
    observation.thread.created_by?.type === "WikidotUser" &&
    observation.thread.created_by.id === null &&
    observation.thread.created_by.name === "Wikidot"
  )
    creator = { type: "system" };
  else
    creator = { type: "user", user_id: actor(observation.thread.created_by) };
  // Author metadata is current; a source post's compiled body can retain a
  // pre-rename identity. Keep that revision-scoped observation separate.
  for (const post of observation.posts) {
    actor(post.created_by);
    if (post.edited_at) actor(post.edited_by);
  }
  const posts = observation.posts
    .map((meta) => {
      const post = corpus.get(meta.id);
      if (
        post?.status !== "active" ||
        post.fullname !== slug ||
        post.reply_to !== meta.parent_id ||
        utc(post.created_at) !== utc(meta.created_at) ||
        post.title !== meta.title
      )
        fail(
          `post ${meta.id} source identity, parent, timestamp, or title differs from the corpus`,
        );
      const fragment = parseFragment(meta.rendered_html ?? "");
      const wrappers = children(fragment).filter(
        (node) => node.tagName && hasClass(node, "content"),
      );
      if (
        wrappers.length !== 1 ||
        serializeDiscussionContent(wrappers[0]) !==
          resolveObservedAvatarTemplates(post.html, wrappers[0])
      )
        fail(
          `post ${meta.id} rendered content differs; refresh its corpus observation before importing`,
        );
      const references = all(wrappers[0], (node) =>
        hasClass(node, "printuser"),
      ).map((node) => {
        const links = all(node, (child) => child.tagName === "a");
        const names = links.filter((link) =>
          children(link).some((child) => child.nodeName === "#text"),
        );
        if (names.length !== 1)
          fail(`post ${meta.id} body user identity is ambiguous`);
        const link = names[0];
        const id = Number(
          (attr(link, "onclick") ?? "").match(
            /^WIKIDOT\.page\.listeners\.userInfo\(([1-9]\d*)\); return false;$/u,
          )?.[1],
        );
        const profile = new URL(attr(link, "href"), sourceUrl.origin);
        const slug = profile.pathname.match(
          /^\/user:info\/([a-z0-9-]+)$/u,
        )?.[1];
        if (
          profile.hostname !== "www.wikidot.com" ||
          profile.username ||
          profile.password ||
          profile.search ||
          profile.hash
        )
          fail("post body user reference is outside Wikidot");
        const data = { type: "User", id, name: text(link), slug };
        if (!integer(id) || !data.name || !/^[a-z0-9-]+$/u.test(slug ?? ""))
          fail("incomplete body user identity");
        if (!actors.has(id)) actor(data);
        const images = all(node, (child) => child.tagName === "img");
        if (images.length > 1) fail("ambiguous source user avatar");
        return {
          id,
          name: data.name,
          slug,
          avatar_url: images.length ? attr(images[0], "src") : null,
        };
      });
      const bodyUsers = new Map();
      for (const user of references) {
        const prior = bodyUsers.get(user.id);
        if (
          prior &&
          canonicalDiscussionJson(prior) !== canonicalDiscussionJson(user)
        )
          fail(`post ${meta.id} has conflicting cached user identities`);
        bodyUsers.set(user.id, user);
      }
      return {
        id: meta.id,
        parent_id: meta.parent_id,
        referenced_user_ids: [
          ...new Set(references.map((user) => user.id)),
        ].sort((a, b) => a - b),
        observed_users: [...bodyUsers.values()].sort((a, b) => a.id - b.id),
        user_id: actor(meta.created_by),
        created_at: utc(meta.created_at),
        edit: meta.edited_at
          ? { user_id: actor(meta.edited_by), created_at: utc(meta.edited_at) }
          : null,
        title: post.title,
        wikitext: post.content,
      };
    })
    .sort((a, b) => a.id - b.id);
  const graph = new Map(posts.map((p) => [p.id, p.parent_id]));
  for (const post of posts) {
    const seen = new Set([post.id]);
    let parent = post.parent_id;
    while (parent !== null) {
      if (!graph.has(parent) || seen.has(parent))
        fail("post graph is incomplete or cyclic");
      seen.add(parent);
      parent = graph.get(parent);
    }
  }
  const evidence = {
    schema: "wikijump.corpus_discussion_source_evidence.v1",
    ...provenance,
    page_dom_sha256: discussionSha256(pageDom),
    thread_document_sha256: discussionSha256(threadDocument),
    forum_visible_document_sha256: discussionSha256(forumVisibleDocument),
    forum_all_document_sha256: discussionSha256(forumAllDocument),
    observation_sha256: discussionSha256(canonicalDiscussionJson(observation)),
  };
  const params = {
    site_id: siteId,
    page_id: pageId,
    source: {
      site: sourceSite,
      fullname: slug,
      reference: sourceUrl.href,
      evidence_sha256: discussionSha256(canonicalDiscussionJson(evidence)),
      observed_at: utc(observation.observed_at),
      category,
      thread: {
        id: threadId,
        title: observation.thread.title,
        description: observation.thread.description,
        created_at: utc(observation.thread.created_at),
        creator,
        post_count: posts.length,
      },
      actors: [...actors.values()].sort((a, b) => a.id - b.id),
      posts,
    },
  };
  return {
    schema: "wikijump.corpus_discussion_import_manifest.v1",
    params,
    evidence,
  };
}
