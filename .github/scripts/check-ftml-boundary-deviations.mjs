import { readFileSync } from "node:fs"
import path from "node:path"
import { pathToFileURL } from "node:url"

export const REDIRECT_DEVIATION_NOTE =
  "docs/ftml-boundary-deviations/wikidot-redirect-location.md"

export const REDIRECT_SCANNER_SYMBOLS = [
  "wikidot_redirect_location",
  "REDIRECT_MODULE_PREFIX_REGEX",
  "REDIRECT_MODULE_REGEX",
  "REDIRECT_ARGUMENT_REGEX"
]

export const REDIRECT_DEVIATION_SECTIONS = [
  "Reason",
  "Why FTML is not yet sufficient",
  "Evidence",
  "FTML backlog decision",
  "Migration condition",
  "Owner",
  "Review trigger"
]

const escapeRegularExpression = (value) =>
  value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&")

export function redirectDeviationProblems(source) {
  if (source === null) return [`missing required note ${REDIRECT_DEVIATION_NOTE}`]

  const problems = []
  for (const symbol of REDIRECT_SCANNER_SYMBOLS) {
    if (!source.includes(symbol)) problems.push(`missing scanner symbol ${symbol}`)
  }
  for (const section of REDIRECT_DEVIATION_SECTIONS) {
    const heading = new RegExp(`^## ${escapeRegularExpression(section)}$`, "mu")
    if (!heading.test(source)) problems.push(`missing required section ${section}`)
  }
  return problems
}

export function checkRedirectDeviation(root) {
  let source = null
  try {
    source = readFileSync(path.join(root, REDIRECT_DEVIATION_NOTE), "utf8")
  } catch (error) {
    if (error?.code !== "ENOENT") throw error
  }

  const problems = redirectDeviationProblems(source)
  if (problems.length > 0) throw new Error(problems.join("\n"))
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    checkRedirectDeviation(path.resolve(path.dirname(process.argv[1]), "../.."))
  } catch (error) {
    console.error(error.message)
    process.exitCode = 1
  }
}
