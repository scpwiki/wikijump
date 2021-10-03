import type { Argument } from "./types"

// TODO: should probably make html attrs discriminate based on block name
export const htmlAttributes: Record<string, Argument> = {
  accept: { type: "string" },
  alt: { type: "string" },
  autocapitalize: {
    type: "string",
    enum: ["off", "none", "on", "sentences", "words", "characters"]
  },
  autoplay: { type: "bool" },
  // not accessible from HTML apparently?
  // name: "buffered" values: ["true" "false"]
  checked: { type: "bool" },
  cite: { type: "string" },
  class: { type: "string" },
  cols: { type: "string" },
  colspan: { type: "string" },
  // for the purposes of FTML this is a boolean attribute
  // however in HTML it's actually enumerated
  contenteditable: { type: "bool" },
  controls: { type: "bool" },
  coords: { type: "string" },
  datetime: { type: "string" },
  decoding: { type: "string", enum: ["true", "false"] },
  default: { type: "bool" },
  dir: { type: "string" },
  dirname: { type: "string" },
  disabled: { type: "bool" },
  // can be given without a value like a boolean
  download: { type: "string" },
  // not a boolean apparently
  // empty values aren't true either
  draggable: { type: "string", enum: ["true", "false"] },
  for: { type: "string" },
  form: { type: "string" },
  headers: { type: "string" },
  height: { type: "string" },
  hidden: { type: "bool" },
  high: { type: "string" },
  href: { type: "string" },
  hreflang: { type: "string" },
  id: { type: "string" },
  inputmode: {
    type: "string",
    enum: ["none", "text", "decimal", "numeric", "tel", "search", "email", "url"]
  },
  ismap: { type: "bool" },
  itemprop: { type: "string" },
  kind: {
    type: "string",
    enum: ["subtitles", "captions", "descriptions", "chapters", "metadata"]
  },
  label: { type: "string" },
  lang: { type: "string" },
  list: { type: "string" },
  loop: { type: "bool" },
  low: { type: "string" },
  max: { type: "string" },
  maxlength: { type: "string" },
  minlength: { type: "string" },
  min: { type: "string" },
  multiple: { type: "bool" },
  muted: { type: "bool" },
  name: { type: "string" },
  optimum: { type: "string" },
  pattern: { type: "string" },
  placeholder: { type: "string" },
  poster: { type: "string" },
  // empty string is "auto"
  preload: { type: "string", enum: ["none", "metadata", "auto"] },
  readonly: { type: "bool" },
  required: { type: "bool" },
  reversed: { type: "bool" },
  scope: { type: "string", enum: ["row", "col", "rowgroup", "colgroup"] },
  selected: { type: "bool" },
  shape: { type: "string", enum: ["rect", "circle", "poly", "default"] },
  size: { type: "string" },
  sizes: { type: "string" },
  // not boolean but "true"|"false" is _required_
  spellcheck: { type: "string", enum: ["true", "false"] },
  src: { type: "string" },
  srclang: { type: "string" },
  srcset: { type: "string" },
  start: { type: "string" },
  step: { type: "string" },
  style: { type: "string" },
  tabindex: { type: "string" },
  target: { type: "string", enum: ["_self", "_blank", "_parent", "_top"] },
  title: { type: "string" },
  // this is not a boolean and it requires "yes"|"no". seriously.
  translate: { type: "string", enum: ["yes", "no"] },
  // this is a union of all the possible values of "type"
  // this is really just a fallback
  type: {
    type: "string",
    enum: [
      "submit",
      "reset",
      "button",
      "checkbox",
      "color",
      "date",
      "datetime-local",
      "email",
      "file",
      "hidden",
      "image",
      "month",
      "number",
      "password",
      "radio",
      "range",
      "reset",
      "search",
      "submit",
      "tel",
      "text",
      "time",
      "url",
      "week"
    ]
  },
  usemap: { type: "string" },
  width: { type: "string" },
  wrap: { type: "string", enum: ["hard", "soft", "off"] }
  // DEPRECATED
  // probably more should go here - but it may depend on the block
  // TODO: allow deprecated attributes
  // {
  //   name: "align",
  //   deprecated: true,
  //   enum: ["top", "middle", "bottom", "left", "right"]
  // },
  // { name: "background", deprecated: true },
  // { name: "bgcolor", deprecated: true },
  // { name: "border", deprecated: true }
}
