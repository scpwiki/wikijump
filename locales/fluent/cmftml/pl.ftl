### CodeMirror FTML Extension

cmftml-undocumented-block = Podany blok jest prawidłowy, ale nie istnieje jeszcze jego dokumentacja.

## Linting

cmftml-lint =
  .warning-source = ftml({ $rule } = { $kind } w { $token }) [{ NUMBER($from, useGrouping: 0) }, { NUMBER($to, useGrouping: 0) }]

  .recursion-depth-exceeded = Zbyt dużo rekurencji w znaczniku.

  .end-of-input = Reguła typu '{ $rule }' nie mogła zostać przetworzona przed osiągnięciem końca dokumentu.

  .no-rules-matched = Wyszukiwanie '{ $slice }' nie zgadza się z żadnym źródłem, więc zostanie pokazane jako czysty tekst.

  .rule-failed = Reguła '{ $rule }' nie wyszukała nic w tym miejscu i musiała wycofać inną regułę.

  .not-start-of-line = Reguła '{ $rule }' nie wyszukała nic w tym miejscu, bo może szukać jedynie zaczynając od początku nowego wiersza.

  .invalid-include = To włączenie jest nieprawidłowe i nie zostanie wygenerowane.

  .list-empty = W tej liście nic się nie znajduje.

  .list-contains-non-item = Lista ma bezpośrednie węzły potomne, które nie są blokami list-item.

  .list-item-outside-list = Ten obiekt list-item nie jest w liście.

  .list-depth-exceeded = Lista jest zagnieżdżona zbyt głęboko i nie może być wygenerowana.

  .table-contains-non-row = Tabela ma bezpośrednie węzły potomne, które nie są wierszami tabeli.

  .table-row-contains-non-cell = Wiersz tabeli ma bezpośrednie węzły potomne, które nie są komórkamk tabeli.

  .table-row-outside-table = Wiesz nie znajduje się w tabeli.

  .table-cell-outside-table = Komórka tabeli nie znajduje się w wierszu.

  .footnotes-nested = Przypis jest nieprawidłowy, ponieważ znajduje się w innym przypisie.

  .blockquote-depth-exceeded = Blockquote jest zagnieżdżony zbyt głęboko i nie może zostać wygenerowany.
  .no-such-block = Unknown block '{ $slice }'.

  .block-disallows-star = Blok '{ $slice }' nie obsługuje gwiazdek. (rozpoczynających się znakiem '*')

  .block-disallows-score = Blok '{ $slice }' nie obsługuje wyników. (rozpoczynających się zmakiem '_')

  .block-missing-name = Blok '{ $slice }' wymaga nazwy lub wartości, której nie podano.

  .block-missing-close-brackets = W bloku brakuje nawiasów zamykających ']]'.

  .black-malformed-arguments = Blok '{ $slice }' zniekształcił argumenty.

  .block-missing-arguments = W bloku '{ $slice }' brakuje jednego lub więcej wymaganych argumentów.

  .block-expected-end = Blok typu '{ $rule }' powinien zakończyć się w tym miejscu.
  .block-end-mismatch = Blok typu '{ $rule }' powinien zakończyć się tutaj, a nie w '{ $slice }'.

  .no-such-module = Nieznany moduł '{ $slice }'.

  .module-missing-name = Moduł powinien zawierać nazwę.

  .no-such-page = Strona '{ $slice }' nie istnieje.

  .invalid-url = Adres URL '{ $slice }' jest nieprawidłowy.

## Block Acceptance

cmftml-accepts =
  .star =
    Blok akceptuje prefiks '*' (gwiazdek).
    Efekt użycia tego prefiksu zależy od bloku.

  .score =
    Blok akceptuje sufiks '_' (wyniku),
    który usunie wiodące i końcowe znaki nowych linii.

  .newlines =
    Blok akceptuje nowe linie między początkiem a końcem węzła (node).

  .html-attributes =
    Blok akceptuje atrybuty/argumenty HTML.
    Atrybuty HTML są obiektem na białej liście, ale większości z nich można użyć niezależnie.

## Block Argument Types

cmftml-argument-none = BRAK
  .info = Ten blok nie akceptuje żadnych argumentów.

cmftml-argument-value = WARTOŚĆ
  .info = Ten blok akceptuje tekst między początkiem a końcem węzła (node).

cmftml-argument-map = MAPA
  .info = Ten blok akceptuje argumenty.

cmftml-argument-value-map = WARTOŚĆ+MAPA
  .info = Ten blok akceptuje tekst oraz argumenty, które są po spacji.

## Block Body Types

cmftml-body-none = BRAK
  .info = Ten blok nie zawiera ciała (body) i nie wymaga końcowego węzła (node).

cmftml-body-raw = CZYSTY-TEKST
  .info = Ten blok akceptuje ciało (body), ale interpretuje je jako czysty tekst.

cmftml-body-elements = ELEMENTY
  .info = Ten blok akceptuje ciało (body) i może zagnieździć w nim dodatkowe elementy.

cmftml-body-other = INNE
  .info = Ten blok używa specjalnej składni, która nie może być skategoryzowana.
