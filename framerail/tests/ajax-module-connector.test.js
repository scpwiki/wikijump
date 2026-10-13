import assert from "node:assert/strict"
import test from "node:test"

import { handleAjaxModuleConnectorRequest } from "../src/lib/server/ajax-module-connector.js"

const request = (form, options = {}) =>
  new Request("http://scp-wiki.local/ajax-module-connector.php", {
    method: options.method ?? "POST",
    headers: {
      "content-type": "application/x-www-form-urlencoded",
      ...(form.wikidot_token7 !== undefined
        ? { cookie: `wikidot_token7=${form.wikidot_token7}` }
        : {}),
      ...(options.headers ?? {})
    },
    body: options.method === "GET" ? undefined : new URLSearchParams(form)
  })

test("dispatches ListPages forms and returns the Wikidot JSON envelope", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "list/ListPagesModule",
      module_body: '[[div class="page"]]%%fullname%%[[/div]]',
      wikidot_token7: "client-token",
      category: "_default",
      name: "scp-173",
      perPage: "250",
      separate: "no",
      wrapper: "no"
    }),
    {
      siteId: 6000006,
      renderListPages: async (input) => {
        received = input
        return { body: '<div class="page">scp-173</div>' }
      }
    }
  )

  assert.equal(response.status, 200)
  assert.deepEqual(await response.json(), {
    status: "ok",
    body: '<div class="page">scp-173</div>'
  })
  assert.equal(response.headers.get("content-type"), "text/plain; charset=UTF-8")
  assert.deepEqual(received, {
    siteId: 6000006,
    moduleBody: '[[div class="page"]]%%fullname%%[[/div]]',
    parameters: {
      category: "_default",
      name: "scp-173",
      perPage: "250",
      separate: "no",
      wrapper: "no"
    }
  })
  assert.equal(response.headers.get("cache-control"), "no-store")
})

test("ListPages omits module_body for Deepwell's default row template", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: {
        "content-type": "application/x-www-form-urlencoded",
        cookie: "wikidot_token7=client-token"
      },
      body: "moduleName=list%2FListPagesModule&wikidot_token7=client-token"
    }),
    {
      siteId: 6000006,
      renderListPages: async (input) => {
        received = input
        return { body: "default-row" }
      }
    }
  )

  assert.equal(response.status, 200)
  assert.deepEqual(await response.json(), {
    status: "ok",
    body: "default-row"
  })
  assert.deepEqual(received, {
    siteId: 6000006,
    moduleBody: "",
    parameters: {}
  })
})

test("ListPages ignores unknown non-data-form selectors while recognized selectors apply", async () => {
  const calls = []
  const forms = [
    "moduleName=list%2FListPagesModule&module_body=body&category=one&unsupported_future_selector=ignored&wikidot_token7=client-token",
    "moduleName=list%2FListPagesModule&module_body=body&unsupported_future_selector=ignored&category=one&wikidot_token7=client-token"
  ]

  for (const form of forms) {
    const response = await handleAjaxModuleConnectorRequest(
      new Request("http://scp-wiki.local/ajax-module-connector.php", {
        method: "POST",
        headers: {
          "content-type": "application/x-www-form-urlencoded",
          cookie: "wikidot_token7=client-token"
        },
        body: form
      }),
      {
        siteId: 6000006,
        renderListPages: async (input) => {
          calls.push(input)
          return { body: "rows" }
        }
      }
    )
    assert.equal(response.status, 200)
    assert.deepEqual(await response.json(), { status: "ok", body: "rows" })
  }

  assert.deepEqual(calls, [
    { siteId: 6000006, moduleBody: "body", parameters: { category: "one" } },
    { siteId: 6000006, moduleBody: "body", parameters: { category: "one" } }
  ])
})

test("ListPages keeps the later URL-form value for duplicate scalar fields", async () => {
  const calls = []
  const renderListPages = async (input) => {
    calls.push(input)
    return { body: input.moduleBody }
  }
  const forms = [
    {
      body: "moduleName=list%2FListPagesModule&module_body=first&module_body=second&category=one&category=two&wikidot_token7=first-token&wikidot_token7=second-token",
      cookie: "wikidot_token7=second-token"
    },
    {
      body: "moduleName=list%2FListPagesModule&module_body=second&module_body=first&category=two&category=one&wikidot_token7=second-token&wikidot_token7=first-token",
      cookie: "wikidot_token7=first-token"
    }
  ]

  for (const { body: form, cookie } of forms) {
    const response = await handleAjaxModuleConnectorRequest(
      new Request("http://scp-wiki.local/ajax-module-connector.php", {
        method: "POST",
        headers: { "content-type": "application/x-www-form-urlencoded", cookie },
        body: form
      }),
      { siteId: 6000006, renderListPages }
    )
    assert.equal(response.status, 200)
    assert.deepEqual(await response.json(), {
      status: "ok",
      body: calls.at(-1).moduleBody
    })
  }

  assert.deepEqual(calls, [
    {
      siteId: 6000006,
      moduleBody: "second",
      parameters: { category: "two" }
    },
    {
      siteId: 6000006,
      moduleBody: "first",
      parameters: { category: "one" }
    }
  ])
})

test("ListPages rejects a mismatched public token with wrong_token7", async () => {
  const calls = []
  for (const { body, cookie } of [
    {
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=999999",
      cookie: "wikidot_token7=123456"
    },
    {
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=123456",
      cookie: "wikidot_token7=777777"
    },
    {
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=999999",
      cookie: "other-cookie=1"
    },
    {
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=999999",
      cookie: null
    }
  ]) {
    const headers = { "content-type": "application/x-www-form-urlencoded" }
    if (cookie !== null) headers.cookie = cookie
    const response = await handleAjaxModuleConnectorRequest(
      new Request("http://scp-wiki.local/ajax-module-connector.php", {
        method: "POST",
        headers,
        body
      }),
      {
        siteId: 6000006,
        renderListPages: async (input) => {
          calls.push(input)
          return { body: "rows" }
        }
      }
    )
    assert.equal(response.status, 200)
    const payload = await response.json()
    assert.equal(payload.status, "wrong_token7")
    assert.equal(payload.message, "no")
    assert.equal(payload.callbackIndex, null)
    assert.equal(typeof payload.CURRENT_TIMESTAMP, "number")
  }
  assert.equal(calls.length, 0)
})

test("ListPages rejects an omitted public token with wrong_token7 when cookie is present", async () => {
  const calls = []
  const response = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: {
        "content-type": "application/x-www-form-urlencoded",
        cookie: "wikidot_token7=123456"
      },
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%"
    }),
    {
      siteId: 6000006,
      renderListPages: async (input) => {
        calls.push(input)
        return { body: "rows" }
      }
    }
  )
  assert.equal(response.status, 200)
  const payload = await response.json()
  assert.equal(payload.status, "wrong_token7")
  assert.equal(payload.message, "no")
  assert.equal(payload.callbackIndex, null)
  assert.equal(typeof payload.CURRENT_TIMESTAMP, "number")
  assert.equal(calls.length, 0)
})

test("ListPages rejects a fully tokenless request with wrong_token7", async () => {
  const calls = []
  const response = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%"
    }),
    {
      siteId: 6000006,
      renderListPages: async (input) => {
        calls.push(input)
        return { body: "rows" }
      }
    }
  )
  assert.equal(response.status, 200)
  const payload = await response.json()
  assert.equal(payload.status, "wrong_token7")
  assert.equal(payload.message, "no")
  assert.equal(payload.callbackIndex, null)
  assert.equal(typeof payload.CURRENT_TIMESTAMP, "number")
  assert.equal(calls.length, 0)
})

test("ListPages accepts any echoed public token without an allowlist", async () => {
  const calls = []
  for (const token of ["123456", "777777"]) {
    const response = await handleAjaxModuleConnectorRequest(
      new Request("http://scp-wiki.local/ajax-module-connector.php", {
        method: "POST",
        headers: {
          "content-type": "application/x-www-form-urlencoded",
          cookie: `wikidot_token7=${token}`
        },
        body: `moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=${token}`
      }),
      {
        siteId: 6000006,
        renderListPages: async (input) => {
          calls.push(input)
          return { body: "rows" }
        }
      }
    )
    assert.equal(response.status, 200)
    assert.deepEqual(await response.json(), { status: "ok", body: "rows" })
  }
  assert.equal(calls.length, 2)
})

test("ListPages validates the selected duplicate public token", async () => {
  const calls = []
  const mismatched = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: {
        "content-type": "application/x-www-form-urlencoded",
        cookie: "wikidot_token7=123456"
      },
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=123456&wikidot_token7=999999"
    }),
    {
      siteId: 6000006,
      renderListPages: async (input) => {
        calls.push(input)
        return { body: "rows" }
      }
    }
  )
  assert.equal((await mismatched.json()).status, "wrong_token7")

  const matched = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: {
        "content-type": "application/x-www-form-urlencoded",
        cookie: "wikidot_token7=123456"
      },
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=999999&wikidot_token7=123456"
    }),
    {
      siteId: 6000006,
      renderListPages: async (input) => {
        calls.push(input)
        return { body: "rows" }
      }
    }
  )
  assert.deepEqual(await matched.json(), { status: "ok", body: "rows" })
  assert.equal(calls.length, 1)
})

test("ListPages selects the first public token from a duplicate cookie header", async () => {
  const cases = [
    { cookie: "wikidot_token7=123456; wikidot_token7=invalid", status: "ok" },
    { cookie: "wikidot_token7=invalid; wikidot_token7=123456", status: "wrong_token7" }
  ]
  for (const { cookie, status } of cases) {
    const response = await handleAjaxModuleConnectorRequest(
      new Request("http://scp-wiki.local/ajax-module-connector.php", {
        method: "POST",
        headers: {
          "content-type": "application/x-www-form-urlencoded",
          cookie
        },
        body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=123456"
      }),
      {
        siteId: 6000006,
        renderListPages: async () => ({ body: "rows" })
      }
    )
    assert.equal((await response.json()).status, status)
  }
})

test("ListPages echoes callbackIndex on the wrong_token7 envelope", async () => {
  const response = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: {
        "content-type": "application/x-www-form-urlencoded",
        cookie: "wikidot_token7=123456"
      },
      body: "moduleName=list%2FListPagesModule&module_body=%%fullname%%&wikidot_token7=999999&callbackIndex=42"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render on token mismatch")
    }
  )
  const payload = await response.json()
  assert.equal(payload.status, "wrong_token7")
  assert.equal(payload.message, "no")
  assert.equal(payload.callbackIndex, "42")
  assert.equal(typeof payload.CURRENT_TIMESTAMP, "number")
})

test("ListPages validates the public token but excludes controls from render parameters", async () => {
  const calls = []
  for (const { body, cookie } of [
    {
      body: "callbackIndex=17&wikidot_token7=client-token&name=scp-173&moduleName=list%2FListPagesModule&module_body=%%fullname%%",
      cookie: "wikidot_token7=client-token"
    },
    {
      body: "module_body=%%fullname%%&moduleName=list%2FListPagesModule&name=scp-173&wikidot_token7=other-token&callbackIndex=91",
      cookie: "wikidot_token7=other-token"
    }
  ]) {
    const response = await handleAjaxModuleConnectorRequest(
      new Request("http://scp-wiki.local/ajax-module-connector.php", {
        method: "POST",
        headers: { "content-type": "application/x-www-form-urlencoded", cookie },
        body
      }),
      {
        siteId: 6000006,
        renderListPages: async (input) => {
          calls.push(input)
          return { body: "scp-173" }
        }
      }
    )
    assert.deepEqual(await response.json(), { status: "ok", body: "scp-173" })
  }

  assert.deepEqual(calls, [
    {
      siteId: 6000006,
      moduleBody: "%%fullname%%",
      parameters: { name: "scp-173" }
    },
    {
      siteId: 6000006,
      moduleBody: "%%fullname%%",
      parameters: { name: "scp-173" }
    }
  ])
})

test("ListPages retains fail-closed boundaries for dynamic selectors and invalid UTF-8", async () => {
  const dynamic = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: "moduleName=list%2FListPagesModule&module_body=ok&_field=1"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render dynamic selector shape")
    }
  )
  assert.equal(dynamic.status, 200)
  assert.deepEqual(await dynamic.json(), {
    status: "not_ok",
    message: "Unsupported AJAX module shape: list/ListPagesModule"
  })

  const malformed = await handleAjaxModuleConnectorRequest(
    {
      method: "POST",
      headers: new Headers({ "content-type": "application/x-www-form-urlencoded" }),
      body: new ReadableStream({
        start(controller) {
          controller.enqueue(
            new Uint8Array([
              ...new TextEncoder().encode(
                "moduleName=list%2FListPagesModule&module_body="
              ),
              0xff
            ])
          )
          controller.close()
        }
      })
    },
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render malformed UTF-8")
    }
  )
  assert.equal(malformed.status, 400)
  assert.deepEqual(await malformed.json(), {
    status: "not_ok",
    message: "The encoded data was not valid for encoding utf-8"
  })
})

test("dispatches only the observed MembersListModule read shape", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "membership/MembersListModule",
      group: "",
      order: "joined",
      page: "0",
      wikidot_token7: "client-token"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderMembersList: async (input) => {
        received = input
        return { status: "ok", body: '\n<div id="ml-12345">members</div>' }
      }
    }
  )

  const body = await response.json()
  assert.equal(body.status, "ok")
  assert.equal(body.body, '\n<div id="ml-12345">members</div>')
  assert.equal(body.callbackIndex, null)
  assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
  assert.deepEqual(body.cssInclude, [])
  assert.deepEqual(body.jsInclude, [])
  assert.deepEqual(received, {
    siteId: 6000006,
    parameters: { group: "", order: "joined", page: "0" }
  })

  const failed = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "membership/MembersListModule",
      group: "",
      order: "joined",
      page: "1"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderMembersList: async () => ({ status: "not_ok", body: "" })
    }
  )
  const failedBody = await failed.json()
  assert.equal(failedBody.status, "not_ok")
  assert.equal(failedBody.body, "")
  assert.equal(failedBody.callbackIndex, null)
  assert.equal(Number.isInteger(failedBody.CURRENT_TIMESTAMP), true)
  assert.deepEqual(failedBody.cssInclude, [])
  assert.deepEqual(failedBody.jsInclude, [])
})

test("returns the frozen UserInfo no-target error for the observed empty identity shape", async () => {
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "profile/UserInfoModule",
      user_id: "",
      callbackIndex: "1"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages")
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.equal(body.status, "ok")
  assert.equal(body.body, '<div class="error-block">No user specified.</div>')
  assert.equal(body.callbackIndex, null)
  assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
  assert.deepEqual(body.cssInclude, [])
  assert.deepEqual(body.jsInclude, [])

  for (const form of [
    { moduleName: "profile/UserInfoModule", user_id: "1", callbackIndex: "1" },
    { moduleName: "profile/UserInfoModule", user_id: "", callbackIndex: "1", extra: "1" }
  ]) {
    const unsupported = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages")
    })
    assert.equal((await unsupported.json()).status, "not_ok")
  }
})

test("accepts the wikidot.py MembersList default and applies Wikidot joined order", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "membership/MembersListModule",
      group: "",
      page: "1"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderMembersList: async (input) => {
        received = input
        return { status: "ok", body: '<div id="ml-12345">members</div>' }
      }
    }
  )

  assert.equal((await response.json()).status, "ok")
  assert.deepEqual(received, {
    siteId: 6000006,
    parameters: { group: "", order: "joined", page: "1" }
  })
})

test("fails closed before Deepwell for unobserved MembersListModule shapes", async () => {
  let calls = 0
  const invalidForms = [
    { group: "" },
    { group: "members", order: "joined", page: "1" },
    { group: "", order: "name", page: "1" },
    { group: "", order: "joined", page: "" },
    { group: "", order: "joined", page: "01" },
    { group: "", order: "joined", page: "-1" },
    { group: "", order: "joined", page: "1.0" },
    { group: "", order: "joined", page: "1", extra: "1" },
    { group: "", order: "joined", page: "1", module_body: "" },
    { group: "", order: "joined", page: "1", eventSource: "member-list" }
  ]

  for (const parameters of invalidForms) {
    const response = await handleAjaxModuleConnectorRequest(
      request({ moduleName: "membership/MembersListModule", ...parameters }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderMembersList: async () => {
          calls += 1
          assert.fail("unobserved MembersListModule shapes must fail before Deepwell")
        }
      }
    )
    assert.equal((await response.json()).status, "not_ok")
  }
  assert.equal(calls, 0)

  const duplicate = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: "moduleName=membership%2FMembersListModule&group=&group=members&order=joined&page=1"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderMembersList: async () =>
        assert.fail("the later duplicate value must fail shape validation")
    }
  )
  assert.equal(duplicate.status, 400)
  assert.deepEqual(await duplicate.json(), {
    status: "not_ok",
    message: "AJAX Module Connector field is duplicated: group"
  })
})

test("dispatches the sealed read-only forum modules with Wikidot metadata", async () => {
  const cases = [
    ["forum/ForumStartModule", {}, []],
    ["forum/ForumStartModule", { hidden: "true" }],
    ["forum/ForumViewCategoryModule", { c: "8503559", p: "1" }],
    [
      "forum/ForumViewThreadModule",
      { t: "18029831" },
      [
        "http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadPostsModule.js",
        "http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadModule.js"
      ]
    ],
    [
      "forum/ForumViewThreadPostsModule",
      { t: "18029831", pageNo: "1" },
      [
        "http://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadPostsModule.js"
      ]
    ],
    ["forum/ForumRecentPostsListModule", { page: "1", categoryId: "8503559" }]
  ]

  for (const [moduleName, parameters, jsInclude = []] of cases) {
    let received
    const response = await handleAjaxModuleConnectorRequest(
      request({
        moduleName,
        ...parameters,
        callbackIndex: "3",
        wikidot_token7: "client-token"
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderForumModule: async (input) => {
          received = input
          return {
            status: "ok",
            body: `<div>${moduleName}</div>`,
            js_include: jsInclude
          }
        }
      }
    )

    const body = await response.json()
    assert.equal(body.status, "ok")
    assert.equal(body.body, `<div>${moduleName}</div>`)
    assert.equal(body.callbackIndex, "3")
    assert.equal(typeof body.CURRENT_TIMESTAMP, "number")
    assert.deepEqual(body.cssInclude, [])
    assert.deepEqual(body.jsInclude, jsInclude)
    assert.deepEqual(received, { siteId: 6000006, moduleName, parameters })
  }
})

test("rejects noncanonical forum numeric scalars before Deepwell", async () => {
  const categoryId = "8503559"
  const pageId = "1927127"
  const threadId = "18029831"
  const cases = [
    ["forum/ForumCommentsListModule", "pageId", pageId, { pageId }],
    ["forum/ForumViewCategoryModule", "c", categoryId, { c: categoryId, p: "1" }],
    ["forum/ForumViewCategoryModule", "p", "1", { c: categoryId, p: "1" }],
    ["forum/ForumViewThreadModule", "t", threadId, { t: threadId }],
    ["forum/ForumViewThreadPostsModule", "t", threadId, { t: threadId, pageNo: "1" }],
    ["forum/ForumViewThreadPostsModule", "pageNo", "1", { t: threadId, pageNo: "1" }],
    ["forum/ForumRecentPostsListModule", "page", "1", { page: "1", categoryId }],
    [
      "forum/ForumRecentPostsListModule",
      "categoryId",
      categoryId,
      { page: "1", categoryId }
    ]
  ]

  let calls = 0
  for (const [moduleName, field, value, parameters] of cases) {
    for (const noncanonical of [`+${value}`, `0${value}`]) {
      const response = await handleAjaxModuleConnectorRequest(
        request({ moduleName, ...parameters, [field]: noncanonical }),
        {
          siteId: 6000006,
          renderListPages: async () => assert.fail("must not render ListPages"),
          renderForumModule: async () => {
            calls += 1
            assert.fail("noncanonical numeric scalars must fail before Deepwell")
          }
        }
      )
      assert.equal((await response.json()).status, "not_ok", `${moduleName} ${field}`)
    }
  }
  assert.equal(calls, 0)
})

test("rejects unobserved ForumStart hidden values before Deepwell", async () => {
  let renders = 0
  for (const hidden of ["false", ""]) {
    const response = await handleAjaxModuleConnectorRequest(
      request({ moduleName: "forum/ForumStartModule", hidden }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderForumModule: async () => {
          renders += 1
          return { status: "ok", body: "must not render" }
        }
      }
    )
    const body = await response.json()
    assert.equal(body.status, "not_ok", `hidden=${hidden}`)
    assert.match(body.message, /Unsupported AJAX module shape/u)
  }
  assert.equal(renders, 0)
})

test("dispatches the sealed page comments reads without adding mutation authority", async () => {
  const cases = [
    {
      parameters: { pageId: "1927127" },
      jsInclude: [
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadModule.js",
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadPostsModule.js",
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/sub/ForumNewPostFormModule.js"
      ]
    },
    {
      parameters: { pageId: "1927127", order: "reverse" },
      jsInclude: [
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadModule.js",
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/sub/ForumNewPostFormModule.js",
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadPostsModule.js"
      ]
    },
    {
      parameters: { pageId: "1927127", order: "forwards" },
      jsInclude: [
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadModule.js",
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/ForumViewThreadPostsModule.js",
        "https://d3g0gp89917ko0.cloudfront.net/v--7690939296dc/common--modules/js/forum/sub/ForumNewPostFormModule.js"
      ]
    }
  ]

  for (const { parameters, jsInclude } of cases) {
    let received
    const response = await handleAjaxModuleConnectorRequest(
      request({
        moduleName: "forum/ForumCommentsListModule",
        ...parameters,
        callbackIndex: "7",
        wikidot_token7: "client-token"
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderForumModule: async (input) => {
          received = input
          return {
            status: "ok",
            body: '<div id="thread-container-posts">comments</div>',
            thread_id: 76632,
            js_include: jsInclude
          }
        }
      }
    )

    const body = await response.json()
    assert.equal(body.status, "ok")
    assert.equal(body.threadId, 76632)
    assert.equal(body.callbackIndex, "7")
    assert.equal(typeof body.CURRENT_TIMESTAMP, "number")
    assert.deepEqual(body.cssInclude, [])
    assert.deepEqual(body.jsInclude, jsInclude)
    assert.deepEqual(received, {
      siteId: 6000006,
      moduleName: "forum/ForumCommentsListModule",
      parameters
    })
  }
})

test("passes read-only forum missing states through and rejects unsealed shapes", async () => {
  let calls = 0
  const missing = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "forum/ForumViewCategoryModule",
      c: "999999999",
      p: "1"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderForumModule: async () => {
        calls += 1
        return { status: "no_category", body: "" }
      }
    }
  )
  const missingBody = await missing.json()
  assert.equal(missingBody.status, "no_category")
  assert.equal(missingBody.body, "")
  assert.equal(missingBody.callbackIndex, null)
  assert.equal(typeof missingBody.CURRENT_TIMESTAMP, "number")
  assert.deepEqual(missingBody.cssInclude, [])
  assert.deepEqual(missingBody.jsInclude, [])

  for (const form of [
    { moduleName: "forum/ForumCommentsListModule", t: "18029831" },
    {
      moduleName: "forum/ForumCommentsListModule",
      pageId: "1927127",
      order: "forward"
    },
    { moduleName: "forum/ForumNewThreadModule", c: "8503559" },
    { moduleName: "forum/ForumViewThreadModule", t: "18029831", write: "1" },
    { moduleName: "forum/ForumViewCategoryModule", c: "8503559" },
    { moduleName: "forum/ForumViewThreadPostsModule", t: "18029831" },
    { moduleName: "forum/ForumRecentPostsListModule", page: "1" }
  ]) {
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderForumModule: async () => {
        calls += 1
        assert.fail("unsealed forum module shape must fail before Deepwell")
      }
    })
    assert.equal((await response.json()).status, "not_ok")
  }
  assert.equal(calls, 1)
})

const forumResponseBody = async (output) => {
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "forum/ForumViewCategoryModule",
      c: "8503559",
      p: "1"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderForumModule: async () => output
    }
  )
  return response.json()
}

test("fails closed when an AMC renderer returns a malformed status", async () => {
  assert.equal((await forumResponseBody({ status: {}, body: "" })).status, "not_ok")
})

test("fails closed when an AMC renderer returns an empty status", async () => {
  assert.equal((await forumResponseBody({ status: "", body: "" })).status, "not_ok")
})

test("fails closed when an AMC renderer omits status", async () => {
  assert.equal((await forumResponseBody({ body: "" })).status, "not_ok")
})

test("fails closed when an AMC renderer returns a non-string status", async () => {
  assert.equal((await forumResponseBody({ status: 503, body: "" })).status, "not_ok")
})

test("passes through a non-empty AMC try_again status", async () => {
  assert.equal(
    (await forumResponseBody({ status: "try_again", body: "" })).status,
    "try_again"
  )
})

test("dispatches the sealed SiteChanges control-browser-shape matrix with Wikidot metadata", async () => {
  const cases = [
    { page: "1", categoryId: "", options: '{"all":true}' },
    { page: "2", categoryId: "", options: '{"all":true}' },
    { page: "3", categoryId: "", options: '{"all":true}' },
    { page: "999999", categoryId: "", options: '{"all":true}' },
    { page: "1", categoryId: "", options: '{"source":true}' },
    { page: "1", categoryId: "", options: '{"files":true}' },
    { page: "1", categoryId: "", options: "{}" },
    { page: "1", categoryId: "999999999", options: '{"all":true}' }
  ]

  for (const { page, categoryId, options } of cases) {
    let received
    const response = await handleAjaxModuleConnectorRequest(
      request({
        moduleName: "changes/SiteChangesListModule",
        page,
        perpage: "20",
        pageId: "74503778",
        categoryId,
        options,
        callbackIndex: "5",
        wikidot_token7: "client-token"
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderSiteChangesModule: async (input) => {
          received = input
          return {
            status: "ok",
            body:
              page === "999999" || categoryId === "999999999"
                ? "Sorry, no revisions matching your criteria."
                : `<div class="pager">page ${page}</div>`
          }
        }
      }
    )

    const body = await response.json()
    assert.equal(body.status, "ok")
    assert.equal(
      body.body,
      page === "999999" || categoryId === "999999999"
        ? "Sorry, no revisions matching your criteria."
        : `<div class="pager">page ${page}</div>`
    )
    assert.equal(body.callbackIndex, "5")
    assert.equal(typeof body.CURRENT_TIMESTAMP, "number")
    assert.deepEqual(body.cssInclude, [])
    assert.deepEqual(body.jsInclude, [])
    assert.deepEqual(received, {
      siteId: 6000006,
      pageId: "74503778",
      page,
      perpage: "20",
      categoryId,
      options
    })
  }
})

test("SiteChanges browser family forwards perpage 10 like the observed perpage 20 shape", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "changes/SiteChangesListModule",
      page: "1",
      perpage: "10",
      pageId: "74503778",
      categoryId: "",
      options: '{"all":true}',
      callbackIndex: "5",
      wikidot_token7: "client-token"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async (input) => {
        received = input
        return { status: "ok", body: '<div class="pager">page 1</div>' }
      }
    }
  )

  assert.equal((await response.json()).status, "ok")
  assert.deepEqual(received, {
    siteId: 6000006,
    pageId: "74503778",
    page: "1",
    perpage: "10",
    categoryId: "",
    options: '{"all":true}'
  })
})

test("SiteChanges browser family forwards the observed perpage values", async () => {
  for (const { perpage, expectedBody } of [
    { perpage: "1", expectedBody: "1 row" },
    { perpage: "50", expectedBody: "50 rows" },
    { perpage: "100", expectedBody: "100 rows" },
    { perpage: "200", expectedBody: "200 rows" },
    { perpage: "0", expectedBody: "Sorry, no revisions matching your criteria." },
    { perpage: "-1", expectedBody: "Sorry, no revisions matching your criteria." }
  ]) {
    let received
    const response = await handleAjaxModuleConnectorRequest(
      request({
        moduleName: "changes/SiteChangesListModule",
        page: "1",
        perpage,
        pageId: "74503778",
        categoryId: "",
        options: '{"all":true}'
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderSiteChangesModule: async (input) => {
          received = input
          return { status: "ok", body: expectedBody }
        }
      }
    )

    assert.equal(response.status, 200)
    const body = await response.json()
    assert.equal(body.status, "ok")
    assert.equal(body.body, expectedBody)
    assert.equal(body.callbackIndex, null)
    assert.equal(typeof body.CURRENT_TIMESTAMP, "number")
    assert.deepEqual(body.cssInclude, [])
    assert.deepEqual(body.jsInclude, [])
    assert.deepEqual(received, {
      siteId: 6000006,
      pageId: "74503778",
      page: "1",
      perpage,
      categoryId: "",
      options: '{"all":true}'
    })
  }
})

test("SiteChanges browser family accepts single-quoted all/source/files via canonical forwarding", async () => {
  for (const { options, expected } of [
    { options: "{'all':true}", expected: '{"all":true}' },
    { options: "{'source':true}", expected: '{"source":true}' },
    { options: "{'files':true}", expected: '{"files":true}' }
  ]) {
    let received
    const response = await handleAjaxModuleConnectorRequest(
      request({
        moduleName: "changes/SiteChangesListModule",
        page: "1",
        perpage: "20",
        pageId: "74503778",
        categoryId: "",
        options,
        callbackIndex: "5",
        wikidot_token7: "client-token"
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderSiteChangesModule: async (input) => {
          received = input
          return { status: "ok", body: '<div class="pager">page 1</div>' }
        }
      }
    )

    assert.equal((await response.json()).status, "ok")
    assert.deepEqual(received, {
      siteId: 6000006,
      pageId: "74503778",
      page: "1",
      perpage: "20",
      categoryId: "",
      options: expected
    })
  }

  for (const options of ['{"title":true}', "{'title':true}", '{"unlisted":true}']) {
    const rejected = await handleAjaxModuleConnectorRequest(
      request({
        moduleName: "changes/SiteChangesListModule",
        page: "1",
        perpage: "20",
        pageId: "74503778",
        categoryId: "",
        options
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderSiteChangesModule: async () =>
          assert.fail(`unobserved browser options must fail before Deepwell: ${options}`)
      }
    )
    assert.equal((await rejected.json()).status, "not_ok")
  }
})

test("SiteChanges accepts wikidot.py client-page-one-default without browser host fields", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "changes/SiteChangesListModule",
      perpage: "1000",
      page: "1",
      options: "{'all':true}"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async (input) => {
        received = input
        return { status: "ok", body: '<div class="pager">page 1</div>' }
      }
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.equal(body.status, "ok")
  assert.equal(body.body, '<div class="pager">page 1</div>')
  assert.equal(body.callbackIndex, null)
  assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
  assert.deepEqual(body.cssInclude, [])
  assert.deepEqual(body.jsInclude, [])
  assert.deepEqual(received, {
    siteId: 6000006,
    page: "1",
    perpage: "1000",
    options: '{"all":true}'
  })
})

test("SiteChanges keeps client-later-page on the wikidot.py 1000-row family", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "changes/SiteChangesListModule",
      perpage: "1000",
      page: "2",
      options: "{'all':true}"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async (input) => {
        received = input
        return { status: "ok", body: '<div class="pager">page 2</div>' }
      }
    }
  )

  assert.equal((await response.json()).status, "ok")
  assert.deepEqual(received, {
    siteId: 6000006,
    page: "2",
    perpage: "1000",
    options: '{"all":true}'
  })
})

test("SiteChanges preserves control-empty-options control-source-options and control-missing-options", async () => {
  const cases = [
    { options: undefined, expected: '{"all":true}' },
    { options: "{}", expected: '{"all":true}' },
    { options: "{'source':true}", expected: '{"source":true}' },
    { options: '{"all":true}', expected: '{"all":true}' },
    { options: '{"source":true}', expected: '{"source":true}' },
    { options: '{"files":true}', expected: '{"files":true}' },
    { options: "{'files':true}", expected: '{"files":true}' }
  ]

  for (const { options, expected } of cases) {
    let received
    const form = {
      moduleName: "changes/SiteChangesListModule",
      perpage: "20",
      page: "1"
    }
    if (options !== undefined) form.options = options
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async (input) => {
        received = input
        return { status: "ok", body: "rows" }
      }
    })

    assert.equal((await response.json()).status, "ok")
    assert.deepEqual(received, {
      siteId: 6000006,
      page: "1",
      perpage: "20",
      options: expected
    })
  }
})

test("SiteChanges gives control-bad-page bounded nonnumeric first-page semantics", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "changes/SiteChangesListModule",
      perpage: "20",
      page: "not-a-page",
      options: "{'all':true}"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async (input) => {
        received = input
        return { status: "ok", body: "20 rows" }
      }
    }
  )

  assert.equal((await response.json()).status, "ok")
  assert.deepEqual(received, {
    siteId: 6000006,
    page: "1",
    perpage: "20",
    options: '{"all":true}'
  })
})

test("SiteChanges returns the control-bad-perpage empty result without widening valid sizes", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "changes/SiteChangesListModule",
      perpage: "not-a-number",
      page: "1",
      options: "{'all':true}"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async (input) => {
        received = input
        return {
          status: "ok",
          body: "\tSorry, no revisions matching your criteria."
        }
      }
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.equal(body.status, "ok")
  assert.equal(body.body, "\tSorry, no revisions matching your criteria.")
  assert.equal(body.callbackIndex, null)
  assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
  assert.deepEqual(body.cssInclude, [])
  assert.deepEqual(body.jsInclude, [])
  assert.deepEqual(received, {
    siteId: 6000006,
    page: "1",
    perpage: "not-a-number",
    options: '{"all":true}'
  })
})

test("SiteChanges ignores one bounded control-unknown-field only in the wikidot.py family", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "changes/SiteChangesListModule",
      perpage: "20",
      page: "1",
      options: "{'all':true}",
      unknownField: "control"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async (input) => {
        received = input
        return { status: "ok", body: "rows" }
      }
    }
  )

  assert.equal((await response.json()).status, "ok")
  assert.deepEqual(received, {
    siteId: 6000006,
    page: "1",
    perpage: "20",
    options: '{"all":true}'
  })
})

test("SiteChanges wikidot.py family fails closed for mixed and unobserved scalar shapes", async () => {
  const overrides = [
    { page: "0" },
    { page: "-1" },
    { page: "1.5" },
    { page: "1tail" },
    { page: "9007199254740993" },
    { perpage: "10" },
    { pageId: "74503778" },
    { categoryId: "" },
    { module_body: "" },
    { action: "read" },
    { event: "read" },
    { unknownOne: "1", unknownTwo: "2" }
  ]
  let calls = 0
  const valid = {
    moduleName: "changes/SiteChangesListModule",
    page: "1",
    perpage: "1000",
    options: "{'all':true}"
  }

  for (const override of overrides) {
    const response = await handleAjaxModuleConnectorRequest(
      request({ ...valid, ...override }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderSiteChangesModule: async () => {
          calls += 1
          assert.fail("unsupported wikidot.py shape must fail before Deepwell")
        }
      }
    )
    assert.equal((await response.json()).status, "not_ok")
  }
  assert.equal(calls, 0)
})

test("fails closed for unobserved SiteChanges shapes before Deepwell", async () => {
  const forms = [
    {},
    { page: "0" },
    { page: "-1" },
    { page: "1.0" },
    { page: "9007199254740993" },
    { pageId: "" },
    { pageId: "-1" },
    { pageId: "9007199254740993" },
    { perpage: "1.5" },
    { perpage: "5001" },
    { perpage: "9007199254740993" },
    { perpage: "-9007199254740993" },
    { categoryId: "missing" },
    { options: '{"all":false}' },
    { options: '{"source":true,"files":true}' },
    { options: '{ "all": true }' },
    { unknown: "value" },
    { module_body: "" }
  ]
  const valid = {
    moduleName: "changes/SiteChangesListModule",
    page: "1",
    perpage: "20",
    pageId: "74503778",
    categoryId: "",
    options: '{"all":true}'
  }

  for (const override of forms) {
    const form = { ...valid, ...override }
    if (Object.keys(override).length === 0) delete form.options
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async () =>
        assert.fail("unobserved SiteChanges shape must fail before Deepwell")
    })
    assert.equal((await response.json()).status, "not_ok")
  }

  const duplicate = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: `${new URLSearchParams(valid)}&page=2`
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderSiteChangesModule: async () =>
        assert.fail("duplicate SiteChanges fields must fail before Deepwell")
    }
  )
  assert.equal(duplicate.status, 400)
  assert.deepEqual(await duplicate.json(), {
    status: "not_ok",
    message: "AJAX Module Connector field is duplicated: page"
  })
})

test("ListPages keeps later module names while other modules reject duplicates", async () => {
  const unsupported = await handleAjaxModuleConnectorRequest(
    request({ moduleName: "forum/ForumStartModule", module_body: "" }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )
  assert.deepEqual(await unsupported.json(), {
    status: "not_ok",
    message: "Unsupported AJAX module: forum/ForumStartModule"
  })

  let listPagesReceived
  const unknownThenListPages = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: {
        "content-type": "application/x-www-form-urlencoded",
        cookie: "wikidot_token7=client-token"
      },
      body: "moduleName=not-a-real-module&moduleName=list%2FListPagesModule&module_body=x&wikidot_token7=client-token"
    }),
    {
      siteId: 6000006,
      renderListPages: async (input) => {
        listPagesReceived = input
        return { body: "list-pages" }
      }
    }
  )
  assert.equal(unknownThenListPages.status, 200)
  assert.deepEqual(await unknownThenListPages.json(), {
    status: "ok",
    body: "list-pages"
  })
  assert.equal(listPagesReceived.moduleBody, "x")

  const listPagesThenUnknown = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: "moduleName=list%2FListPagesModule&moduleName=not-a-real-module&module_body=x"
    }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )
  assert.equal(listPagesThenUnknown.status, 400)
  assert.deepEqual(await listPagesThenUnknown.json(), {
    status: "not_ok",
    message: "AJAX Module Connector field is duplicated: moduleName"
  })
})

test("dispatches wikidot.py page reads without rewriting their request fields", async () => {
  const calls = []
  for (const moduleName of ["viewsource/ViewSourceModule", "files/PageFilesModule"]) {
    const response = await handleAjaxModuleConnectorRequest(
      request({ moduleName, page_id: "1469071756", callbackIndex: "client-0" }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderPageReadModule: async (input) => {
          calls.push(input)
          return { status: "ok", body: `<div>${moduleName}</div>` }
        }
      }
    )

    assert.equal(response.status, 200)
    const body = await response.json()
    assert.equal(body.status, "ok")
    assert.equal(body.body, `<div>${moduleName}</div>`)
    assert.equal(body.callbackIndex, "client-0")
    assert.deepEqual(body.cssInclude, [])
    assert.deepEqual(body.jsInclude, [])
    assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
  }

  assert.deepEqual(calls, [
    {
      siteId: 6000006,
      moduleName: "viewsource/ViewSourceModule",
      parameters: { page_id: "1469071756" }
    },
    {
      siteId: 6000006,
      moduleName: "files/PageFilesModule",
      parameters: { page_id: "1469071756" }
    }
  ])
})

test("returns the observed no_page envelope for a missing ViewSource page", async () => {
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "viewsource/ViewSourceModule",
      page_id: "0",
      callbackIndex: "client-missing-page"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderPageReadModule: async () => assert.fail("must not render a missing page")
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.equal(body.status, "no_page")
  assert.equal(body.callbackIndex, "client-missing-page")
  assert.equal(typeof body.message, "string")
  assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
  assert.deepEqual(Object.keys(body).sort(), [
    "CURRENT_TIMESTAMP",
    "callbackIndex",
    "message",
    "status"
  ])
})

test("dispatches only the canonical wikidot.py WhoRated shape", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "pagerate/WhoRatedPageModule",
      pageId: "1468540301",
      callbackIndex: "client-who-rated"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderPageReadModule: async (input) => {
        received = input
        return {
          status: "ok",
          body: '<h2>Users who rated:</h2>\n\n<div style="-moz-column-count:3"></div>'
        }
      }
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.equal(body.status, "ok")
  assert.equal(body.body.length, 66)
  assert.equal(body.callbackIndex, "client-who-rated")
  assert.deepEqual(received, {
    siteId: 6000006,
    moduleName: "pagerate/WhoRatedPageModule",
    parameters: { pageId: "1468540301" }
  })
})

test("WhoRated pageId zero follows the observed HTTP failure boundary", async () => {
  const response = await handleAjaxModuleConnectorRequest(
    request({ moduleName: "pagerate/WhoRatedPageModule", pageId: "0" }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderPageReadModule: async () => assert.fail("must not expose votes")
    }
  )

  assert.equal(response.status, 500)
  assert.equal((await response.json()).status, "not_ok")
})

test("WhoRated rejects unknown and malformed request fields", async () => {
  const malformedShapes = [
    request({ moduleName: "pagerate/WhoRatedPageModule", pageId: "+1" }),
    request({ moduleName: "pagerate/WhoRatedPageModule", pageId: "1", extra: "1" })
  ]
  let renders = 0
  for (const malformed of malformedShapes) {
    const response = await handleAjaxModuleConnectorRequest(malformed, {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderPageReadModule: async () => {
        renders += 1
        return { status: "ok", body: "must not render" }
      }
    })
    assert.equal((await response.json()).status, "not_ok")
  }
  assert.equal(renders, 0)

  const duplicate = await handleAjaxModuleConnectorRequest(
    new Request("http://scp-wiki.local/ajax-module-connector.php", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded" },
      body: "moduleName=pagerate%2FWhoRatedPageModule&pageId=1&pageId=2"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderPageReadModule: async () =>
        assert.fail("duplicate WhoRated fields must fail before Deepwell")
    }
  )
  assert.equal(duplicate.status, 400)
  assert.deepEqual(await duplicate.json(), {
    status: "not_ok",
    message: "AJAX Module Connector field is duplicated: pageId"
  })
})

test("dispatches the exact wikidot.py page revision list shape", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "history/PageRevisionListModule",
      page_id: "1469071756",
      options: "{'all': True}",
      perpage: "100000000",
      callbackIndex: "client-history"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderPageReadModule: async (input) => {
        received = input
        return { status: "ok", body: '<table class="page-history"></table>' }
      }
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
  delete body.CURRENT_TIMESTAMP
  assert.deepEqual(body, {
    status: "ok",
    body: '<table class="page-history"></table>',
    callbackIndex: "client-history",
    cssInclude: [],
    jsInclude: []
  })
  assert.deepEqual(received, {
    siteId: 6000006,
    moduleName: "history/PageRevisionListModule",
    parameters: {
      page_id: "1469071756",
      options: "{'all': True}",
      perpage: "100000000"
    }
  })
})

test("dispatches the exact wikidot.py historical source and version shapes", async () => {
  const calls = []
  for (const moduleName of ["history/PageSourceModule", "history/PageVersionModule"]) {
    const response = await handleAjaxModuleConnectorRequest(
      request({ moduleName, revision_id: "1000003" }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render ListPages"),
        renderPageReadModule: async (input) => {
          calls.push(input)
          return { status: "ok", body: `<div>${moduleName}</div>` }
        }
      }
    )

    assert.equal(response.status, 200)
    assert.equal((await response.json()).status, "ok")
  }

  assert.deepEqual(calls, [
    {
      siteId: 6000006,
      moduleName: "history/PageSourceModule",
      parameters: { revision_id: "1000003" }
    },
    {
      siteId: 6000006,
      moduleName: "history/PageVersionModule",
      parameters: { revision_id: "1000003" }
    }
  ])
})

test("fails closed for unevidenced wikidot.py page read shapes", async () => {
  let calls = 0
  for (const form of [
    { moduleName: "viewsource/ViewSourceModule" },
    { moduleName: "files/PageFilesModule", page_id: "0" },
    {
      moduleName: "viewsource/ViewSourceModule",
      page_id: "1469071756",
      pageId: "1469071756"
    },
    {
      moduleName: "history/PageRevisionListModule",
      page_id: "1469071756",
      options: "{'all': False}",
      perpage: "100000000"
    },
    {
      moduleName: "history/PageRevisionListModule",
      page_id: "1469071756",
      options: "{'all': True}",
      perpage: "100"
    },
    {
      moduleName: "history/PageSourceModule",
      revision_id: "1000003",
      page_id: "1469071756"
    },
    {
      moduleName: "history/PageVersionModule",
      revision_id: "0"
    }
  ]) {
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderPageReadModule: async () => {
        calls += 1
        assert.fail("unsupported shape must fail before the renderer")
      }
    })

    assert.deepEqual(await response.json(), {
      status: "not_ok",
      message: `Unsupported AJAX module shape: ${form.moduleName}`
    })
  }
  assert.equal(calls, 0)
})

test("rejects oversized bodies while streaming missing-length requests", async () => {
  let arrayBufferCalls = 0
  const response = await handleAjaxModuleConnectorRequest(
    {
      method: "POST",
      headers: new Headers({
        "content-type": "application/x-www-form-urlencoded"
      }),
      body: new Blob([
        "moduleName=list%2FListPagesModule&module_body=",
        "x".repeat(131_073)
      ]).stream(),
      arrayBuffer: async () => {
        arrayBufferCalls += 1
        throw new Error("must not buffer full request")
      }
    },
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )

  assert.equal(response.status, 413)
  assert.equal((await response.json()).status, "not_ok")
  assert.equal(arrayBufferCalls, 0)
})

test("converts Deepwell failures to a stable Wikidot error envelope", async () => {
  const originalConsoleError = console.error
  console.error = () => {}
  try {
    const response = await handleAjaxModuleConnectorRequest(
      request({
        moduleName: "list/ListPagesModule",
        module_body: "%%fullname%%",
        name: "=",
        wikidot_token7: "client-token"
      }),
      {
        siteId: 6000006,
        renderListPages: async () => {
          throw new Error("current-page selectors are unsupported")
        }
      }
    )
    assert.deepEqual(await response.json(), {
      status: "not_ok",
      message: "Unable to render ListPages module"
    })
  } finally {
    console.error = originalConsoleError
  }
})

test("dispatches Wikidot page discussion creation and preserves its wire envelope", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "ForumAction",
      event: "createPageDiscussionThread",
      moduleName: "Empty",
      page_id: "1469071756",
      callbackIndex: "lane4-callback-success"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render"),
      createPageDiscussion: async (input) => {
        received = input
        return {
          thread_id: 18232631,
          thread_unix_title: "lane-4-discussion"
        }
      }
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.deepEqual(received, { siteId: 6000006, pageId: 1469071756 })
  assert.equal(body.status, "ok")
  assert.equal(body.thread_id, 18232631)
  assert.equal(body.thread_unix_title, "lane-4-discussion")
  assert.equal(body.callbackIndex, "lane4-callback-success")
  assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
})

test("page discussion creation uses Wikidot no_page and stable failure boundaries", async () => {
  for (const pageId of ["", "-1", "1.5", "9007199254740993"]) {
    const response = await handleAjaxModuleConnectorRequest(
      request({
        action: "ForumAction",
        event: "createPageDiscussionThread",
        moduleName: "Empty",
        page_id: pageId,
        callbackIndex: `lane4-callback-${pageId}`
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render"),
        createPageDiscussion: async () => assert.fail("must not create")
      }
    )
    const body = await response.json()
    assert.equal(body.status, "no_page")
    assert.equal(body.message, "The page does not exist")
    assert.equal(body.callbackIndex, `lane4-callback-${pageId}`)
    assert.equal(Number.isInteger(body.CURRENT_TIMESTAMP), true)
  }

  const missing = await handleAjaxModuleConnectorRequest(
    request({
      action: "ForumAction",
      event: "createPageDiscussionThread",
      moduleName: "Empty",
      page_id: "1469071758",
      callbackIndex: "lane4-callback-missing"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render"),
      createPageDiscussion: async () => null
    }
  )
  const missingBody = await missing.json()
  assert.equal(missingBody.status, "no_page")
  assert.equal(missingBody.message, "The page does not exist")
  assert.equal(missingBody.callbackIndex, "lane4-callback-missing")
  assert.equal(Number.isInteger(missingBody.CURRENT_TIMESTAMP), true)

  const originalConsoleError = console.error
  console.error = () => {}
  try {
    const failed = await handleAjaxModuleConnectorRequest(
      request({
        action: "ForumAction",
        event: "createPageDiscussionThread",
        moduleName: "Empty",
        page_id: "1469071760",
        callbackIndex: "lane4-callback-failed"
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render"),
        createPageDiscussion: async () => {
          throw new Error("backend unavailable")
        }
      }
    )
    const failedBody = await failed.json()
    assert.equal(failedBody.status, "not_ok")
    assert.equal(failedBody.message, "Unable to create page discussion")
    assert.equal(failedBody.callbackIndex, "lane4-callback-failed")
    assert.equal(Number.isInteger(failedBody.CURRENT_TIMESTAMP), true)
  } finally {
    console.error = originalConsoleError
  }
})

test("anonymous ForumAction savePost hashes the private guest email before Deepwell", async () => {
  const calls = []
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "ForumAction",
      event: "savePost",
      moduleName: "Empty",
      threadId: "18029831",
      parentId: "",
      guestName: "Guest Name",
      guestEmail: "  SUPPORT@GRAVATAR.COM  ",
      source: "Guest body",
      wikidot_token7: "client-token"
    }),
    {
      siteId: 6000006,
      createForumPost: async (input) => {
        calls.push(input)
        return { forum_post_id: 9036580 }
      }
    }
  )

  assert.deepEqual(await response.json(), {
    status: "ok",
    postId: 9036580
  })
  assert.deepEqual(calls, [
    {
      siteId: 6000006,
      threadId: 18029831,
      parentPostId: null,
      title: "",
      source: "Guest body",
      guestName: "Guest Name",
      guestEmailMd5: "367c4ed53ac64deb1b7753b1556236c2"
    }
  ])
  assert.equal(JSON.stringify(calls).includes("SUPPORT@GRAVATAR.COM"), false)
})

test("ForumAction savePost rejects incomplete or malformed guest identity before Deepwell", async () => {
  const canonical = {
    action: "ForumAction",
    event: "savePost",
    moduleName: "Empty",
    threadId: "18029831",
    parentId: "",
    guestName: "Guest Name",
    guestEmail: "support@gravatar.com",
    source: "Guest body"
  }
  const invalid = [
    { ...canonical, guestName: "" },
    { ...canonical, guestEmail: "" },
    { ...canonical, guestEmail: "not-an-email" },
    { ...canonical, guestEmail: "a@b" },
    { ...canonical, guestEmail: `${"a".repeat(39)}@example.com` },
    { ...canonical, threadId: "0" },
    { ...canonical, parentId: "-1" },
    { ...canonical, extra: "unsupported" }
  ]

  let calls = 0
  for (const form of invalid) {
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      createForumPost: async () => {
        calls += 1
        assert.fail("invalid savePost controls must fail before Deepwell")
      }
    })
    assert.notEqual((await response.json()).status, "ok")
  }
  assert.equal(calls, 0)
})

test("dispatches the observed WikiPageAction deletePage request", async () => {
  const calls = []
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "WikiPageAction",
      event: "deletePage",
      page_id: "1469167148",
      moduleName: "Empty",
      wikidot_token7: "client-token"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      deletePage: async (input) => calls.push(input)
    }
  )

  assert.deepEqual(await response.json(), { status: "ok" })
  assert.deepEqual(calls, [{ siteId: 6000006, pageId: 1469167148 }])
})

test("dispatches the observed full-page draft synchronize and existence lifecycle", async () => {
  const saves = []
  const existsChecks = []
  const synchronize = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "Empty",
      action: "WikiPageAction",
      event: "synchronize",
      mode: "page",
      wiki_page: "run-owned:draft-existing",
      lock_id: "91",
      lock_secret: "observed-lock-secret",
      revision_id: "123",
      page_id: "42",
      source: "draft source",
      title: "Draft title",
      comments: "draft evidence",
      since_last_input: "7",
      wikidot_token7: "client-token"
    }),
    {
      siteId: 6000006,
      savePageDraft: async (input) => saves.push(input)
    }
  )
  assert.deepEqual(await synchronize.json(), { status: "ok", savedDraft: true })
  assert.deepEqual(saves, [
    {
      siteId: 6000006,
      pageId: 42,
      slug: "run-owned:draft-existing",
      title: "Draft title",
      wikitext: "draft source"
    }
  ])

  const exists = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "Empty",
      action: "WikiPageAction",
      event: "checkDraftExists",
      wiki_page: "run-owned:draft-existing",
      lock_id: "91",
      page_id: "42",
      title: "Draft title",
      source: "draft source",
      wikidot_token7: "client-token"
    }),
    {
      siteId: 6000006,
      pageDraftExists: async (input) => {
        existsChecks.push(input)
        return true
      }
    }
  )
  assert.deepEqual(await exists.json(), { status: "ok", draftExists: true })
  assert.deepEqual(existsChecks, [
    { siteId: 6000006, pageId: 42, slug: "run-owned:draft-existing" }
  ])
})

test("removePageEditLock preserves or discards only the observed draft identity", async () => {
  const removals = []
  const base = {
    moduleName: "Empty",
    action: "WikiPageAction",
    event: "removePageEditLock",
    wiki_page: "run-owned:draft-new",
    lock_id: "92",
    lock_secret: "observed-lock-secret",
    wikidot_token7: "client-token"
  }
  for (const leaveDraft of ["true", "false"]) {
    const response = await handleAjaxModuleConnectorRequest(
      request({ ...base, leave_draft: leaveDraft }),
      {
        siteId: 6000006,
        removePageDraft: async (input) => removals.push(input)
      }
    )
    assert.deepEqual(await response.json(), { status: "ok" })
  }
  assert.deepEqual(removals, [{ siteId: 6000006, slug: "run-owned:draft-new" }])
})

test("page draft actions fail closed outside the observed full-page request shape", async () => {
  let calls = 0
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "Empty",
      action: "WikiPageAction",
      event: "synchronize",
      mode: "section",
      wiki_page: "run-owned:draft-section",
      lock_id: "93",
      lock_secret: "observed-lock-secret",
      revision_id: "",
      source: "draft source",
      title: "Draft title",
      comments: "",
      since_last_input: "0",
      range_start: "1",
      range_end: "2",
      wikidot_token7: "client-token"
    }),
    {
      siteId: 6000006,
      savePageDraft: async () => {
        calls += 1
      }
    }
  )
  assert.deepEqual(await response.json(), { status: "not_ok" })
  assert.equal(calls, 0)
})

test("deletePage fails closed for invalid controls and Deepwell failures", async () => {
  const canonical = {
    action: "WikiPageAction",
    event: "deletePage",
    page_id: "1469167148",
    moduleName: "Empty",
    wikidot_token7: "client-token"
  }
  const invalidForms = [
    { ...canonical, extra: "unexpected" },
    ...["action", "event", "page_id", "moduleName"].map((field) => {
      const form = { ...canonical }
      delete form[field]
      return form
    }),
    ...["", "0", "-1", "1.5", "9007199254740993"].map((page_id) => ({
      ...canonical,
      page_id
    }))
  ]

  let calls = 0
  for (const form of invalidForms) {
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      deletePage: async () => {
        calls += 1
        assert.fail("invalid deletePage controls must fail before Deepwell")
      }
    })
    assert.equal((await response.json()).status, "not_ok")
  }
  assert.equal(calls, 0)

  const missingDependency = await handleAjaxModuleConnectorRequest(request(canonical), {
    siteId: 6000006,
    renderListPages: async () => assert.fail("must not render ListPages")
  })
  assert.equal((await missingDependency.json()).status, "not_ok")

  const originalConsoleError = console.error
  console.error = () => {}
  try {
    const failed = await handleAjaxModuleConnectorRequest(request(canonical), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      deletePage: async () => {
        throw new Error("Deepwell unavailable")
      }
    })
    assert.equal((await failed.json()).status, "not_ok")
  } finally {
    console.error = originalConsoleError
  }
})

test("dispatches pagepath Create new as the observed immediate DataFormAction mutation", async () => {
  const calls = []
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "DataFormAction",
      event: "newPage",
      category: "tree",
      parent: "tree:alpha",
      title: "gamma",
      moduleName: "Empty",
      callbackIndex: "2",
      wikidot_token7: "client-token"
    }),
    {
      siteId: 6000006,
      canCreateNewPage: true,
      pageExists: async () => false,
      createNewPage: async (input) => calls.push(input)
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.equal(body.status, "ok")
  assert.equal(body.fullname, "tree:gamma")
  assert.equal(body.callbackIndex, "2")
  assert.equal(Number.isSafeInteger(body.CURRENT_TIMESTAMP), true)
  assert.deepEqual(calls, [
    {
      slug: "tree:gamma",
      title: "gamma",
      wikitext: "",
      tags: [],
      parentPage: "tree:alpha"
    }
  ])
})

test("pagepath first root child bootstraps the empty _root before creating the child", async () => {
  const calls = []
  const existing = new Set()
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "DataFormAction",
      event: "newPage",
      category: "tree-name",
      parent: "",
      title: "alpha",
      moduleName: "Empty",
      callbackIndex: "2"
    }),
    {
      siteId: 6000006,
      canCreateNewPage: true,
      pageExists: async (slug) => existing.has(slug),
      createNewPage: async (input) => {
        calls.push(input)
        existing.add(input.slug)
      }
    }
  )

  const body = await response.json()
  assert.equal(body.status, "ok")
  assert.equal(body.fullname, "tree-name:alpha")
  assert.deepEqual(calls, [
    {
      slug: "tree-name:_root",
      title: "Tree-name",
      wikitext: "",
      tags: [],
      parentPage: null
    },
    {
      slug: "tree-name:alpha",
      title: "alpha",
      wikitext: "",
      tags: [],
      parentPage: "tree-name:_root"
    }
  ])
})

test("pagepath Create new fails before mutation for unsupported shape, collision, or denied actor", async () => {
  const base = {
    action: "DataFormAction",
    event: "newPage",
    category: "tree",
    parent: "tree:alpha",
    title: "gamma",
    moduleName: "Empty",
    callbackIndex: "2"
  }
  for (const [form, options] of [
    [
      { ...base, extra: "unsupported" },
      { canCreateNewPage: true, pageExists: async () => false }
    ],
    [base, { canCreateNewPage: true, pageExists: async () => true }],
    [base, { canCreateNewPage: false, pageExists: async () => false }]
  ]) {
    let mutated = false
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      ...options,
      createNewPage: async () => {
        mutated = true
      }
    })
    assert.equal((await response.json()).status, "not_ok")
    assert.equal(mutated, false)
  }
})

test("dispatches NewPage helper default action with Wikidot edit-routing fields", async () => {
  const pageName = `run-owned:${"x".repeat(51)}`
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName,
      tags: "alpha beta",
      parent: "main"
    }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )

  assert.deepEqual(await response.json(), {
    status: "ok",
    unixName: pageName.slice(0, 60),
    pageTitle: pageName,
    tags: "alpha beta",
    parentPage: "main"
  })
})

test("dispatches NewPage template and category action fields like Wikidot", async () => {
  const templated = await handleAjaxModuleConnectorRequest(
    request({
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: "run-owned:newpage-template-edit",
      mode: "edit",
      template: "1469068384",
      tags: "alpha beta",
      parent: "run-owned:newpage-parent"
    }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )

  assert.deepEqual(await templated.json(), {
    status: "ok",
    unixName: "run-owned:newpage-template-edit",
    pageTitle: "run-owned:newpage-template-edit",
    tags: "alpha beta",
    parentPage: "run-owned:newpage-parent",
    templateId: "1469068384"
  })

  const defaultCategory = await handleAjaxModuleConnectorRequest(
    request({
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: "newpage-default-category",
      categoryName: "_default"
    }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )

  assert.deepEqual(await defaultCategory.json(), {
    status: "ok",
    unixName: "newpage-default-category",
    pageTitle: "newpage-default-category",
    tags: "",
    parentPage: ""
  })
})

test("rejects NewPage helper requests when the target already exists", async () => {
  const calls = []
  const pageName = "run-owned:newpage-existing-target"
  for (const mode of [undefined, "edit", "save-and-go", "save-and-refresh"]) {
    const form = {
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName
    }
    if (mode !== undefined) form.mode = mode

    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render"),
      createNewPage: async (input) => calls.push(input),
      pageExists: async (slug) => slug === pageName
    })

    assert.deepEqual(await response.json(), {
      status: "page_exists",
      message:
        'The page <em>run-owned:newpage-existing-target</em> already exists. <a href="/run-owned:newpage-existing-target">Jump to it</a> if you wish.'
    })
  }
  assert.deepEqual(calls, [])
})

test("rejects NewPage helper requests without a page name like Wikidot", async () => {
  for (const form of [
    {
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty"
    },
    {
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: ""
    }
  ]) {
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render")
    })

    assert.deepEqual(await response.json(), {
      status: "no_name",
      message: "You should provide a page name"
    })
  }
})

test("allows NewPage edit routing without page creation permission", async () => {
  for (const mode of [undefined, "edit"]) {
    const form = {
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: "run-owned:newpage-anonymous-edit"
    }
    if (mode !== undefined) form.mode = mode

    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render"),
      canCreateNewPage: false
    })

    const body = await response.json()
    assert.equal(body.status, "ok")
    assert.equal(body.unixName, "run-owned:newpage-anonymous-edit")
    assert.equal(body.pageTitle, "run-owned:newpage-anonymous-edit")
  }
})

test("rejects NewPage autosave without page creation permission", async () => {
  const calls = []
  for (const mode of ["save-and-go", "save-and-refresh"]) {
    const response = await handleAjaxModuleConnectorRequest(
      request({
        action: "misc/NewPageHelperAction",
        event: "createNewPage",
        moduleName: "Empty",
        pageName: "run-owned:newpage-anonymous-autosave",
        mode
      }),
      {
        siteId: 6000006,
        renderListPages: async () => assert.fail("must not render"),
        canCreateNewPage: false,
        createNewPage: async (input) => calls.push(input)
      }
    )

    assert.deepEqual(await response.json(), {
      status: "no_permission",
      message:
        'Sorry, you can not create a new page in this category. Only members of this site, site administrators and perhaps selected moderators are allowed to do it. <a href="#action:login">Sign in as Wikidot user</a>'
    })
  }
  assert.deepEqual(calls, [])
})

test("ignores malformed NewPage format strings but enforces valid patterns", async () => {
  const malformedFormats = ["//", "/^[a-z]+$", "/[/", "^[a-z]+$"]
  for (const format of malformedFormats) {
    const response = await handleAjaxModuleConnectorRequest(
      request({
        action: "misc/NewPageHelperAction",
        event: "createNewPage",
        moduleName: "Empty",
        pageName: "run-owned:newpage-format-probe",
        format
      }),
      { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
    )

    assert.equal((await response.json()).status, "ok")
  }

  const rejected = await handleAjaxModuleConnectorRequest(
    request({
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: "run-owned:newpage-format-probe",
      format: "/^[0-9]+$/"
    }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )

  assert.deepEqual(await rejected.json(), {
    status: "incorrect_name",
    message: "The page name is not correct: please fix it and try again"
  })
})

test("rejects catastrophic NewPage format patterns without running them", async () => {
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: `${"a".repeat(30)}!`,
      format: "/^(a+)+$/"
    }),
    { siteId: 6000006, renderListPages: async () => assert.fail("must not render") }
  )

  assert.deepEqual(await response.json(), {
    status: "incorrect_name",
    message: "The page name is not correct: please fix it and try again"
  })
})

test("NewPage template autosave creates an empty page and ignores parent", async () => {
  const calls = []
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: "run-owned:newpage-template-autosave",
      mode: "save-and-go",
      template: "1469068213",
      parent: "main"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render"),
      createNewPage: async (input) => calls.push(input)
    }
  )

  assert.deepEqual(await response.json(), {
    status: "ok",
    goToUrl: "run-owned:newpage-template-autosave"
  })
  assert.deepEqual(calls, [
    {
      slug: "run-owned:newpage-template-autosave",
      title: "run-owned:newpage-template-autosave",
      wikitext: "",
      tags: [],
      parentPage: ""
    }
  ])
})

test("rejects NewPage template autosave when tags are submitted", async () => {
  const calls = []
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: "run-owned:newpage-template-tags-autosave",
      mode: "save-and-refresh",
      template: "1469068213",
      tags: "alpha beta"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render"),
      createNewPage: async (input) => calls.push(input)
    }
  )

  assert.deepEqual(await response.json(), {
    status: "not_ok",
    message: "An error occurred while processing the request."
  })
  assert.deepEqual(calls, [])
})

test("dispatches NewPage autosave modes through the injected create callback", async () => {
  const calls = []
  const response = await handleAjaxModuleConnectorRequest(
    request({
      action: "misc/NewPageHelperAction",
      event: "createNewPage",
      moduleName: "Empty",
      pageName: "run-owned:newpage-autosave",
      mode: "save-and-go",
      tags: "alpha beta",
      parent: "main"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render"),
      createNewPage: async (input) => calls.push(input)
    }
  )

  assert.deepEqual(await response.json(), {
    status: "ok",
    goToUrl: "run-owned:newpage-autosave"
  })
  assert.deepEqual(calls, [
    {
      slug: "run-owned:newpage-autosave",
      title: "run-owned:newpage-autosave",
      wikitext: "",
      tags: ["alpha", "beta"],
      parentPage: "main"
    }
  ])
})

test("dispatches WikiCategoriesPageListModule through the actor-scoped read callback", async () => {
  let received
  const response = await handleAjaxModuleConnectorRequest(
    request({
      moduleName: "list/WikiCategoriesPageListModule",
      category_id: "17",
      callbackIndex: "4"
    }),
    {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderCategoriesPageList: async (input) => {
        received = input
        return {
          status: "ok",
          body: '<ul><li><a href="/alpha">Alpha</a></li></ul>'
        }
      }
    }
  )

  assert.equal(response.status, 200)
  const body = await response.json()
  assert.equal(body.status, "ok")
  assert.equal(body.body, '<ul><li><a href="/alpha">Alpha</a></li></ul>')
  assert.equal(body.categoryId, 17)
  assert.equal(body.callbackIndex, "4")
  assert.deepEqual(received, { siteId: 6000006, categoryId: 17 })
})

test("WikiCategoriesPageListModule rejects forged or malformed category selectors", async () => {
  for (const form of [
    { moduleName: "list/WikiCategoriesPageListModule", category_id: "0" },
    { moduleName: "list/WikiCategoriesPageListModule", category_id: "17", extra: "x" }
  ]) {
    let called = false
    const response = await handleAjaxModuleConnectorRequest(request(form), {
      siteId: 6000006,
      renderListPages: async () => assert.fail("must not render ListPages"),
      renderCategoriesPageList: async () => {
        called = true
        return { status: "ok", body: "unexpected" }
      }
    })
    const body = await response.json()
    assert.equal(body.status, "not_ok")
    assert.equal(called, false)
  }
})
