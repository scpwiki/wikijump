const pageLabels = {
  en: {
    "user-profile-info": "User profile information",
    "wiki-page-append-content": "Wikitext to append",
    "wiki-page-file-upload.select": "Select file:",
    "wiki-page-file-upload.name": "File name:",
    "wiki-page-layout": "Page layout",
    "wiki-page-meta-tag-name": "Meta tag name",
    "wiki-page-meta-tag-content": "Meta tag content"
  },
  ja: {
    "user-profile-info": "ユーザープロフィール情報",
    "wiki-page-append-content": "追記するWikiテキスト",
    "wiki-page-file-upload.select": "ファイルを選択:",
    "wiki-page-file-upload.name": "ファイル名:",
    "wiki-page-layout": "ページレイアウト",
    "wiki-page-meta-tag-name": "メタタグ名",
    "wiki-page-meta-tag-content": "メタタグの内容"
  }
}

/** @param {{ rpcRequest: any }} input */
export const handlePageLabelTranslationRpc = ({ rpcRequest }) => {
  if (
    rpcRequest.method !== "translate" ||
    !Array.isArray(rpcRequest.params?.locales) ||
    typeof rpcRequest.params.messages !== "object" ||
    rpcRequest.params.messages === null
  ) {
    return undefined
  }

  const keys = Object.keys(rpcRequest.params.messages)
  if (!keys.some((key) => Object.hasOwn(pageLabels.en, key))) return undefined

  const locale = rpcRequest.params.locales.some((value) =>
    `${value}`.toLowerCase().startsWith("ja")
  )
    ? "ja"
    : "en"
  return {
    result: Object.fromEntries(keys.map((key) => [key, pageLabels[locale][key] ?? key]))
  }
}
