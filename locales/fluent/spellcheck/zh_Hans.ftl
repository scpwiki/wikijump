### 工具架拼写检查

spellcheck-word =
  .misspelled = 错拼单词：“{ $slice }”
  .forbidden = 单词 “{ $slice }” 被禁用。
  .warned = 单词 “{ $slice }” 可能并不是您希望使用的词。

spellcheck-add-word = 将 “{ $slice }” 添加至词库
  .tooltip = 将单词 “{ $slice }” 添加至您的本地词库

spellcheck-accept =
  将单词 “{ $slice }” 替换为 “{ $suggestion }”

spellcheck-source =
  spellchecker(word: { $slice }) [{ $from }, { $to }]
