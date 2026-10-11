import assert from "node:assert/strict"
import { randomUUID } from "node:crypto"
import { spawn } from "node:child_process"
import {
  lstatSync,
  mkdirSync,
  readFileSync,
  rmdirSync,
  statSync,
  unlinkSync,
  writeFileSync
} from "node:fs"
import https from "node:https"
import path from "node:path"
import { once } from "node:events"
import { test } from "node:test"

import {
  cleanupOwnedTlsDirectory,
  createOwnedTlsDirectory
} from "./playwright-https-tls.js"

const tlsDirectoryA = "/tmp/wikijump-playwright-tls-fixture-restore-a"
const tlsDirectoryB = "/tmp/wikijump-playwright-tls-fixture-restore-b"
const startScript = "tests/start-playwright-https-vite.sh"

const requestHttps = (port, requestPath = "/@vite/client") =>
  new Promise((resolve, reject) => {
    const request = https.get(
      `https://127.0.0.1:${port}${requestPath}`,
      { rejectUnauthorized: false, timeout: 2000 },
      (response) => {
        response.resume()
        response.once("end", () => resolve(response.statusCode))
      }
    )
    request.once("timeout", () =>
      request.destroy(new Error("HTTPS fixture request timed out"))
    )
    request.once("error", reject)
  })

const waitForHttps = async (port, child) => {
  const deadline = Date.now() + 90_000
  let lastError

  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(
        `HTTPS Vite fixture exited before becoming ready (${child.exitCode})`
      )
    }
    try {
      const status = await requestHttps(port)
      assert.equal(status, 200)
      return
    } catch (error) {
      lastError = error
      await new Promise((resolve) => setTimeout(resolve, 250))
    }
  }

  throw new Error(
    `HTTPS Vite fixture did not become ready: ${lastError?.message ?? "timeout"}`
  )
}

const startHttpsFixture = (port, tlsDirectory, token) => {
  const child = spawn("/bin/sh", [startScript], {
    cwd: process.cwd(),
    env: {
      ...process.env,
      FRAMERAIL_VITE_TEST_CACHE: "1",
      PLAYWRIGHT_HTTPS_APP_PORT: String(port),
      PLAYWRIGHT_FIXTURE_PORT: "42747",
      WIKIJUMP_PLAYWRIGHT_TLS_DIR: tlsDirectory,
      WIKIJUMP_PLAYWRIGHT_TLS_OWNER_TOKEN: token
    },
    stdio: "ignore"
  })

  return child
}

const stopChild = async (child) => {
  if (child.exitCode !== null || child.signalCode !== null) return
  const exitPromise = once(child, "exit")
  child.kill("SIGTERM")
  await exitPromise
}

test("Playwright HTTPS fixture rejects unsafe and already-existing TLS paths", () => {
  const token = randomUUID()
  assert.throws(() =>
    createOwnedTlsDirectory("/tmp/../tmp/wikijump-playwright-tls-traversal", token)
  )
  assert.throws(() =>
    createOwnedTlsDirectory("/home/roku/wikijump-playwright-tls-outside", token)
  )
  mkdirSync(tlsDirectoryA, { mode: 0o700 })
  const occupant = path.join(tlsDirectoryA, "keep-me.txt")
  writeFileSync(occupant, "pre-existing content\n", { flag: "wx", mode: 0o600 })
  try {
    assert.throws(
      () => createOwnedTlsDirectory(tlsDirectoryA, token),
      /newly-created real directory/u
    )
    assert.equal(readFileSync(occupant, "utf8"), "pre-existing content\n")
  } finally {
    unlinkSync(occupant)
    rmdirSync(tlsDirectoryA)
  }
})

test("two task-owned Playwright HTTPS fixtures handshake concurrently and clean only their own keys", async () => {
  const tokenA = randomUUID()
  const tokenB = randomUUID()
  let childA
  let childB

  try {
    childA = startHttpsFixture(49473, tlsDirectoryA, tokenA)
    childB = startHttpsFixture(49474, tlsDirectoryB, tokenB)

    await Promise.all([waitForHttps(49473, childA), waitForHttps(49474, childB)])

    for (const directory of [tlsDirectoryA, tlsDirectoryB]) {
      assert.equal(lstatSync(directory).mode & 0o777, 0o700)
      assert.equal(
        statSync(path.join(directory, "localhost-key.pem")).mode & 0o777,
        0o600
      )
      assert.equal(
        statSync(path.join(directory, "localhost-cert.pem")).mode & 0o777,
        0o600
      )
    }
    assert.notDeepEqual(
      readFileSync(path.join(tlsDirectoryA, "localhost-cert.pem")),
      readFileSync(path.join(tlsDirectoryB, "localhost-cert.pem"))
    )

    assert.throws(
      () => cleanupOwnedTlsDirectory(tlsDirectoryA, tokenB),
      /owned by another run/u
    )
    const unexpectedFile = path.join(tlsDirectoryA, "do-not-delete.txt")
    writeFileSync(unexpectedFile, "preserve unexpected content\n", {
      flag: "wx",
      mode: 0o600
    })
    assert.throws(
      () => cleanupOwnedTlsDirectory(tlsDirectoryA, tokenA),
      /unexpected content/u
    )
    assert.equal(readFileSync(unexpectedFile, "utf8"), "preserve unexpected content\n")
    unlinkSync(unexpectedFile)
  } finally {
    if (childA) await stopChild(childA)
    if (childB) await stopChild(childB)
    cleanupOwnedTlsDirectory(tlsDirectoryA, tokenA)
    cleanupOwnedTlsDirectory(tlsDirectoryB, tokenB)
  }

  assert.throws(() => lstatSync(tlsDirectoryA), { code: "ENOENT" })
  assert.throws(() => lstatSync(tlsDirectoryB), { code: "ENOENT" })
})
