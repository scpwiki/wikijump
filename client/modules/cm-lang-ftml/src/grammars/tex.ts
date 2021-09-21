import { TarnationLanguage } from "@wikijump/cm-tarnation"
import texGrammar from "./tex.yaml"

export const TexLanguage = new TarnationLanguage({
  name: "wikimath",
  grammar: texGrammar as any
})
