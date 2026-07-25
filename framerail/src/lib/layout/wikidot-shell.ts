import { Layout } from "$lib/types"
import {
  resolveShellLayoutValue,
  shouldUseWikidotShellValue,
  type ShellViewData
} from "$lib/layout/wikidot-shell-decision"

export function shouldUseWikidotShell(data: ShellViewData | null | undefined): boolean {
  return shouldUseWikidotShellValue(data)
}

export function resolveShellLayout(data: ShellViewData | null | undefined): Layout {
  return resolveShellLayoutValue(data) as Layout
}
