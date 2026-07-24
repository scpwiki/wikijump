import { randomUUID } from "node:crypto"

import { expect, test } from "@playwright/test"

import {
  handleXmlRpcRequest,
  parseXmlRpcCall,
  serializeMethodResponse
} from "../src/lib/server/xmlrpc"

test.describe.configure({ mode: "serial" })

const fixtureUrl = `http://127.0.0.1:${process.env.PLAYWRIGHT_FIXTURE_PORT ?? "42747"}`

const requiredEnvironmentValue = (name: string): string => {
  const value = process.env[name]
  if (!value) throw new Error(`Missing required test environment variable: ${name}`)
  return value
}

const wikidotAppName = requiredEnvironmentValue("WIKIDOT_APP_NAME")
const wikidotApiKey = requiredEnvironmentValue("WIKIDOT_API_KEY")
const xmlRpcWritePassword = requiredEnvironmentValue("XML_RPC_WRITE_PASSWORD")

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

const xmlRpcUsersGetMeRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>users.get_me</methodName>
  <params />
</methodCall>`

const xmlRpcUsersGetMeEmptyStructRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>users.get_me</methodName>
  <params>
    <param><value><struct /></value></param>
  </params>
</methodCall>`

const xmlRpcUsersGetMeEmptyArrayRequest = `<?xml version="1.0"?>
<methodCall>
  <methodName>users.get_me</methodName>
  <params>
    <param><value><array><data /></array></value></param>
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

function xmlRpcPagesGetMetaForPagesRequest(pages: string[]): string {
  const pageValues = pages
    .map((page) => `<value><string>${xmlEscape(page)}</string></value>`)
    .join("")

  return `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.get_meta</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>pages</name><value><array><data>${pageValues}</data></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

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

function xmlRpcPagesGetOneForPageRequest(page: string): string {
  return `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.get_one</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>page</name><value><string>${xmlEscape(page)}</string></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

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

function xmlEscape(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;")
}

function xmlRpcFilesSelectRequest(page: string): string {
  return `<?xml version="1.0"?>
<methodCall>
  <methodName>files.select</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>page</name><value><string>${xmlEscape(page)}</string></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcFilesGetMetaRequest(page: string, files: string[]): string {
  return `<?xml version="1.0"?>
<methodCall>
  <methodName>files.get_meta</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>page</name><value><string>${xmlEscape(page)}</string></value></member>
          <member><name>files</name><value><array><data>${files
            .map((file) => `<value><string>${xmlEscape(file)}</string></value>`)
            .join("")}</data></array></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcFilesGetOneRequest(page: string, file: string): string {
  return `<?xml version="1.0"?>
<methodCall>
  <methodName>files.get_one</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>page</name><value><string>${xmlEscape(page)}</string></value></member>
          <member><name>file</name><value><string>${xmlEscape(file)}</string></value></member>
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcFilesSaveOneRequest({
  page,
  file,
  content,
  comment,
  saveMode,
  revisionComment
}: {
  page: string
  file: string
  content: string
  comment?: string
  saveMode?: string
  revisionComment?: string
}): string {
  const optionalMembers = [
    comment !== undefined
      ? `<member><name>comment</name><value><string>${xmlEscape(comment)}</string></value></member>`
      : "",
    saveMode !== undefined
      ? `<member><name>save_mode</name><value><string>${xmlEscape(saveMode)}</string></value></member>`
      : "",
    revisionComment !== undefined
      ? `<member><name>revision_comment</name><value><string>${xmlEscape(revisionComment)}</string></value></member>`
      : ""
  ].join("")

  return `<?xml version="1.0"?>
<methodCall>
  <methodName>files.save_one</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>page</name><value><string>${xmlEscape(page)}</string></value></member>
          <member><name>file</name><value><string>${xmlEscape(file)}</string></value></member>
          <member><name>content</name><value><string>${xmlEscape(content)}</string></value></member>
          ${optionalMembers}
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcPostsSelectRequest(page?: string, replyTo?: string | number): string {
  const pageMember =
    page !== undefined
      ? `<member><name>page</name><value><string>${xmlEscape(page)}</string></value></member>`
      : ""
  const replyToMember =
    replyTo !== undefined
      ? `<member><name>reply_to</name><value>${
          typeof replyTo === "number"
            ? `<int>${replyTo}</int>`
            : `<string>${xmlEscape(replyTo)}</string>`
        }</value></member>`
      : ""

  return `<?xml version="1.0"?>
<methodCall>
  <methodName>posts.select</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          ${pageMember}
          ${replyToMember}
        </struct>
      </value>
    </param>
  </params>
</methodCall>`
}

function xmlRpcPostsGetRequest(
  posts: string[],
  valueType: "string" | "int" = "string"
): string {
  return `<?xml version="1.0"?>
<methodCall>
  <methodName>posts.get</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>posts</name><value><array><data>${posts
            .map((post) =>
              valueType === "int"
                ? `<value><int>${post}</int></value>`
                : `<value><string>${xmlEscape(post)}</string></value>`
            )
            .join("")}</data></array></value></member>
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

const basicAuth = `Basic ${Buffer.from(`${wikidotAppName}:${wikidotApiKey}`).toString("base64")}`
const legacyBasicAuth = `Basic ${Buffer.from("legacy-app:legacy-key").toString("base64")}`

const xmlRpcHeaders = {
  authorization: basicAuth,
  "content-type": "text/xml"
}

const xmlRpcHandlerRequest = (authorization: string): Request =>
  new Request("http://127.0.0.1/xml-rpc-api.php", {
    body: xmlRpcListMethodsRequest,
    headers: {
      authorization,
      "content-type": "text/xml"
    },
    method: "POST"
  })

const restoreEnv = (name: string, value: string | undefined): void => {
  if (value === undefined) {
    delete process.env[name]
  } else {
    process.env[name] = value
  }
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

test("XML-RPC handler prefers complete legacy Basic auth and rejects partial legacy config", async () => {
  const previousEnv = {
    WIKIDOT_API_KEY: process.env.WIKIDOT_API_KEY,
    WIKIDOT_APP_NAME: process.env.WIKIDOT_APP_NAME,
    XML_RPC_PASSWORD: process.env.XML_RPC_PASSWORD,
    XML_RPC_USERNAME: process.env.XML_RPC_USERNAME
  }

  try {
    process.env.WIKIDOT_APP_NAME = "test-app"
    process.env.WIKIDOT_API_KEY = "test-key"
    process.env.XML_RPC_USERNAME = "legacy-app"
    process.env.XML_RPC_PASSWORD = "legacy-key"

    const legacyResponse = await handleXmlRpcRequest(
      xmlRpcHandlerRequest(legacyBasicAuth)
    )
    expect(legacyResponse.status).toBe(200)
    expect(await legacyResponse.text()).toContain("<string>system.listMethods</string>")

    const fallbackResponse = await handleXmlRpcRequest(xmlRpcHandlerRequest(basicAuth))
    expect(fallbackResponse.status).toBe(401)

    delete process.env.XML_RPC_PASSWORD
    const partialLegacyResponse = await handleXmlRpcRequest(
      xmlRpcHandlerRequest(basicAuth)
    )
    expect(partialLegacyResponse.status).toBe(401)
    expect(await partialLegacyResponse.text()).toContain(
      "<name>faultCode</name><value><int>401</int></value>"
    )
  } finally {
    restoreEnv("WIKIDOT_API_KEY", previousEnv.WIKIDOT_API_KEY)
    restoreEnv("WIKIDOT_APP_NAME", previousEnv.WIKIDOT_APP_NAME)
    restoreEnv("XML_RPC_PASSWORD", previousEnv.XML_RPC_PASSWORD)
    restoreEnv("XML_RPC_USERNAME", previousEnv.XML_RPC_USERNAME)
  }
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

  const deepwellRequest = await request.get(`${fixtureUrl}/last-page-tags-request`)
  expect(deepwellRequest.status()).toBe(200)
  expect(await deepwellRequest.json()).toEqual({
    headers: {
      sessionToken: "fixture-session-token"
    },
    params: {
      categories: ["_default"],
      pages: ["the-great-hippo"],
      site: "scp-wiki"
    }
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

  const deepwellRequest = await request.get(`${fixtureUrl}/last-page-select-request`)
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

  const deepwellRequests = await request.get(`${fixtureUrl}/last-page-read-requests`)
  expect(deepwellRequests.status()).toBe(200)
  expect(await deepwellRequests.json()).toEqual({
    forumPostPageSummary: [
      {
        page: "scp-173",
        site_id: 6000005
      },
      {
        page: "scp-173",
        site_id: 6000005
      }
    ],
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
        details: { compiled_html: false, wikitext: false },
        page: "scp-173",
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
    pageView: [
      {
        headers: {
          page: "scp-173",
          sessionToken: "fixture-session-token",
          siteId: "6000005"
        },
        params: {
          locales: [],
          route: { extra: "", slug: "scp-173" },
          session_token: "fixture-session-token",
          site_id: 6000005
        }
      },
      {
        headers: {
          page: "scp-173",
          sessionToken: "fixture-session-token",
          siteId: "6000005"
        },
        params: {
          locales: [],
          route: { extra: "", slug: "scp-173" },
          session_token: "fixture-session-token",
          site_id: 6000005
        }
      }
    ],
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

test("XML-RPC endpoint enforces page view ACLs for page reads", async ({ request }) => {
  const metaResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetMetaForPagesRequest(["scp-173", "private-page"]),
    headers: xmlRpcHeaders
  })
  expect(metaResponse.status()).toBe(200)

  const metaBody = await metaResponse.text()
  expect(metaBody).toContain("<methodResponse>")
  expect(metaBody).toContain(
    "<name>fullname</name><value><string>scp-173</string></value>"
  )
  expect(metaBody).not.toContain("private-page")
  expect(metaBody).not.toContain("Private Page")

  const oneResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetOneForPageRequest("private-page"),
    headers: xmlRpcHeaders
  })
  expect(oneResponse.status()).toBe(403)

  const oneBody = await oneResponse.text()
  expect(oneBody).toContain("<fault>")
  expect(oneBody).toContain("<name>faultCode</name><value><int>403</int></value>")
  expect(oneBody).toContain("XML-RPC user is not allowed to view this page")
  expect(oneBody).not.toContain("Private page body marker")

  const deepwellRequests = await request.get(`${fixtureUrl}/last-page-read-requests`)
  expect(deepwellRequests.status()).toBe(200)
  const readRequests = await deepwellRequests.json()
  expect(
    readRequests.pageView.map(
      (entry: { params: { route: { slug: string } } }) => entry.params.route.slug
    )
  ).toEqual(["scp-173", "private-page", "private-page"])
  expect(
    readRequests.pageGet.some(
      (entry: { page: string; details: { compiled_html: boolean } }) =>
        entry.page === "private-page" && entry.details.compiled_html === true
    )
  ).toBe(false)
  expect(
    readRequests.forumPostPageSummary.some(
      (entry: { page: string }) => entry.page === "private-page"
    )
  ).toBe(false)
})

test("XML-RPC page HTML omits generated CSS that browser views place in head", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetOneForPageRequest("theme:yossistyle"),
    headers: xmlRpcHeaders
  })
  expect(response.status()).toBe(200)

  const body = await response.text()
  expect(body).toContain("XML-RPC theme body marker.")
  const htmlMember = /<name>html<\/name><value><string>(.*?)<\/string><\/value>/s.exec(
    body
  )?.[1]
  expect(htmlMember).toBeDefined()
  expect(htmlMember).toContain("XML-RPC theme body marker.")
  expect(htmlMember).not.toContain("#header h2 span")
  expect(htmlMember).not.toContain("&lt;style")

  const reset = await request.get(`${fixtureUrl}/last-page-read-requests`)
  expect(reset.status()).toBe(200)
})

test("XML-RPC endpoint returns page comment summaries and forum posts", async ({
  request
}) => {
  const pageOneResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesGetOneForPageRequest("xmlrpc-post-page"),
    headers: xmlRpcHeaders
  })
  expect(pageOneResponse.status()).toBe(200)
  const pageOneBody = await pageOneResponse.text()
  expect(pageOneBody).toContain("<name>comments</name><value><int>1</int></value>")
  expect(pageOneBody).toContain(
    "<name>commented_at</name><value><string>2026-06-21T00:00:00Z</string></value>"
  )
  expect(pageOneBody).toContain(
    "<name>commented_by</name><value><string>administrator</string></value>"
  )

  const selectResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPostsSelectRequest("xmlrpc-post-page"),
    headers: xmlRpcHeaders
  })
  expect(selectResponse.status()).toBe(200)
  expect(await selectResponse.text()).toContain("<value><int>7000300</int></value>")

  const siteWideSelectResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPostsSelectRequest(),
    headers: xmlRpcHeaders
  })
  expect(siteWideSelectResponse.status()).toBe(200)
  expect(await siteWideSelectResponse.text()).toContain(
    "<value><int>7000300</int></value>"
  )

  const topLevelResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPostsSelectRequest("xmlrpc-post-page", "-"),
    headers: xmlRpcHeaders
  })
  expect(topLevelResponse.status()).toBe(200)
  expect(await topLevelResponse.text()).toContain("<value><int>7000300</int></value>")

  const directReplyResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPostsSelectRequest("xmlrpc-post-page", 7000300),
    headers: xmlRpcHeaders
  })
  expect(directReplyResponse.status()).toBe(200)
  expect(await directReplyResponse.text()).toContain("<array><data></data></array>")

  const invalidReplyToResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPostsSelectRequest("xmlrpc-post-page", "9223372036854775808"),
    headers: xmlRpcHeaders
  })
  expect(invalidReplyToResponse.status()).toBe(200)
  const invalidReplyToBody = await invalidReplyToResponse.text()
  expect(invalidReplyToBody).toContain("<fault>")
  expect(invalidReplyToBody).toContain(
    "<name>faultCode</name><value><int>-32603</int></value>"
  )

  const postsGetResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPostsGetRequest(["7000300"], "int"),
    headers: xmlRpcHeaders
  })
  expect(postsGetResponse.status()).toBe(200)
  const postsGetBody = await postsGetResponse.text()
  expect(postsGetBody).toContain("<name>7000300</name>")
  expect(postsGetBody).toContain("<name>id</name><value><int>7000300</int></value>")
  expect(postsGetBody).toContain(
    "<name>fullname</name><value><string>xmlrpc-post-page</string></value>"
  )
  expect(postsGetBody).toContain("<name>reply_to</name><value><nil /></value>")
  expect(postsGetBody).toContain(
    "<name>title</name><value><string>XML-RPC comment proof</string></value>"
  )
  expect(postsGetBody).toContain(
    "<name>content</name><value><string>XML-RPC page comment proof body.</string></value>"
  )
  expect(postsGetBody).toContain(
    "<name>html</name><value><string>&lt;p&gt;XML-RPC page comment proof body.&lt;/p&gt;</string></value>"
  )
  expect(postsGetBody).toContain(
    "<name>created_by</name><value><string>administrator</string></value>"
  )
  expect(postsGetBody).toContain(
    "<name>created_at</name><value><string>2026-06-21T00:00:00Z</string></value>"
  )

  const unsafeNumericPostResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPostsGetRequest(["9007199254740992"], "int"),
    headers: xmlRpcHeaders
  })
  expect(unsafeNumericPostResponse.status()).toBe(200)
  const unsafeNumericPostBody = await unsafeNumericPostResponse.text()
  expect(unsafeNumericPostBody).toContain("<fault>")
  expect(unsafeNumericPostBody).toContain(
    "<name>faultCode</name><value><int>-32602</int></value>"
  )

  const outOfRangeStringPostResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPostsGetRequest(["9223372036854775808"]),
    headers: xmlRpcHeaders
  })
  expect(outOfRangeStringPostResponse.status()).toBe(200)
  const outOfRangeStringPostBody = await outOfRangeStringPostResponse.text()
  expect(outOfRangeStringPostBody).toContain("<fault>")
  expect(outOfRangeStringPostBody).toContain(
    "<name>faultCode</name><value><int>-32603</int></value>"
  )
})

test("XML-RPC endpoint saves pages with actor context, parents, tags, and rename", async ({
  request
}) => {
  const resetWriteRequests = await request.get(`${fixtureUrl}/last-page-write-requests`)
  expect(resetWriteRequests.status()).toBe(200)

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

  const writeRequests = await request.get(`${fixtureUrl}/last-page-write-requests`)
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

test("XML-RPC endpoint saves and reads small page attachments", async ({ request }) => {
  const pageSlug = `fixture-xmlrpc-file-${randomUUID()}`
  const fileName = "proof.txt"
  const initialText = "XML-RPC file proof initial content."
  const updatedText = "XML-RPC file proof updated content with extra bytes."
  const initialContent = Buffer.from(initialText).toString("base64")
  const updatedContent = Buffer.from(updatedText).toString("base64")

  const pageResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcPagesSaveOneRequest({
      page: pageSlug,
      title: "XML-RPC File Proof",
      content: "Page for XML-RPC file proof.",
      tags: ["verification", "xmlrpc-file"],
      saveMode: "create",
      revisionComment: "xmlrpc file page create proof"
    }),
    headers: xmlRpcHeaders
  })
  expect(pageResponse.status()).toBe(200)
  expect(await pageResponse.text()).toContain(
    `<name>fullname</name><value><string>${pageSlug}</string></value>`
  )

  const saveResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcFilesSaveOneRequest({
      page: pageSlug,
      file: fileName,
      content: initialContent,
      comment: "initial file proof",
      saveMode: "create",
      revisionComment: "xmlrpc file create proof"
    }),
    headers: xmlRpcHeaders
  })
  expect(saveResponse.status()).toBe(200)

  const saveBody = await saveResponse.text()
  expect(saveBody).toContain("<methodResponse>")
  expect(saveBody).toContain("<name>size</name><value><int>35</int></value>")
  expect(saveBody).toContain(
    "<name>comment</name><value><string>xmlrpc file create proof</string></value>"
  )
  expect(saveBody).toContain("<name>mime_type</name><value><string>text/plain")

  const selectResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcFilesSelectRequest(pageSlug),
    headers: xmlRpcHeaders
  })
  expect(selectResponse.status()).toBe(200)
  expect(await selectResponse.text()).toContain("<string>proof.txt</string>")

  const metaResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcFilesGetMetaRequest(pageSlug, [fileName]),
    headers: xmlRpcHeaders
  })
  expect(metaResponse.status()).toBe(200)
  const metaBody = await metaResponse.text()
  expect(metaBody).toContain("<name>proof.txt</name>")
  expect(metaBody).toContain("<name>size</name><value><int>35</int></value>")
  expect(metaBody).toContain(
    "<name>comment</name><value><string>xmlrpc file create proof</string></value>"
  )
  expect(metaBody).not.toContain(initialContent)

  const oneResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcFilesGetOneRequest(pageSlug, fileName),
    headers: xmlRpcHeaders
  })
  expect(oneResponse.status()).toBe(200)
  const oneBody = await oneResponse.text()
  expect(oneBody).toContain(
    `<name>content</name><value><string>${initialContent}</string></value>`
  )

  const updateResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcFilesSaveOneRequest({
      page: pageSlug,
      file: fileName,
      content: updatedContent,
      comment: "updated file proof",
      saveMode: "update",
      revisionComment: "xmlrpc file update proof"
    }),
    headers: xmlRpcHeaders
  })
  expect(updateResponse.status()).toBe(200)
  const updateBody = await updateResponse.text()
  expect(updateBody).toContain("<name>size</name><value><int>52</int></value>")
  expect(updateBody).toContain(
    "<name>comment</name><value><string>xmlrpc file update proof</string></value>"
  )

  const updatedOneResponse = await request.post("/xml-rpc-api.php", {
    data: xmlRpcFilesGetOneRequest(pageSlug, fileName),
    headers: xmlRpcHeaders
  })
  expect(updatedOneResponse.status()).toBe(200)
  const updatedOneBody = await updatedOneResponse.text()
  expect(updatedOneBody).toContain(
    `<name>content</name><value><string>${updatedContent}</string></value>`
  )
  expect(updatedOneBody).not.toContain(initialContent)

  const fileLogResponse = await request.get(`${fixtureUrl}/last-file-requests`)
  expect(fileLogResponse.status()).toBe(200)
  const fileLog = await fileLogResponse.json()
  expect(fileLog.blobUpload).toHaveLength(2)
  expect(fileLog.fileCreate).toHaveLength(1)
  expect(fileLog.fileEdit).toHaveLength(1)
  expect(fileLog.pageGetFiles).toHaveLength(1)
  expect(fileLog.fileCreate[0].params).toMatchObject({
    bypass_filter: true,
    name: fileName,
    revision_comments: "xmlrpc file create proof",
    user_id: 123
  })
  expect(fileLog.fileCreate[0].headers).toMatchObject({
    page: pageSlug,
    sessionToken: "fixture-session-token",
    siteId: "6000005"
  })
  expect(fileLog.fileEdit[0].params).toMatchObject({
    bypass_filter: true,
    revision_comments: "xmlrpc file update proof",
    user_id: 123
  })
  expect(fileLog.fileEdit[0].headers).toMatchObject({
    page: pageSlug,
    sessionToken: "fixture-session-token",
    siteId: "6000005"
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

test("XML-RPC endpoint returns the authenticated XML-RPC principal", async ({
  request
}) => {
  const resetWriteRequests = await request.get(`${fixtureUrl}/last-page-write-requests`)
  expect(resetWriteRequests.status()).toBe(200)

  for (const data of [
    xmlRpcUsersGetMeRequest,
    xmlRpcUsersGetMeEmptyStructRequest,
    xmlRpcUsersGetMeEmptyArrayRequest
  ]) {
    const response = await request.post("/xml-rpc-api.php", {
      data,
      headers: xmlRpcHeaders
    })

    expect(response.status()).toBe(200)
    expect(response.headers()["content-type"]).toContain("text/xml")

    const body = await response.text()
    expect(body).toContain("<methodResponse>")
    expect(body).not.toContain("<fault>")
    expect(body).toContain("<name>name</name><value><string>rokurokubi</string></value>")
    expect(body).toContain("<name>title</name><value><string>Rokurokubi</string></value>")
    expect(body).toContain("<name>id</name><value><int>123</int></value>")
    expect(body).not.toContain(wikidotApiKey)
    expect(body).not.toContain(xmlRpcWritePassword)
    expect(body).not.toContain("fixture-session-token")
  }

  const writeRequests = await request
    .get(`${fixtureUrl}/last-page-write-requests`)
    .then((response) => response.json())
  expect(writeRequests.login).toHaveLength(0)
  expect(writeRequests.sessionGet).toHaveLength(0)
  expect(writeRequests.userGet).toHaveLength(3)
  for (const userGetRequest of writeRequests.userGet) {
    expect(userGetRequest).toMatchObject({
      params: { user: "rokurokubi" }
    })
    expect(userGetRequest.headers).not.toHaveProperty("sessionToken")
  }
})

test("XML-RPC endpoint reports unsupported unknown methods", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcUnknownMethodRequest,
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<name>faultCode</name><value><int>-32601</int></value>")
  expect(body).toContain("Unsupported XML-RPC method: not.realMethod")
})

test("XML-RPC endpoint accepts Basic auth scheme case-insensitively", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcListMethodsRequest,
    headers: {
      authorization: basicAuth.replace(/^Basic/u, "basic"),
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
  expect(body).not.toContain(wikidotApiKey)
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

test("XML-RPC endpoint rejects duplicate struct members", async ({ request }) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: `<?xml version="1.0"?>
<methodCall>
  <methodName>pages.select</methodName>
  <params>
    <param>
      <value>
        <struct>
          <member><name>site</name><value><string>scp-wiki</string></value></member>
          <member><name>site</name><value><string>other-site</string></value></member>
        </struct>
      </value>
    </param>
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
  expect(body).toContain("Duplicate XML-RPC struct member: site")
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
