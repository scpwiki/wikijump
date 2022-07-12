### CodeMirror FTML Extension

cmftml-undocumented-block = 이 블록은 올바르지만 아직 설명이 쓰이지 않았습니다.

## Linting

cmftml-lint =
  .warning-source = ftml({ $rule } = { $kind } at { $token }) [{ NUMBER($from, useGrouping: 0) }, { NUMBER($to, useGrouping: 0) }]

  .recursion-depth-exceeded = 마크업에 재귀가 너무 많습니다.

  .end-of-input = 문서의 끝에 이르기 전에 '{ $rule }' 유형의 규칙을 처리할 수 없습니다. 

  .no-rules-matched = 문자열 '{ $slice }'와(과) 일치하는 규칙이 없어 일반 텍스트로 처리됩니다.

  .rule-failed = '{ $rule }' 규칙이 이 부분과 일치하지 않아 다른 규칙으로 대체되었습니다.

  .not-start-of-line = '{ $rule }' 규칙은 새 줄의 맨 앞에서만 해당하므로 이 부분과 일치하지 않습니다.

  .invalid-include = 이 포함은 올바르지 않으며 처리되지 않습니다.

  .list-empty = 이 목록 안에는 아무것도 없습니다.

  .list-contains-non-item = 이 목록의 바로 아래 자식 중 항목 블록이 아닌 것이 있습니다.

  .list-item-outside-list = 이 항목은 목록 밖에 있습니다.

  .list-depth-exceeded = 이 목록은 너무 많이 중첩되어 처리할 수 없습니다.

  .table-contains-non-row = 이 표의 바로 아래 자식 중 행이 아닌 것이 있습니다.

  .table-row-contains-non-cell = 이 행의 바로 아래 자식 중 셀이 아닌 것이 있습니다.

  .table-row-outside-table = 이 행은 표 밖에 있습니다.

  .table-cell-outside-table = 이 셀은 행 밖에 있습니다.

  .footnotes-nested = 이 각주는 다른 각주 안에 있으므로 올바르지 않습니다.

  .blockquote-depth-exceeded = 이 인용문은 너무 많이 중첩되어 처리할 수 없습니다.

  .no-such-block = '{ $slice }'은(는) 알 수 없는 블록입니다.

  .block-disallows-star = '{ $slice }' 블록은 별표 첨가('*' 문자로 시작)를 지원하지 않습니다.

  .block-disallows-score = '{ $slice }' 블록은 밑줄 문자 첨가('_' 문자로 시작)를 지원하지 않습니다.

  .block-missing-name = '{ $slice }' 블록에게 필요한 이름이나 값이 주어지지 않았습니다.

  .block-missing-close-brackets = 이 블록은 닫는 괄호(']]')가 없습니다.

  .black-malformed-arguments = '{ $slice }' 블록의 인수가 잘못되었습니다.

  .block-missing-arguments = '{ $slice }' 블록에게 필요한 한 개 이상의 인수가 주어지지 않았습니다.

  .block-expected-end = '{ $rule }' 유형의 블록은 적어도 이 지점에서 끝나야 합니다. 

  .block-end-mismatch = '{ $rule }' 유형의 블록은 '{ $slice }'이(가) 아니라 이 지점에서 끝나야 합니다.

  .no-such-module = '{ $slice }'은(는) 알 수 없는 모듈입니다.

  .module-missing-name = 모듈 이름이 입력되어야 합니다.

  .no-such-page = '{ $slice }'(이)라는 페이지는 존재하지 않습니다.

  .invalid-url = URL '{ $slice }'은(는) 올바르지 않습니다.

## Block Acceptance

cmftml-accepts =
  .star =
    이 블록은 '*'(별표) 접두사를 받을 수 있습니다.
    별표 접두사의 역할은 블록에 따라 달라집니다.

  .score =
    이 블록은 '_'(밑줄 문자) 접미사를 받을 수 있습니다.
    밑줄 문자 접미사는 블록의 앞과 뒤에 새 줄이 만들어지지 않게 합니다.

  .newlines =
    이 블록의 시작 노드와 종료 노드 사이에 새 줄을 넣을 수 있습니다.

  .html-attributes =
    이 블록은 일반적인 HTML 속성이나 인수를 받을 수 있습니다.
    금지된 HTML 속성도 있지만 대부분 쓸 수 있습니다.

## Block Argument Types

cmftml-argument-none = NONE
  .info = 이 블록은 어떤 인수도 받지 않습니다.

cmftml-argument-value = VALUE
  .info = 이 블록은 노드의 시작과 끝 사이에 텍스트를 받을 수 있습니다.

cmftml-argument-map = MAP
  .info = 이 블록은 인수를 받을 수 있습니다.

cmftml-argument-value-map = VALUE+MAP
  .info = 이 블록은 텍스트를 받을 수 있으며, 텍스트 뒤를 공백으로 한 칸 띄우고 나면 인수를 받습니다.

## Block Body Types

cmftml-body-none = NONE
  .info = 이 블록에는 내용을 넣을 수 없으며 종료 노드가 필요하지 않습니다.

cmftml-body-raw = RAW
  .info = 이 블록에는 내용을 넣을 수 있으나 이를 일반 텍스트로 해석합니다. 

cmftml-body-elements = ELEMENTS
  .info = 이 블록에는 내용을 넣을 수 있으며 안에 다른 요소를 중첩할 수 있습니다.

cmftml-body-other = OTHER
  .info = 이 블록은 분류하기 어려운 특수한 구문을 갖고 있습니다.
