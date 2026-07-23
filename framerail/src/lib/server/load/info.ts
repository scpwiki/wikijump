import "$lib/vite-env.d.ts"

import defaults from "$lib/defaults"
import process from "process"

import { info } from "$lib/server/deepwell"
import { translate } from "$lib/server/deepwell/translate"

import type { PreloadDataAsync } from "$lib/server/deepwell/views"
import type { TranslateKeys } from "$lib/types"

export async function loadInfo(preloadData: PreloadDataAsync) {
  const parentData = await preloadData()
  const locales = parentData.locales

  const response = await info()

  const translateKeys: TranslateKeys = {
    ...defaults.translateKeys
  }

  const viewData = {
    backend: response,
    frontend: {
      name: serverInfo.frontendName,
      description: serverInfo.frontendDescription,
      repository: serverInfo.frontendRepository,
      version: serverInfo.frontendVersion,
      license: serverInfo.frontendLicense,
      node: process.versions.node,
      pnpm: serverInfo.pnpmVersion
    }
  }

  const internationalization = await translate(locales, translateKeys)

  return { ...viewData, internationalization }
}
