### FTML 扩展镜像代码

cmftml-undocumented-block = 此块可用，但仍未编写与其有关的文档。

## 静态代码分析

cmftml-lint =
  .warning-source = ftml({ $rule } = { $kind } at { $token }) [{ NUMBER($from, useGrouping: 0) }, { NUMBER($to, useGrouping: 0) }]

  .recursion-depth-exceeded = 标记中出现过多递归。

  .end-of-input = 规则类型 “{ $rule }” 无法在文档结束前被处理。

  .no-rules-matched = 字符串 “{ $slice }” 不与任何现有字符串匹配，且将以纯文本渲染。

  .rule-failed = 规则 “{ $rule }” 在此处匹配失败，且必须回退为另一条规则。

  .not-start-of-line = 规则 “{ $rule }” 在此处匹配失败，因为其仅可在位于新的一行起始时被匹配。

  .invalid-include = 引用不可用，且将不会被渲染。

  .list-empty = 列表内未包含内容。

  .list-contains-non-item = 列表的直系子元素并非列项。

  .list-item-outside-list = 列项未位于一个列表内。

  .list-depth-exceeded = 列表嵌套得过于深入了，其无法被渲染。

  .table-contains-non-row = 表格的直系子元素并非表格行。

  .table-row-contains-non-cell = 表格行的直系子元素并非单元格。

  .table-row-outside-table = 表格行未位于表格内。

  .table-cell-outside-table = 单元格未位于表格行内。

  .footnotes-nested = 脚注不可用，因其位于另一个脚注内。

  .blockquote-depth-exceeded = 引用块嵌套得过于深入了，其无法被渲染。

  .no-such-block = 未知块 “{ $slice }”。

  .block-disallows-star = 块 “{ $slice }” 不支持星号调用。（以 “*” 字符起始）

  .block-disallows-score = 块 “{ $slice }” 不支持下划线调用。（以 “_” 字符起始）

  .block-missing-name = 块 “{ $slice }” 需求一个名称/值，但其未指定任何名称/值。

  .block-missing-close-brackets = 该块缺少结束用的 “]]” 方括号。

  .black-malformed-arguments = 块 “{ $slice }” 出现异常值。

  .block-missing-arguments = 块 “{ $slice }” 缺失一个或多个必要参数。

  .block-expected-end = “{ $rule }” 类型的块预期至少要在此处结束。

  .block-end-mismatch = “{ $rule }” 类型的块预期在此处结束，而非 “{ $slice }”。

  .no-such-module = 未知模块 “{ $slice }”。

  .module-missing-name = 预期需提供一个模块名称。

  .no-such-page = 页面 “{ $slice }” 不存在。

  .invalid-url = 链接 “{ $slice }” 不存在。

## 块的接受度

cmftml-accepts =
  .star =
    该块接受 “*”（星号）前缀。添加此前缀的效果根据块的不同而有所变化。

  .score =
    该块接受 “_”（下划线）后缀，其将去除前后的换行符。

  .newlines =
    该块在其节点的起始与结尾处接受换行符。

  .html-attributes =
    该块接受通用的 HTML 属性/参数。HTML 属性受制于白名单，但无论如何大多数属性都可以使用。

## 块参数类型

cmftml-argument-none = 无
  .info = 该块不接受任何参数。

cmftml-argument-value = 值
  .info = 该块在其节点的起始与结尾处接受文本。

cmftml-argument-map = 映射
  .info = 该块接受参数。

cmftml-argument-value-map = 值 + 映射
  .info = 该块接受文本，且在加空格后亦接受参数。

## 块主体类型

cmftml-body-none = 无
  .info = 该块无主体，且无需结束节点。

cmftml-body-raw = 纯
  .info = 该块接受主体，但会将主体以纯文本的形式编译。

cmftml-body-elements = 元素
  .info = 该块接受主体，且可在其内嵌套额外的元素。

cmftml-body-other = 其他
  .info = 该块拥有特殊的格式，因此不容易被分类。
