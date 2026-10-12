import assert from "node:assert/strict";
import test from "node:test";
import { parseFragment } from "parse5";
import {
  buildCorpusDiscussionManifest,
  canonicalCorpusPayloadJson,
  discussionSha256,
  resolveObservedAvatarTemplates,
} from "../src/corpus-discussion-manifest.mjs";

function fixture() {
  const prefix = "<script>WIKIREQUEST.info.siteId = 101;</script>";
  const threadDocument = Buffer.from(
    `${prefix}<div class="forum-breadcrumbs"><a href="/forum/c-301/pages">Discussions / Pages</a></div><script>WIKIDOT.forumThreadId = 401;</script>`,
  );
  const forum = Buffer.from(
    `${prefix}<div class="forum-group"><div class="head"><div class="title">Discussions</div><div class="description"></div></div><table><tr><td><a href="/forum/c-301/pages">Pages</a><div class="description">Public pages</div></td></tr></table></div>`,
  );
  const user = {
    type: "User",
    id: 201,
    name: "First User",
    slug: "first-user",
  };
  return {
    sourceSite: "source-site",
    slug: "ordinary-page",
    siteId: 1000000001,
    pageId: 3000000001,
    sourceReference: "https://source-site.wikidot.com/forum/t-401/example",
    pageDom: Buffer.from(
      `${prefix}<a id="discuss-button" href="/forum/t-401/example">Discuss (2)</a>`,
    ),
    threadDocument,
    forumVisibleDocument: forum,
    forumAllDocument: forum,
    pageMeta: { fullname: "ordinary-page", comments: 2 },
    membership: {
      complete: true,
      post_ids: [501, 502],
      unconfirmed_post_ids: [],
    },
    corpusPosts: [501, 502].map((id, i) => ({
      id,
      status: "active",
      fullname: "ordinary-page",
      reply_to: i ? 501 : null,
      created_at: "2026-01-01T00:00:00Z",
      title: "Example",
      html: "<p>Body</p>",
      content: "Body",
    })),
    observation: {
      source_site: "source-site",
      source_site_id: 101,
      source_document_sha256: discussionSha256(threadDocument),
      observed_at: "2026-10-12T00:00:00Z",
      thread: {
        id: 401,
        title: "Example",
        description: "Page discussion",
        post_count: 2,
        created_at: "2026-01-01T00:00:00Z",
        created_by: { type: "WikidotUser", id: null, name: "Wikidot" },
      },
      posts: [501, 502].map((id, i) => ({
        id,
        parent_id: i ? 501 : null,
        created_at: "2026-01-01T00:00:00Z",
        title: "Example",
        rendered_html: '<div class="content"><p>Body</p></div>',
        created_by: user,
        edited_at: null,
        edited_by: null,
      })),
    },
    provenance: { membership_run_id: "frozen-unit" },
  };
}

test("complete source discussion preserves global IDs and parent graph", () => {
  const result = buildCorpusDiscussionManifest(fixture());
  assert.deepEqual(
    result.params.source.posts.map((p) => [p.id, p.parent_id]),
    [
      [501, null],
      [502, 501],
    ],
  );
  assert.equal(result.params.source.category.id, 301);
  assert.deepEqual(result.params.source.thread.creator, { type: "system" });
  assert.equal(result.params.source.actors.length, 1);
});

test("corpus canonical payload hashing preserves Unicode and Python separators", () => {
  assert.equal(
    canonicalCorpusPayloadJson({ z: [1, null], a: "日本語" }),
    '{"a": "日本語", "z": [1, null]}',
  );
});

test("source author edits are observations rather than invented historical revisions", () => {
  const input = fixture();
  input.observation.posts[1].edited_at = "2026-01-02T01:00:00+01:00";
  input.observation.posts[1].edited_by = {
    type: "User",
    id: 202,
    name: "Editor",
    slug: "editor",
  };
  const post = buildCorpusDiscussionManifest(input).params.source.posts[1];
  assert.deepEqual(post.edit, {
    user_id: 202,
    created_at: "2026-01-02T00:00:00.000Z",
  });
});

test("source identity, incomplete membership, source edits and graph cycles fail closed", () => {
  for (const mutate of [
    (x) => {
      x.membership.complete = false;
    },
    (x) => {
      x.membership.post_ids.pop();
    },
    (x) => {
      x.observation.posts[0].created_by.id = null;
    },
    (x) => {
      x.observation.posts[0].rendered_html =
        '<div class="content"><p>Changed live content</p></div>';
    },
    (x) => {
      x.observation.posts[0].parent_id = 502;
      x.corpusPosts[0].reply_to = 502;
    },
    (x) => {
      x.sourceReference = "https://other.wikidot.com/forum/t-401/example";
    },
    (x) => {
      x.observation.source_document_sha256 = "a".repeat(64);
    },
    (x) => {
      x.corpusPosts[0].created_at = "2026-01-01T00:00:00";
    },
  ]) {
    const input = fixture();
    mutate(input);
    assert.throws(
      () => buildCorpusDiscussionManifest(input),
      /discussion import:/u,
    );
  }
});

function avatar(id, base, label = "Observed User") {
  return `<span class="printuser avatarhover"><a><img src="${base}/avatar.php?userid=${id}&amp;size=small&amp;timestamp=123" style="background-image:url(${base}/userkarma.php?u=${id})" alt="${label}"></a></span>`;
}

test("two observed starred-user avatar attributes resolve against exact live authority", () => {
  for (const [id, origin] of [
    [201, "https://www.wikidot.com"],
    [202, "http://www.wikidot.com"],
  ]) {
    const saved = avatar(id, "(%%%URL_AVATARS%%%)");
    const live = parseFragment(avatar(id, origin));
    assert.equal(
      resolveObservedAvatarTemplates(saved, live),
      resolveObservedAvatarTemplates(avatar(id, origin), live),
    );
  }
});

test("avatar placeholders do not erase unrelated changes or expand arbitrary tokens", () => {
  for (const [saved, live] of [
    [
      avatar(201, "(%%%URL_AVATARS%%%)"),
      avatar(202, "https://www.wikidot.com"),
    ],
    [avatar(201, "(%%%URL_AVATARS%%%)"), avatar(201, "https://other.example")],
    [
      avatar(201, "(%%%URL_AVATARS%%%)", "Original"),
      avatar(201, "https://www.wikidot.com", "Changed"),
    ],
    ["<p>(%%%URL_AVATARS%%%)</p>", "<p>https://www.wikidot.com</p>"],
  ]) {
    assert.notEqual(
      resolveObservedAvatarTemplates(saved, parseFragment(live)),
      resolveObservedAvatarTemplates(live, parseFragment(live)),
    );
  }
});

test("body user references carry observed identity without creating a full profile", () => {
  const input = fixture();
  const html =
    '<p>Thanks <span class="printuser"><a href="http://www.wikidot.com/user:info/body-user" onclick="WIKIDOT.page.listeners.userInfo(202); return false;">Body User</a></span></p>';
  input.corpusPosts[0].html = html;
  input.corpusPosts[0].content = "Thanks [[user body-user]]";
  input.observation.posts[0].rendered_html = `<div class="content">${html}</div>`;
  const result = buildCorpusDiscussionManifest(input);
  assert.deepEqual(result.params.source.posts[0].referenced_user_ids, [202]);
  assert.deepEqual(result.params.source.actors[1], {
    id: 202,
    name: "Body User",
    slug: "body-user",
  });
  assert.equal(result.params.source.actors[1].created_at, undefined);
});

test("client attribute serialization order does not conceal attribute value changes", () => {
  const input = fixture();
  input.corpusPosts[0].html = '<p class="text" id="body">Body</p>';
  input.observation.posts[0].rendered_html =
    '<div class="content"><p id="body" class="text">Body</p></div>';
  assert.doesNotThrow(() => buildCorpusDiscussionManifest(input));
  input.observation.posts[0].rendered_html =
    '<div class="content"><p id="changed" class="text">Body</p></div>';
  assert.throws(
    () => buildCorpusDiscussionManifest(input),
    /rendered content differs/u,
  );
});

test("cached body identity stays scoped to its revision after a current author rename", () => {
  const input = fixture();
  const html =
    '<p><span class="printuser"><a href="http://www.wikidot.com/user:info/old-name" onclick="WIKIDOT.page.listeners.userInfo(201); return false;">Old Name</a></span></p>';
  input.corpusPosts[0].html = html;
  input.corpusPosts[0].content = "[[user old-name]]";
  input.observation.posts[0].rendered_html = `<div class="content">${html}</div>`;
  const result = buildCorpusDiscussionManifest(input);
  assert.equal(result.params.source.actors[0].name, "First User");
  assert.equal(
    result.params.source.posts[0].observed_users[0].name,
    "Old Name",
  );
  input.observation.posts[1].created_by = {
    ...input.observation.posts[1].created_by,
    name: "Contradictory Current Name",
  };
  assert.throws(
    () => buildCorpusDiscussionManifest(input),
    /source author identity changes/u,
  );
});
