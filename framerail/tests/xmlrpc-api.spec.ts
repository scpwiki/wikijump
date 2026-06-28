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

test("XML-RPC endpoint reports advertised but unimplemented methods", async ({
  request
}) => {
  const response = await request.post("/xml-rpc-api.php", {
    data: xmlRpcAdvertisedUnimplementedRequest,
    headers: xmlRpcHeaders
  })

  expect(response.status()).toBe(200)
  expect(response.headers()["content-type"]).toContain("text/xml")

  const body = await response.text()
  expect(body).toContain("<methodResponse>")
  expect(body).toContain("<name>faultCode</name><value><int>-32601</int></value>")
  expect(body).toContain("XML-RPC method is not implemented yet: pages.select")
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
