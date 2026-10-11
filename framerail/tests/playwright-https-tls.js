import { randomUUID } from "node:crypto"
import {
  chmodSync,
  existsSync,
  lstatSync,
  mkdirSync,
  realpathSync,
  readFileSync,
  readdirSync,
  rmdirSync,
  unlinkSync,
  writeFileSync
} from "node:fs"
import path from "node:path"
import { pathToFileURL } from "node:url"

const markerName = ".wikijump-playwright-tls-owner"
const generatedFiles = ["localhost-key.pem", "localhost-cert.pem"]
const tlsDirectoryPattern = /^wikijump-playwright-tls-[a-zA-Z0-9-]{1,80}$/
const ownerTokenPattern = /^[a-f0-9-]{36}$/

const canonicalTlsDirectory = (directory) => {
  if (typeof directory !== "string" || !path.isAbsolute(directory)) {
    throw new Error("Playwright TLS directory must be an absolute path")
  }

  const resolved = path.resolve(directory)
  if (resolved !== directory || path.dirname(resolved) !== "/tmp") {
    throw new Error("Playwright TLS directory must be a canonical direct child of /tmp")
  }

  const canonicalTmp = realpathSync("/tmp")
  if (canonicalTmp !== "/tmp" || !tlsDirectoryPattern.exec(path.basename(resolved))) {
    throw new Error(
      "Playwright TLS directory is outside the permitted task-owned /tmp namespace"
    )
  }

  return resolved
}

const ownerMarker = (token) => {
  if (typeof token !== "string" || !ownerTokenPattern.test(token)) {
    throw new Error("Playwright TLS owner token must be a UUID")
  }
  return `v1:${token}\n`
}

export const createOwnedTlsDirectory = (directory, token = randomUUID()) => {
  const tlsDirectory = canonicalTlsDirectory(directory)
  const markerContents = ownerMarker(token)

  let created = false
  try {
    lstatSync(tlsDirectory)
  } catch (error) {
    if (error.code !== "ENOENT") throw error
    mkdirSync(tlsDirectory, { mode: 0o700 })
    chmodSync(tlsDirectory, 0o700)
    created = true
  }

  const metadata = lstatSync(tlsDirectory)
  if (!created || !metadata.isDirectory() || metadata.isSymbolicLink()) {
    throw new Error("Playwright TLS directory must be a newly-created real directory")
  }
  if (readdirSync(tlsDirectory).length !== 0) {
    throw new Error("Playwright TLS directory must be empty before use")
  }

  try {
    writeFileSync(path.join(tlsDirectory, markerName), markerContents, {
      flag: "wx",
      mode: 0o600
    })
  } catch (error) {
    try {
      rmdirSync(tlsDirectory)
    } catch {
      // Leave any directory that gained unexpected content untouched.
    }
    throw error
  }

  return { directory: tlsDirectory, token }
}

export const cleanupOwnedTlsDirectory = (directory, token) => {
  const tlsDirectory = canonicalTlsDirectory(directory)
  const markerContents = ownerMarker(token)

  try {
    const directoryStat = lstatSync(tlsDirectory)
    if (!directoryStat.isDirectory() || directoryStat.isSymbolicLink()) {
      throw new Error("Refusing to clean a non-directory or symlink Playwright TLS path")
    }
  } catch (error) {
    if (error.code === "ENOENT") return false
    throw error
  }

  const markerPath = path.join(tlsDirectory, markerName)
  const markerStat = lstatSync(markerPath)
  if (!markerStat.isFile() || markerStat.isSymbolicLink()) {
    throw new Error(
      "Refusing to clean a Playwright TLS directory without a regular owner marker"
    )
  }
  if (readFileSync(markerPath, "utf8") !== markerContents) {
    throw new Error("Refusing to clean a Playwright TLS directory owned by another run")
  }

  const allowedNames = new Set([...generatedFiles, markerName])
  for (const name of readdirSync(tlsDirectory)) {
    if (!allowedNames.has(name)) {
      throw new Error(
        "Refusing to remove unexpected content from a Playwright TLS directory"
      )
    }
    const filePath = path.join(tlsDirectory, name)
    const fileStat = lstatSync(filePath)
    if (!fileStat.isFile() || fileStat.isSymbolicLink()) {
      throw new Error("Refusing to remove a non-regular Playwright TLS file")
    }
  }

  for (const name of generatedFiles) {
    const filePath = path.join(tlsDirectory, name)
    if (existsSync(filePath)) unlinkSync(filePath)
  }
  unlinkSync(markerPath)
  rmdirSync(tlsDirectory)
  return true
}

const runCli = () => {
  const [, , operation, directory, token] = process.argv
  try {
    if (operation === "create") {
      const result = createOwnedTlsDirectory(directory, token)
      process.stdout.write(`${result.directory}\n`)
      return
    }
    if (operation === "cleanup") {
      cleanupOwnedTlsDirectory(directory, token)
      return
    }
    throw new Error("Expected create or cleanup operation")
  } catch (error) {
    process.stderr.write(`${error.message}\n`)
    process.exitCode = 1
  }
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href
) {
  runCli()
}
