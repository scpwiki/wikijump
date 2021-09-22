import { createLezerLanguage } from "@wikijump/codemirror"
import {
  continuedIndent,
  cssCompletion,
  cssLanguage,
  indentNodeProp,
  styleTags,
  tags as t
} from "@wikijump/codemirror/cm"
import { parser as CSSAttrParser } from "../../vendor/css-attribute"

export const StyleAttributeGrammar = createLezerLanguage({
  name: "style-attribute",
  parser: CSSAttrParser,
  languageData: {
    // @ts-ignore
    ...cssLanguage.data.default[0],
    // @ts-ignore
    ...cssCompletion.value
  },
  configure: {
    props: [
      indentNodeProp.add({
        Declaration: continuedIndent()
      }),
      styleTags({
        "PropertyName": t.propertyName,
        "NumberLiteral": t.number,
        "callee": t.keyword,
        "CallTag ValueName": t.atom,
        "Callee": t.variableName,
        "Unit": t.unit,
        "BinOp": t.arithmeticOperator,
        "Important": t.modifier,
        "Comment": t.blockComment,
        "ParenthesizedContent": t.special(t.name),
        "ColorLiteral": t.color,
        "StringLiteral": t.string,
        ":": t.punctuation,
        "PseudoOp #": t.derefOperator,
        "; ,": t.separator,
        "( )": t.paren,
        "[ ]": t.squareBracket,
        "{ }": t.brace
      })
    ]
  }
})
