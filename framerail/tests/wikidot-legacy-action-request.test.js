// @ts-nocheck
import { strict as assert } from "node:assert"
import test from "node:test"

import {
  requestLegacyRate,
  requestLegacyRateCancel,
  requestLegacyScore,
  requestLegacySetTags
} from "../src/lib/wikidot/wikidot-legacy-action-request.js"
import {
  requestMembershipApplication,
  requestMembershipEmailInvitation,
  requestMembershipJoin,
  requestMembershipPassword
} from "../src/lib/wikidot/wikidot-membership-action-request.js"

const success = (res) => ({ type: "success", data: { res } })

const requestRecorder = (result = success(null)) => {
  const requests = []
  return {
    fetch: async (url, init) => {
      requests.push({ url, init })
      return { text: async () => "serialized" }
    },
    deserialize: () => result,
    requests
  }
}

test("set-tags submits only the server registry index and revision binding", async () => {
  const recorder = requestRecorder(success({ revision_id: 91 }))
  await requestLegacySetTags(recorder, {
    pageId: 42,
    lastRevisionId: 90,
    actionIndex: 3,
    actionFingerprint: "0123456789abcdef0123456789abcdef",
    tags: ["forged"],
    alterations: ["+forged"]
  })

  assert.equal(recorder.requests.length, 1)
  const [{ url, init }] = recorder.requests
  assert.equal(url, "?/legacySetTags")
  assert.equal(init.method, "POST")
  assert.equal(init.credentials, "same-origin")
  assert.deepEqual(JSON.parse(init.body), {
    actionIndex: 3,
    actionFingerprint: "0123456789abcdef0123456789abcdef",
    lastRevisionId: 90,
    pageId: 42
  })
})

test("Rate requests submit only the server registry binding", async () => {
  const recorder = requestRecorder(success({ page_vote_id: 5 }))
  const fingerprint = "0123456789abcdef0123456789abcdef"
  await requestLegacyRate(recorder, {
    pageId: 42,
    lastRevisionId: 90,
    actionIndex: 3,
    actionFingerprint: fingerprint,
    value: -1,
    score: 500,
    count: 80,
    revisionId: 90,
    userId: 7,
    siteId: 2
  })
  await requestLegacyRateCancel(recorder, {
    pageId: 42,
    lastRevisionId: 90,
    actionIndex: 4,
    actionFingerprint: fingerprint,
    score: 500
  })
  await requestLegacyScore(recorder)

  const scoreRequest = recorder.requests.find(({ url }) => url === "?/score")
  assert.equal(scoreRequest?.init.method, "POST")
  // A bodyless score POST is rejected with HTTP 415 before the SvelteKit
  // action executes (issue #2060); the JSON body must always be present.
  assert.equal(typeof scoreRequest?.init.body, "string")
  assert.deepEqual(JSON.parse(scoreRequest?.init.body), {})

  assert.deepEqual(
    recorder.requests.map(({ url, init }) => [url, init.body && JSON.parse(init.body)]),
    [
      [
        "?/legacyRate",
        {
          actionFingerprint: fingerprint,
          actionIndex: 3,
          lastRevisionId: 90,
          pageId: 42
        }
      ],
      [
        "?/legacyRate",
        {
          actionFingerprint: fingerprint,
          actionIndex: 4,
          lastRevisionId: 90,
          pageId: 42
        }
      ],
      ["?/score", {}]
    ]
  )
})

test("server action failures remain observable to the action state machine", async () => {
  const recorder = requestRecorder({
    type: "failure",
    data: { message: "Permission denied.", code: "permission", data: null }
  })

  await assert.rejects(
    requestLegacyRate(recorder, {
      pageId: 42,
      lastRevisionId: 90,
      actionIndex: 0,
      actionFingerprint: "0123456789abcdef0123456789abcdef"
    }),
    /Permission denied\./u
  )
})

test("Join submits only the server registry and revision binding", async () => {
  const recorder = requestRecorder(success({ outcome: "joined" }))
  const fingerprint = "0123456789abcdef0123456789abcdef"
  await requestMembershipJoin(recorder, {
    pageId: 42,
    lastRevisionId: 90,
    actionIndex: 3,
    actionFingerprint: fingerprint,
    userId: 91,
    siteId: 42,
    policy: "open",
    policyRevision: 7,
    invitationToken: "secret"
  })

  assert.deepEqual(
    recorder.requests.map(({ url, init }) => [url, init.method, JSON.parse(init.body)]),
    [
      [
        "?/membershipJoin",
        "POST",
        {
          actionFingerprint: fingerprint,
          actionIndex: 3,
          lastRevisionId: 90,
          pageId: 42
        }
      ]
    ]
  )
})

test("membership application and password requests keep secrets in same-origin action bodies", async () => {
  const fingerprint = "0123456789abcdef0123456789abcdef"
  const application = requestRecorder(success("submitted"))
  assert.equal(
    await requestMembershipApplication(application, {
      pageId: 42,
      lastRevisionId: 90,
      actionIndex: 1,
      actionFingerprint: fingerprint,
      comment: "application text"
    }),
    "submitted"
  )
  assert.deepEqual(JSON.parse(application.requests[0].init.body), {
    pageId: 42,
    lastRevisionId: 90,
    actionIndex: 1,
    actionFingerprint: fingerprint,
    comment: "application text"
  })
  assert.equal(application.requests[0].url, "?/membershipApplication")
  assert.equal(application.requests[0].init.credentials, "same-origin")

  const password = requestRecorder(success("wrong_password"))
  assert.equal(
    await requestMembershipPassword(password, {
      pageId: 42,
      lastRevisionId: 90,
      actionIndex: 2,
      actionFingerprint: fingerprint,
      password: "not-the-membership-password"
    }),
    "wrong_password"
  )
  assert.deepEqual(JSON.parse(password.requests[0].init.body), {
    pageId: 42,
    lastRevisionId: 90,
    actionIndex: 2,
    actionFingerprint: fingerprint,
    password: "not-the-membership-password"
  })
  assert.equal(password.requests[0].url, "?/membershipPassword")
  assert.equal(password.requests[0].init.credentials, "same-origin")
})

test("membership email invitation keeps the opaque route hash out of the action body", async () => {
  const fingerprint = "0123456789abcdef0123456789abcdef"
  const recorder = requestRecorder(
    success({ status: "accepted", site_name: "Test Wiki", site_slug: "test" })
  )
  const result = await requestMembershipEmailInvitation(recorder, {
    pageId: 42,
    lastRevisionId: 90,
    actionIndex: 3,
    actionFingerprint: fingerprint,
    hash: "browser-must-not-forward-this"
  })

  assert.deepEqual(result, {
    status: "accepted",
    site_name: "Test Wiki",
    site_slug: "test"
  })
  assert.equal(recorder.requests.length, 1)
  assert.equal(recorder.requests[0].url, "?/membershipEmailInvitation")
  assert.equal(recorder.requests[0].init.method, "POST")
  assert.equal(recorder.requests[0].init.credentials, "same-origin")
  const body = JSON.parse(recorder.requests[0].init.body)
  assert.deepEqual(body, {
    pageId: 42,
    lastRevisionId: 90,
    actionIndex: 3,
    actionFingerprint: fingerprint
  })
  assert.equal(JSON.stringify(body).includes("browser-must-not-forward-this"), false)
})


test("membership requests surface server action failures", async () => {
  const failure = { type: "failure", data: { message: "Membership denied." } }
  const fingerprint = "0123456789abcdef0123456789abcdef"
  const base = { pageId: 42, lastRevisionId: 90, actionIndex: 1, actionFingerprint: fingerprint }
  for (const [request, input] of [
    [requestMembershipJoin, base],
    [requestMembershipPassword, { ...base, password: "secret" }],
    [requestMembershipApplication, { ...base, comment: "hello" }],
    [requestMembershipEmailInvitation, base]
  ]) {
    await assert.rejects(request(requestRecorder(failure), input), /Membership denied\./u)
  }
})
