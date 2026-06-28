import { randomUUID } from "node:crypto"

import { expect, test } from "@playwright/test"

import { parseXmlRpcCall, serializeMethodResponse } from "../src/lib/server/xmlrpc"

const xmlRpcListMethodsRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <params />
</methodCall>`

const xmlRpcUnknownMethodRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>not.realMethod</methodName>
  <params />
</methodCall>`

const xmlRpcMethodHelpRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>system.methodHelp</methodName>
  <params>
    <param><value><string>pages.select</string></value></param>
  </params>
</methodCall>`

const xmlRpcMethodSignatureRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>system.methodSignature</methodName>
  <params>
    <param><value><string>system.multicall</string></value></param>
  </params>
</methodCall>`

const xmlRpcInheritedMethodHelpRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>system.methodHelp</methodName>
  <params>
    <param><value><string>constructor</string></value></param>
  </params>
</methodCall>`

const xmlRpcMulticallRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>system.multicall</methodName>
  <params>
    <param>
      <value>
        <array>
          <data>
            <value>
              <struct>
                <member><name>methodName</name><value><string>system.listMethods</string></value></member>
                <member><name>params</name><value><array><data /></array></value></member>
              </struct>
            </value>
            <value>
              <struct>
                <member><name>methodName</name><value><string>not.realMethod</string></value></member>
                <member><name>params</name><value><array><data /></array></value></member>
              </struct>
            </value>
          </data>
        </array>
      </value>
    </param>
  </params>
</methodCall>`

const xmlRpcNestedMulticallRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>system.multicall</methodName>
  <params>
    <param>
      <value>
        <array>
          <data>
            <value>
              <struct>
                <member><name>methodName</name><value><string>system.multicall</string></value></member>
                <member><name>params</name><value><array><data /></array></value></member>
              </struct>
            </value>
          </data>
        </array>
      </value>
    </param>
  </params>
</methodCall>`

const xmlRpcAdvertisedUnimplementedRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.select</methodName>
  <params>
    <param><value><struct /></value></param>
  </params>
</methodCall>`

const xmlRpcCategoriesSelectRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>categories.select</methodName>
  <params>
    <param><value><struct><member><name>site</name><value><string>scp-wiki</string></value></member></struct></value></param>
  </params>
</methodCall>`

const xmlRpcTagsSelectRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>tags.select</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>categories</name><value><array><data><value><string>_default</string></value></data></array></value></member>
          <member><name>pages</name><value><array><data><value><string>the-great-hippo</string></value></data></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`

const xmlRpcPagesSelectRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.select</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>pagetype</name><value><string>normal</string></value></member>
          <member><name>categories</name><value><array><data><value><string>_default</string></value></data></array></value></member>
          <member><name>created_by</name><value><string>-1</string></value></member>
          <member><name>rating</name><value><string>&gt;=0</string></value></member>
          <member><name>order</name><value><string>created_at desc</string></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`

const xmlRpcPagesGetMetaRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.get_meta</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>pages</name><value><array><data><value><string>scp-173</string></value><value><string>missing-page</string></value></data></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`

const xmlRpcPagesGetOneRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.get_one</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>page</name><value><string>scp-173</string></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`

const xmlRpcPagesGetMetaTooManyRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.get_meta</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>pages</name><value><array><data><value><string>page-01</string></value><value><string>page-02</string></value><value><string>page-03</string></value><value><string>page-04</string></value><value><string>page-05</string></value><value><string>page-06</string></value><value><string>page-07</string></value><value><string>page-08</string></value><value><string>page-09</string></value><value><string>page-10</string></value><value><string>page-11</string></value></data></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`

const xmlRpcPagesGetMetaMissingSiteRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.get_meta</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>missing-site</string></value></member>
          <member><name>pages</name><value><array><data><value><string>scp-173</string></value></data></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`

const xmlRpcPagesGetMetaMissingSiteEmptyPagesRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.get_meta</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>missing-site</string></value></member>
          <member><name>pages</name><value><array><data /></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`

function xmlRpcPagesSaveOneRequest({
  page,
  title,
  content,
  tags,
  parentFullname,
  saveMode,
  renameAs,
  revisionComment
}: {
  page: string
  title?: string
  content?: string
  tags?: string[]
  parentFullname?: string
  saveMode?: string
  renameAs?: string
  revisionComment?: string
}): string {
  const optionalMembers = [
    title !== undefined
      ? `<member><name>title</name><value><string>${title}</string></value></member>`
      : "",
    content !== undefined
      ? `<member><name>content</name><value><string>${content}</string></value></member>`
      : "",
    tags !== undefined
      ? `<member><name>tags</name><value><array><data>${tags
          .map((tag) => `<value><string>${tag}</string></value>`)
          .join("")}</data></array></value></member>`
      : "",
    parentFullname !== undefined
      ? `<member><name>parent_fullname</name><value><string>${parentFullname}</string></value></member>`
      : "",
    saveMode !== undefined
      ? `<member><name>save_mode</name><value><string>${saveMode}</string></value></member>`
      : "",
    renameAs !== undefined
      ? `<member><name>rename_as</name><value><string>${renameAs}</string></value></member>`
      : "",
    revisionComment !== undefined
      ? `<member><name>revision_comment</name><value><string>${revisionComment}</string></value></member>`
      : ""
  ].join("")

  return `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.save_one</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>page</name><value><string>${page}</string></value></member>
          ${optionalMembers}
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcPagesSelectWithFilterCount(
  filterName: "categories" | "tags_any" | "tags_all" | "tags_none",
  count: number
): string {
  const filterValues = Array.from(
    { length: count },
    (_, index) => `<value><string>${filterName}-${index}</string></value>`
  ).join("")

  return `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.select</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>${filterName}</name><value><array><data>${filterValues}</data></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcPagesSelectWithScalarFilter(name: string, value: string): string {
  return `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.select</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>${name}</name><value><string>${value}</string></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcTagsSelectWithFilterCount(
  filterName: "categories" | "pages",
  count: number
): string {
  const filterValues = Array.from(
    { length: count },
    (_, index) => `<value><string>${filterName}-${index}</string></value>`
  ).join("")

  return `<?xml version="1.0"?>
<methodCall>
  <methodName>tags.select</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>${filterName}</name><value><array><data>${filterValues}</data></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcMulticallWithChildCount(count: number): string {
  const childCall = `<value><struct><member><name>methodName</name><value><string>system.listMethods</string></value></member><member><name>params</name><value><array><data /></array></value></member></struct></value>`

  return `<?xml version="1.0"?>
<methodCall>
  <methodName>system.multicall</methodName>
  <params>
    <param>
      <value>
        <array>
          <data>${childCall.repeat(count)}</data>
        </array>
      </value>
    </param>
  </params>
</methodCall>`
}

const basicAuth = `Basic ${Buffer.from("test-app:test-key").toString("base64")}`

const xmlRpcHeaders = {
  authorization: basicAuth,
  "content-type": "text/xml"
}

test("XML-RPC serializer preserves tiny non-zero doubles", () => {
  expect(serializeMethodResponse(1e-21)).toContain(
    "<double>0.000000000000000000001</double>"
  )
})

test("XML-RPC parser preserves shorthand string whitespace", () => {
  const call = parseXmlRpcCall(`<methodCall>
  <methodName>not.realMethod</methodName>
  <params><param><value> x </value></param></params>
</methodCall>`)

  expect(call.params[0]).toBe(" x ")
})

test("XML-RPC endpoint accepts Basic-authenticated system.listMethods calls", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcListMethodsRequest,
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<array>")
  expect(body).toContain("<string>system.listMethods</string>")
  expect(body).toContain("<string>system.methodHelp</string>")
  expect(body).toContain("<string>system.methodSignature</string>")
  expect(body).toContain("<string>system.multicall</string>")
  expect(body).toContain("<string>pages.select</string>")
})

test("XML-RPC endpoint exposes system method help and signatures", async ({
  request
}) => {
  const helpResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcMethodHelpRequest,
    headers: xmlRpcHeaders
  })
  expect(helpResponse.status()).toBe(200)
  expect(await helpResponse.text()).toContain(
    "Select pages from a Wikidot-compatible site"
  )

  const signatureResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcMethodSignatureRequest,
    headers: xmlRpcHeaders
  })
  const signatureBody = await signatureResponse.text()
  expect(signatureResponse.status()).toBe(200)
  expect(signatureBody).toContain("<string>array</string>")
})

test("XML-RPC endpoint does not treat inherited object properties as methods", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcInheritedMethodHelpRequest,
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32601</int></value>")
  expect(body).toContain("Unsupported XML-RPC method: constructor")
  expect(body).not.toContain("<param>undefined</param>")
})

test("XML-RPC endpoint supports system.multicall with partial faults", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcMulticallRequest,
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<string>system.listMethods</string>")
  expect(body).toContain("<name>faultCode</name><value><int>-32601</int></value>")
  expect(body).toContain("<name>faultString</name>")
})

test("XML-RPC endpoint rejects nested system.multicall child calls", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcNestedMulticallRequest,
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
  expect(body).toContain("Nested system.multicall calls are not supported")
})

test("XML-RPC endpoint bounds system.multicall child call count", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcMulticallWithChildCount(101),
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
  expect(body).toContain("system.multicall accepts at most 100 calls")
})

test("XML-RPC endpoint selects local categories", async ({ request }) => {
  const categoriesResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcCategoriesSelectRequest,
    headers: xmlRpcHeaders
  })
  expect(categoriesResponse.status()).toBe(200)
  const categoriesBody = await categoriesResponse.text()
  expect(categoriesBody).toContain("<string>_default</string>")
  expect(categoriesBody).toContain("<string>nav</string>")
})

test("XML-RPC endpoint selects local tags", async ({ request }) => {
  const tagsResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcTagsSelectRequest,
    headers: xmlRpcHeaders
  })

  expect(tagsResponse.status()).toBe(200)
  const tagsBody = await tagsResponse.text()
  expect(tagsBody).toContain("<string>_cc</string>")
  expect(tagsBody).toContain("<string>tale</string>")

  const deepwellRequest = await request.get(
    "http://127.0.0.1:42747/last-page-tags-request"
  )
  expect(deepwellRequest.status()).toBe(200)
  expect(await deepwellRequest.json()).toEqual({
    categories: ["_default"],
    pages: ["the-great-hippo"],
    site: "scp-wiki"
  })
})

test("XML-RPC endpoint selects pages with documented filters and ordering", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesSelectRequest,
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  const body = await response.text()
  expect(body).toContain("<string>scp-173</string>")
  expect(body).toContain("<string>scp-anthology-2024</string>")
  expect(body).toContain("<string>scp-8566</string>")
  expect(body.indexOf("scp-173")).toBeLessThan(body.indexOf("scp-anthology-2024"))
  expect(body.indexOf("scp-anthology-2024")).toBeLessThan(body.indexOf("scp-8566"))

  const deepwellRequest = await request.get(
    "http://127.0.0.1:42747/last-page-select-request"
  )
  expect(deepwellRequest.status()).toBe(200)
  expect(await deepwellRequest.json()).toEqual({
    categories: ["_default"],
    created_by: "-1",
    order: "created_at desc",
    pagetype: "normal",
    rating: ">=0",
    site: "scp-wiki"
  })
})

test("XML-RPC endpoint bounds pages.select filters", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesSelectWithFilterCount("tags_all", 101),
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
  expect(body).toContain("pages.select tags_all is limited to 100 entries")
})

test("XML-RPC endpoint rejects invalid pages.select scalar filters", async ({
  request
}) => {
  for (const [name, value, message] of [
    ["pagetype", "forum", "Unsupported pages.select pagetype: forum"],
    ["rating", ">=not-a-number", "Invalid pages.select rating filter: &gt;=not-a-number"],
    ["rating", "NaN", "Invalid pages.select rating filter: NaN"],
    ["rating", "0x10", "Invalid pages.select rating filter: 0x10"],
    ["order", "created_at sideways", "Unsupported pages.select order direction: sideways"]
  ]) {
    const response = await request.post("/xml-rpc-api.php", {
      data: xmlRpcPagesSelectWithScalarFilter(name, value),
      headers: xmlRpcHeaders
    })

    expect(response.status()).toBe(200)
    const body = await response.text()
    expect(body).toContain("<methodResponse>")
    expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
    expect(body).toContain(message)
    expect(body).not.toContain("XML-RPC Deepwell request failed")
  }
})

test("XML-RPC endpoint returns page metadata and bodies for corpus clients", async ({
  request
}) => {
  const metaResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetMetaRequest,
    headers: xmlRpcHeaders
  })
  expect(metaResponse.status()).toBe(200)

  const metaBody = await metaResponse.text()
  expect(metaBody).toContain("<methodResponse>")
  expect(metaBody).toContain("<name>scp-173</name>")
  expect(metaBody).not.toContain("<name>missing-page</name>")
  expect(metaBody).toContain(
    "<name>fullname</name><value><string>scp-173</string></value>"
  )
  expect(metaBody).toContain("<name>title</name><value><string>SCP-173</string></value>")
  expect(metaBody).toContain(
    "<name>parent_fullname</name><value><string>scp-173-parent</string></value>"
  )
  expect(metaBody).toContain("<name>created_by</name><value><string>123</string></value>")
  expect(metaBody).toContain("<name>updated_by</name><value><string>456</string></value>")
  expect(metaBody).toContain("<name>tags</name><value><array><data>")
  expect(metaBody).toContain("<name>rating</name><value><int>173</int></value>")
  expect(metaBody).toContain("<name>revisions</name><value><int>3</int></value>")
  expect(metaBody).not.toContain("Item #:")

  const oneResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetOneRequest,
    headers: xmlRpcHeaders
  })
  expect(oneResponse.status()).toBe(200)

  const oneBody = await oneResponse.text()
  expect(oneBody).toContain("<methodResponse>")
  expect(oneBody).toContain(
    "<name>fullname</name><value><string>scp-173</string></value>"
  )
  expect(oneBody).toContain("<name>created_by</name><value><string>123</string></value>")
  expect(oneBody).toContain("<name>updated_by</name><value><string>456</string></value>")
  expect(oneBody).toContain("<name>content</name><value><string>")
  expect(oneBody).toContain("**Item #:** SCP-173")
  expect(oneBody).toContain("<name>html</name><value><string>")
  expect(oneBody).toContain("&lt;strong&gt;Item #:&lt;/strong&gt; SCP-173")
  expect(oneBody).toContain(
    "<name>parent_title</name><value><string>SCP Foundation</string></value>"
  )
  expect(oneBody).toContain("<name>children</name><value><int>2</int></value>")
  expect(oneBody).toContain("<name>comments</name><value><int>0</int></value>")
  expect(oneBody).toContain("<name>commented_at</name><value><nil /></value>")
  expect(oneBody).toContain("<name>commented_by</name><value><nil /></value>")

  const tooManyResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetMetaTooManyRequest,
    headers: xmlRpcHeaders
  })
  expect(tooManyResponse.status()).toBe(200)

  const tooManyBody = await tooManyResponse.text()
  expect(tooManyBody).toContain("<fault>")
  expect(tooManyBody).toContain("<name>faultCode</name><value><int>-32602</int></value>")
  expect(tooManyBody).toContain("pages.get_meta pages is limited to 10 entries")

  const missingSiteResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetMetaMissingSiteRequest,
    headers: xmlRpcHeaders
  })
  expect(missingSiteResponse.status()).toBe(200)

  const missingSiteBody = await missingSiteResponse.text()
  expect(missingSiteBody).toContain("<fault>")
  expect(missingSiteBody).toContain("<name>faultCode</name><value><int>406</int></value>")
  expect(missingSiteBody).toContain("Argument site invalid: site does not exist")

  const missingSiteEmptyPagesResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetMetaMissingSiteEmptyPagesRequest,
    headers: xmlRpcHeaders
  })
  expect(missingSiteEmptyPagesResponse.status()).toBe(200)

  const missingSiteEmptyPagesBody = await missingSiteEmptyPagesResponse.text()
  expect(missingSiteEmptyPagesBody).toContain("<fault>")
  expect(missingSiteEmptyPagesBody).toContain(
    "<name>faultCode</name><value><int>406</int></value>"
  )
  expect(missingSiteEmptyPagesBody).toContain(
    "Argument site invalid: site does not exist"
  )

  const deepwellRequests = await request.get(
    "http://127.0.0.1:42747/last-page-read-requests"
  )
  expect(deepwellRequests.status()).toBe(200)
  expect(await deepwellRequests.json()).toEqual({
    pageGet: [
      {
        details: { compiled_html: false, wikitext: false },
        page: "scp-173",
        site_id: 6000005
      },
      {
        details: { compiled_html: false, wikitext: false },
        page: "missing-page",
        site_id: 6000005
      },
      {
        details: { compiled_html: true, wikitext: true },
        page: "scp-173",
        site_id: 6000005
      }
    ],
    pageGetDirect: [
      {
        allow_deleted: false,
        details: { compiled_html: false, wikitext: false },
        page_id: 3000172,
        site_id: 6000005
      },
      {
        allow_deleted: false,
        details: { compiled_html: false, wikitext: false },
        page_id: 3000172,
        site_id: 6000005
      }
    ],
    pageRevisionGet: [
      {
        details: { compiled_html: false, wikitext: false },
        page_id: 3000173,
        revision_number: 0,
        site_id: 6000005
      },
      {
        details: { compiled_html: false, wikitext: false },
        page_id: 3000173,
        revision_number: 0,
        site_id: 6000005
      }
    ],
    pageSelect: [{ parent: "scp-173", site: "scp-wiki" }],
    parentRelationshipsGet: [
      {
        page: "scp-173",
        relationship_type: "parents",
        site_id: 6000005
      },
      {
        page: "scp-173",
        relationship_type: "parents",
        site_id: 6000005
      }
    ],
    siteGet: [
      { site: "scp-wiki" },
      { site: "scp-wiki" },
      { site: "missing-site" },
      { site: "missing-site" }
    ]
  })
})

test("XML-RPC endpoint saves pages with actor context, parents, tags, and rename", async ({
  request
}) => {
  const slug = `fixture-xmlrpc-save-${randomUUID()}`
  const renamedSlug = `${slug}-renamed`

  const createResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesSaveOneRequest({
      page: slug,
      title: "XML-RPC Save Proof",
      content: "XML-RPC save proof initial content.",
      tags: ["verification", "xmlrpc-save"],
      parentFullname: "main",
      saveMode: "create",
      revisionComment: "xmlrpc save create proof"
    }),
    headers: xmlRpcHeaders
  })
  expect(createResponse.status()).toBe(200)

  const createBody = await createResponse.text()
  expect(createBody).toContain(
    `<name>fullname</name><value><string>${slug}</string></value>`
  )
  expect(createBody).toContain(
    "<name>title</name><value><string>XML-RPC Save Proof</string></value>"
  )
  expect(createBody).toContain(
    "<name>content</name><value><string>XML-RPC save proof initial content.</string></value>"
  )
  expect(createBody).toContain(
    "<name>parent_fullname</name><value><string>main</string></value>"
  )
  expect(createBody).toContain("<value><string>xmlrpc-save</string></value>")

  const updateResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesSaveOneRequest({
      page: slug,
      title: "XML-RPC Save Proof Updated",
      content: "XML-RPC save proof updated content.",
      tags: ["verification", "xmlrpc-save-updated"],
      parentFullname: "-",
      saveMode: "update",
      revisionComment: "xmlrpc save update proof"
    }),
    headers: xmlRpcHeaders
  })
  expect(updateResponse.status()).toBe(200)

  const updateBody = await updateResponse.text()
  expect(updateBody).toContain(
    "<name>title</name><value><string>XML-RPC Save Proof Updated</string></value>"
  )
  expect(updateBody).toContain(
    "<name>content</name><value><string>XML-RPC save proof updated content.</string></value>"
  )
  expect(updateBody).toContain("<name>parent_fullname</name><value><nil /></value>")
  expect(updateBody).toContain("<value><string>xmlrpc-save-updated</string></value>")
  expect(updateBody).not.toContain("<value><string>xmlrpc-save</string></value>")

  const renameResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesSaveOneRequest({
      page: slug,
      title: "XML-RPC Save Proof Renamed",
      content: "XML-RPC save proof renamed content.",
      tags: ["verification", "xmlrpc-save-renamed"],
      parentFullname: "main",
      saveMode: "update",
      renameAs: renamedSlug,
      revisionComment: "xmlrpc save rename proof"
    }),
    headers: xmlRpcHeaders
  })
  expect(renameResponse.status()).toBe(200)

  const renameBody = await renameResponse.text()
  expect(renameBody).toContain(
    `<name>fullname</name><value><string>${renamedSlug}</string></value>`
  )
  expect(renameBody).toContain(
    "<name>title</name><value><string>XML-RPC Save Proof Renamed</string></value>"
  )
  expect(renameBody).toContain(
    "<name>content</name><value><string>XML-RPC save proof renamed content.</string></value>"
  )
  expect(renameBody).toContain(
    "<name>parent_fullname</name><value><string>main</string></value>"
  )
  expect(renameBody).toContain("<value><string>xmlrpc-save-renamed</string></value>")

  const writeRequests = await request.get(
    "http://127.0.0.1:42747/last-page-write-requests"
  )
  expect(writeRequests.status()).toBe(200)
  const writeLog = await writeRequests.json()
  expect(writeLog.login).toHaveLength(3)
  expect(writeLog.sessionGet).toHaveLength(3)
  expect(writeLog.pageCreate).toHaveLength(1)
  expect(writeLog.pageEdit).toHaveLength(3)
  expect(writeLog.parentGetAll).toHaveLength(3)
  expect(writeLog.parentUpdate).toHaveLength(3)
  expect(writeLog.pageMove).toHaveLength(1)
  expect(writeLog.pageCreate[0].params).toMatchObject({
    revision_comments: "xmlrpc save create proof",
    title: "XML-RPC Save Proof",
    user_id: 123,
    wikitext: "XML-RPC save proof initial content."
  })
  expect(writeLog.pageCreate[0].headers).toMatchObject({
    page: slug,
    sessionToken: "fixture-session-token",
    siteId: "6000005"
  })
  expect(writeLog.parentGetAll[0].headers).toMatchObject({
    page: slug,
    sessionToken: "fixture-session-token",
    siteId: "6000005"
  })
  expect(writeLog.pageEdit[0].params).toMatchObject({
    tags: ["verification", "xmlrpc-save"],
    user_id: 123
  })
  expect(writeLog.pageMove[0].params).toMatchObject({
    new_slug: renamedSlug,
    page: slug,
    user_id: 123
  })
})

test("XML-RPC endpoint bounds tags.select page filters", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcTagsSelectWithFilterCount("pages", 101),
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
  expect(body).toContain("tags.select pages is limited to 100 entries")
})

test("XML-RPC endpoint accepts tags.select page filters at the cap", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcTagsSelectWithFilterCount("pages", 100),
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).not.toContain("<fault>")
  expect(body).toContain("<string>_cc</string>")
  expect(body).toContain("<string>tale</string>")
})

test("XML-RPC endpoint bounds tags.select category filters", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcTagsSelectWithFilterCount("categories", 101),
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
  expect(body).toContain("tags.select categories is limited to 100 entries")
})

test("XML-RPC endpoint accepts tags.select category filters at the cap", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcTagsSelectWithFilterCount("categories", 100),
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).not.toContain("<fault>")
  expect(body).toContain("<string>_cc</string>")
  expect(body).toContain("<string>tale</string>")
})

test("XML-RPC endpoint reports advertised but unimplemented methods", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcAdvertisedUnimplementedRequest.replace("pages.select", "files.save_one"),
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<name>faultCode</name><value><int>-32601</int></value>")
  expect(body).toContain("XML-RPC method is not implemented yet: files.save_one")
})

test("XML-RPC endpoint accepts Basic auth scheme case-insensitively", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcListMethodsRequest,
    headers: {
      authorization: `basic ${Buffer.from("test-app:test-key").toString("base64")}`,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<string>system.listMethods</string>")
})

test("XML-RPC endpoint returns XML-RPC faults for unauthenticated requests", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcListMethodsRequest,
    headers: {
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(401)
  expect(response.headers()["content-type"]).toContain("text/xml")
  expect(response.headers()["www-authenticate"]).toBe('Basic realm="Wikijump XML-RPC"')

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>401</int></value>")
  expect(body).not.toContain("test-key")
})

test("XML-RPC endpoint returns XML-RPC faults for invalid Basic auth headers", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcListMethodsRequest,
    headers: {
      authorization: "Basic not-base64",
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(401)
  expect(response.headers()["content-type"]).toContain("text/xml")
  expect(response.headers()["www-authenticate"]).toBe('Basic realm="Wikijump XML-RPC"')

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>401</int></value>")
})

test("XML-RPC endpoint rejects wrong but well-formed Basic credentials", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcListMethodsRequest,
    headers: {
      authorization: `Basic ${Buffer.from("test-app:wrong-key").toString("base64")}`,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(401)
  expect(response.headers()["content-type"]).toContain("text/xml")
  expect(response.headers()["www-authenticate"]).toBe('Basic realm="Wikijump XML-RPC"')

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>401</int></value>")
  expect(body).not.toContain("wrong-key")
})

test("XML-RPC endpoint returns XML-RPC faults for malformed XML", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: "<methodCall>",
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects malformed UTF-8 request bodies", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: Buffer.from([0xc3, 0x28]),
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32700</int></value>")
})

test("XML-RPC endpoint requires methodCall as the document root", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<wrapper>
  <methodCall>
    <methodName>system.listMethods</methodName>
    <params />
  </methodCall>
</wrapper>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects non-XML whitespace before the document root", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `\f<methodCall>
  <methodName>system.listMethods</methodName>
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects tag-name prefixes before the real document root", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCallExtra />
<methodCall>
  <methodName>system.listMethods</methodName>
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint decodes numeric character references", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system&#46;listMethods</methodName>
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<string>system.listMethods</string>")
})

test("XML-RPC endpoint decodes XML entities only once", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system&#38;#46;listMethods</methodName>
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32601</int></value>")
})

test("XML-RPC endpoint rejects unknown XML entities", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system&unknown;listMethods</methodName>
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects invalid XML character references", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system&#0;listMethods</methodName>
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects oversized numeric character references", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>not.realMethod</methodName>
  <params>
    <param><value><string>&#${"9".repeat(400)};</string></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects nested XML declarations in element content", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<methodCall>
  <methodName>not.real<?xml version="1.0"?>Method</methodName>
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects comments instead of stripping partial comment bodies", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <!--<methodName>system.listMethods</methodName>-->
  <methodName>system.listMethods</methodName>
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects partial numeric tokens", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <params>
    <param><value><int>12abc</int></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
})

test("XML-RPC endpoint rejects parameters for system.listMethods", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <params>
    <param><value><string>unexpected</string></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
})

test("XML-RPC endpoint rejects skipped child content in containers", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <params>
    <bogus />
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects non-XML whitespace in structural gaps", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <params>\v<param><value><string>x</string></value></param></params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects skipped child content in methodCall", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <bogus />
  <params />
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects skipped child content in param", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>not.realMethod</methodName>
  <params>
    <param><bogus /><value><string>x</string></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects skipped child content in struct member", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>not.realMethod</methodName>
  <params>
    <param><value><struct><member><name>x</name><bogus /><value><string>y</string></value></member></struct></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects skipped child content in arrays", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>not.realMethod</methodName>
  <params>
    <param><value><array><bogus /><data /></array></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32600</int></value>")
})

test("XML-RPC endpoint rejects out-of-range integers", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <params>
    <param><value><int>2147483648</int></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
})

test("XML-RPC endpoint rejects exponent-form doubles", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <params>
    <param><value><double>1.2e3</double></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
})

test("XML-RPC endpoint rejects base64 values until they can round-trip", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>not.realMethod</methodName>
  <params>
    <param><value><base64>YWJj</base64></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
})

test("XML-RPC endpoint rejects dateTime values until they can round-trip", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>not.realMethod</methodName>
  <params>
    <param><value><dateTime.iso8601>20260628T07:50:00</dateTime.iso8601></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
})

test("XML-RPC endpoint rejects sibling value elements", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>system.listMethods</methodName>
  <params>
    <param><value><string>x</string><int>1</int></value></param>
  </params>
</methodCall>`,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32602</int></value>")
})

test("XML-RPC endpoint returns XML-RPC faults for unsupported methods", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcUnknownMethodRequest,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>-32601</int></value>")
})

test("XML-RPC endpoint lets SvelteKit reject non-POST GET requests", async ({
  request
}) => {
  const response = await request.get("/xml-rpc-api.php", {
    headers: {
      authorization: basicAuth
    }
  })

  expect(response.status()).toBe(405)
  expect(response.headers().allow).toBe("POST")
})

test("XML-RPC endpoint lets SvelteKit reject non-POST HEAD requests", async ({
  request
}) => {
  const response = await request.head("/xml-rpc-api.php", {
    headers: {
      authorization: basicAuth
    }
  })

  expect(response.status()).toBe(405)
  expect(response.headers().allow).toBe("POST")
})

test("XML-RPC endpoint rejects oversized request bodies", async ({ request }) => {
  const oversizedBody = `${" ".repeat(1_048_577)}<methodCall />`
  const response = await request.post("/xml-rpc-api.php", {
    data: oversizedBody,
    headers: {
      authorization: basicAuth,
      "content-type": "text/xml"
    }
  })

  expect(response.status()).toBe(413)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<fault>")
  expect(body).toContain("<name>faultCode</name><value><int>413</int></value>")
})
