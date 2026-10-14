import { execFileSync } from "node:child_process"
import { existsSync } from "node:fs"
import path from "node:path"
import { fileURLToPath } from "node:url"

const REPOSITORY_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../../..")

// The main checkout of this repository, even when running from a linked
// worktree (whose own parent directory has no sibling checkouts).
function mainCheckout(root) {
  try {
    const commonDir = execFileSync("git", ["-C", root, "rev-parse", "--path-format=absolute", "--git-common-dir"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"]
    }).trim()
    return path.dirname(commonDir)
  } catch {
    return root
  }
}

/** Resolve a sibling checkout such as ../ftml: an explicit environment
 * override first, then a sibling of this checkout, then a sibling of the
 * main checkout of a linked worktree. */
export function siblingCheckout(name, environmentVariable, root = REPOSITORY_ROOT) {
  const override = process.env[environmentVariable]
  if (override) return override
  const direct = path.resolve(root, "..", name)
  if (existsSync(direct)) return direct
  return path.resolve(mainCheckout(root), "..", name)
}
