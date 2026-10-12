import assert from "node:assert/strict"
import { createHash } from "node:crypto"
import test from "node:test"

import {
  renderWikidotPageFiles,
  renderWikidotPageRevisionList,
  renderWikidotPageRevisionSource,
  renderWikidotPageRevisionVersion,
  renderWikidotEditMeta,
  renderWikidotWhoRated,
  renderWikidotViewSource
} from "../src/lib/server/ajax-module-connector-page-reads.js"

const ROBOTS_META = {
  name: "robots",
  content:
    "noindex,nofollow,nosnippet,noimageindex,noarchive,notranslate,max-snippet:0,max-image-preview:none,max-video-preview:0",
  all_pages: true
}

const sha256 = (value) => createHash("sha256").update(value).digest("hex")

test("renders the captured EditMeta empty-page body byte for byte", () => {
  assert.equal(
    sha256(renderWikidotEditMeta([ROBOTS_META])),
    "ba74cf64542995d0f634b0ec5a4b28b2a3bc445ef0d1c6c6a12284c8f24e55d1"
  )
})

test("renders site and page EditMeta rows in the captured DOM order", () => {
  const body = renderWikidotEditMeta([
    ROBOTS_META,
    { name: "codex-probe-b", content: "Second", all_pages: false },
    { name: "codex-probe-a", content: "Alpha & <Beta>", all_pages: false }
  ])
  assert.equal(
    sha256(body),
    "6027f151ed85262ec164df909916e1c0b0c038ea74017c84bbe37c1068e62d24"
  )
  assert.match(body, /content="Alpha &amp; &lt;Beta&gt;"/u)
  assert.match(body, /\(all pages\)/u)
})

test("renders the captured EditMeta update and delete result byte for byte", () => {
  assert.equal(
    sha256(
      renderWikidotEditMeta([
        ROBOTS_META,
        { name: "codex-probe-a", content: "Updated", all_pages: false }
      ])
    ),
    "2f40950e801e9a09a53677078e9bbc4e52b080c5c01b4feec540188b5308cb86"
  )
})

test("renders page source in the Wikidot client parsing boundary", () => {
  assert.equal(
    renderWikidotViewSource('alpha < beta & [[div title="x"]]'),
    '<h1>Page Source</h1>\n\n<div class="page-source">\n\talpha &lt; beta &amp; [[div title=&quot;x&quot;]]\n</div>\n'
  )
})

test("renders typed WhoRated votes in the exact wikidot.py DOM boundary", () => {
  assert.equal(
    renderWikidotWhoRated([
      {
        user: {
          "user-id": 11111,
          "user-name": "Voter <One>",
          "user-slug": "voter-one"
        },
        value: 1
      },
      {
        user: {
          "user-id": 22222,
          "user-name": "Voter Two",
          "user-slug": "voter-two"
        },
        value: -1
      }
    ]),
    '<h2>Users who rated:</h2>\n\n<div style="-moz-column-count:3"><span class="printuser avatarhover"><a href="https://www.wikidot.com/user:info/voter-one" onclick="WIKIDOT.page.listeners.userInfo(11111); return false;">Voter &lt;One&gt;</a></span>\n        <span style="color:#777">\n                    +              </span><br/><span class="printuser avatarhover"><a href="https://www.wikidot.com/user:info/voter-two" onclick="WIKIDOT.page.listeners.userInfo(22222); return false;">Voter Two</a></span>\n        <span style="color:#777">\n                    -              </span><br/></div>'
  )
})

test("renders the observed 66-byte empty WhoRated body without identities", () => {
  const body = renderWikidotWhoRated([])
  assert.equal(
    body,
    '<h2>Users who rated:</h2>\n\n<div style="-moz-column-count:3"></div>'
  )
  assert.equal(body.length, 66)
})

test("WhoRated rejects unevidenced vote values instead of widening its DOM", () => {
  assert.throws(
    () =>
      renderWikidotWhoRated([
        {
          user: {
            "user-id": 11111,
            "user-name": "Voter One",
            "user-slug": "voter-one"
          },
          value: 5
        }
      ]),
    /only observed plus\/minus/u
  )
})

test("renders attached files in the Wikidot client parsing boundary", () => {
  assert.equal(
    renderWikidotPageFiles("drafts:client parity", [
      {
        file_id: 402,
        name: 'alpha & "beta".txt',
        mime: "text/plain",
        size: 2048
      }
    ]),
    '<h1>Files</h1>\n<table class="page-files"><tbody><tr id="file-row-402"><td><a href="/local--files/drafts%3Aclient%20parity/alpha%20%26%20%22beta%22.txt">alpha &amp; &quot;beta&quot;.txt</a></td><td><span title="text/plain">text/plain</span></td><td>2048 Bytes</td></tr></tbody></table>\n<p class="manage-attachments-link" style="text-align: center;">\n<a href="javascript:;" onclick="WIKIDOT.page.listeners.filesClick(null)">Manage attachments</a>\n</p>'
  )
})

test("renders an empty file list without fabricating rows", () => {
  assert.equal(
    renderWikidotPageFiles("main", []),
    '<h1>Files</h1>\n<p>\n\t\tNo files attached to this page.\t\t\n\t</p>\n<p class="manage-attachments-link" style="text-align: center;">\n<a href="javascript:;" onclick="WIKIDOT.page.listeners.filesClick(null)">Manage attachments</a>\n</p>'
  )
})

test("renders typed Deepwell revisions in the wikidot.py history parser boundary", () => {
  const body = renderWikidotPageRevisionList([
    {
      revision_id: 1000003,
      revision_type: "move",
      revision_number: 2,
      created_at: "2023-11-14T22:46:40Z",
      user_id: 12345,
      author: {
        "user-id": 12345,
        "user-slug": "test-user",
        "user-name": "Test <User>",
        "user-karma": 5,
        "user-avatar-data":
          "http://www.wikidot.com/avatar.php?userid=12345&amp;size=small",
        "user-profile-url": "http://www.wikidot.com/user:info/test-user"
      },
      changes: ["slug"],
      comments: "Renamed & retained",
      wikitext: null,
      compiled_body_html: null
    },
    {
      revision_id: 1000002,
      revision_type: "regular",
      revision_number: 1,
      created_at: "2023-11-14T22:30:00Z",
      user_id: 45678,
      author: null,
      changes: ["title"],
      comments: null,
      wikitext: null,
      compiled_body_html: null
    }
  ])

  assert.match(body, /^<table class="page-history">/u)
  assert.match(body, /<tr id="revision-row-1000003"><td>3\.<\/td>/u)
  assert.match(body, /name="to" value="1000003" checked="checked"/u)
  assert.match(body, /name="from" value="1000002" checked="checked"/u)
  assert.match(body, /<td style="width: 5em" class="optionstd">/u)
  assert.match(body, /title="View page revision"[^>]*>V<\/a>/u)
  assert.match(body, /title="Revert to revision"[^>]*>R<\/a>/u)
  assert.match(
    body,
    /<span class="printuser avatarhover"><a href="https:\/\/www\.wikidot\.com\/user:info\/test-user"><img class="small" src="https:\/\/www\.wikidot\.com\/avatar\.php\?userid=12345&amp;amp;size=small" alt="Test &lt;User&gt;" style="background-image:url\(https:\/\/www\.wikidot\.com\/userkarma\.php\?u=12345\)"\/><\/a><a href="https:\/\/www\.wikidot\.com\/user:info\/test-user">Test &lt;User&gt;<\/a><\/span>/u
  )
  assert.match(
    body,
    /<td style="width: 5em"><input id="1000003" type="radio" name="from" value="1000003"/u
  )
  assert.match(
    body,
    /<span class="odate time_1700002000 format_%25e%20%25b%20%25Y%7Cagohover">14 Nov 2023 22:46<\/span>/u
  )
  assert.match(body, /Renamed &amp; retained/u)
  assert.match(body, /<span class="printuser deleted" data-id="45678"><\/span>/u)
  assert.match(
    body,
    /<td>rev\.<\/td><td>&nbsp;<\/td><td>flags<\/td><td>actions<\/td><td>by<\/td><td>date<\/td><td>comments<\/td>/u
  )
})

test("History flags and headers use the shared localized Wikidot contract", () => {
  const body = renderWikidotPageRevisionList(
    [
      {
        revision_id: 1000004,
        revision_type: "regular",
        revision_number: 3,
        created_at: "2023-11-14T22:46:40Z",
        user_id: 12345,
        author: {
          "user-id": 12345,
          "user-slug": "test-user",
          "user-name": "Test User",
          "user-karma": 0,
          "user-avatar-data":
            "http://www.wikidot.com/avatar.php?userid=12345&amp;size=small",
          "user-profile-url": "http://www.wikidot.com/user:info/test-user"
        },
        changes: ["wikitext"],
        comments: "Source",
        wikitext: null,
        compiled_body_html: null
      },
      {
        revision_id: 1000003,
        revision_type: "regular",
        revision_number: 2,
        created_at: "2023-11-14T22:30:00Z",
        user_id: 12345,
        author: {
          "user-id": 12345,
          "user-slug": "test-user",
          "user-name": "Test User",
          "user-karma": 0,
          "user-avatar-data":
            "http://www.wikidot.com/avatar.php?userid=12345&amp;size=small",
          "user-profile-url": "http://www.wikidot.com/user:info/test-user"
        },
        changes: ["tags"],
        comments: "Tags",
        wikitext: null,
        compiled_body_html: null
      }
    ],
    "ja-JP"
  )

  assert.equal((body.match(/<td\b[^>]*>/gu) ?? []).length, 21)
  assert.match(
    body,
    /<td>フラグ<\/td><td>アクション<\/td><td>by<\/td><td>日付<\/td><td>コメント<\/td>/u
  )
  assert.match(body, /title="コンテンツソースが変更されました">S<\/span>/u)
  assert.match(body, /title="タグが変更されました">A<\/span>/u)
  assert.equal((body.match(/class="optionstd"/gu) ?? []).length, 2)
  assert.equal((body.match(/>R<\/a>/gu) ?? []).length, 1)
  assert.doesNotMatch(body, /Metadata changed|class="spantip" title=""/u)
})

test("renders historical source and compiled HTML without exposing source markup", () => {
  const revision = {
    revision_id: 1000003,
    revision_type: "regular",
    revision_number: 2,
    created_at: "2023-11-14T22:46:40Z",
    user_id: 12345,
    author: null,
    changes: ["wikitext"],
    comments: null,
    wikitext: 'alpha < beta & [[div title="x"]]',
    compiled_body_html: '<p class="historical">Rendered revision</p>'
  }

  assert.equal(
    renderWikidotPageRevisionSource(revision),
    '<div class="page-source">alpha &lt; beta &amp; [[div title=&quot;x&quot;]]</div>'
  )
  assert.equal(
    renderWikidotPageRevisionVersion(revision),
    '<p class="historical">Rendered revision</p>'
  )
})
